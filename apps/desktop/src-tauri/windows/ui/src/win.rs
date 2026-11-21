//! The Windows side: takes over the NSIS window, draws the scene into it and turns clicks and
//! keys into NSIS page navigation.
//!
//! The NSIS window keeps working as usual underneath: its pages, buttons and progress bar
//! exist but stay hidden, covered by one child window of ours. Buttons of ours post the same
//! `WM_COMMAND` that NSIS's Next and Cancel buttons send, so NSIS runs its page callbacks as
//! always. The window frame is removed (`WM_NCCALCSIZE`) and the whole background drags it.

use crate::nsis::{Call, ExtraParameters, StackT, Vars, INST_INSTDIR, NSPIM_GUIUNLOAD};
use crate::ui::{Hit, Kind, PageSpec, Strings, Ui, H, W};
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tiny_skia::Pixmap;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_BORDER_COLOR,
    DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, GetDeviceCaps, GetMonitorInfoW, InvalidateRect, MonitorFromWindow,
    ReleaseDC, ScreenToClient, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, LOGPIXELSX, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleExW, GetModuleHandleW, GetProcAddress, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::UI::Controls::{MARGINS, WM_MOUSELEAVE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    VK_ESCAPE, VK_RETURN, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, FileOpenDialog, IFileOpenDialog, IShellItem, RemoveWindowSubclass,
    SHCreateItemFromParsingName, SetWindowSubclass, FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST,
    FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
};
use windows::Win32::UI::WindowsAndMessaging::*;

const CLASS: PCWSTR = w!("AepdInstallerUi");
const TIMER_ANIM: usize = 1;
const TIMER_GUARD: usize = 2;
const WM_APP_STATUS: u32 = WM_APP + 1;
const PBM_GETPOS: u32 = 0x0408;
const PBM_GETRANGE: u32 = 0x0407;

struct App {
    parent: HWND,
    overlay: HWND,
    ui: Ui,
    frame: Pixmap,
    bits: Vec<u32>,
    last: Instant,
    animating: bool,
    action: i32,
    product: String,
    vars: Vars,
    /// Title, subtitle and button of the error page.
    error: (String, String, String),
    /// NSIS's `exec_flags`, to see when the install stopped on an error.
    exec_flags: usize,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}
static OVERLAY: AtomicIsize = AtomicIsize::new(0);
static STATUS: Mutex<Option<String>> = Mutex::new(None);

/// Runs `f` on the app unless it is missing or already borrowed (re-entrant message).
fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok().and_then(|mut a| a.as_mut().map(f)))
}

fn module() -> HMODULE {
    let mut m = HMODULE::default();
    unsafe {
        let _ = GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(module as *const () as *const u16),
            &mut m,
        );
    }
    m
}

fn dpi(hwnd: HWND) -> u32 {
    unsafe {
        if let Ok(user32) = GetModuleHandleW(w!("user32.dll")) {
            if let Some(f) = GetProcAddress(user32, windows::core::s!("GetDpiForWindow")) {
                let f: unsafe extern "system" fn(HWND) -> u32 = std::mem::transmute(f);
                let d = f(hwnd);
                if d > 0 {
                    return d;
                }
            }
        }
        let dc = GetDC(Some(hwnd));
        let d = GetDeviceCaps(Some(dc), LOGPIXELSX);
        ReleaseDC(Some(hwnd), dc);
        if d > 0 {
            d as u32
        } else {
            96
        }
    }
}

fn reduced_motion() -> bool {
    let mut on = BOOL(1);
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut on as *mut BOOL as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
    !on.as_bool()
}

fn px_size(scale: f32) -> (i32, i32) {
    ((W * scale).round() as i32, (H * scale).round() as i32)
}

/// Window chrome: dark, rounded on Windows 11 with a subtle border, a shadow everywhere.
/// Returns whether Windows draws the border itself (Windows 11).
fn style_frame(parent: HWND) -> bool {
    unsafe {
        let dark = BOOL(1);
        let _ = DwmSetWindowAttribute(
            parent,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const _ as *const c_void,
            4,
        );
        let round: i32 = 2; // DWMWCP_ROUND
        let win11 = DwmSetWindowAttribute(
            parent,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &round as *const _ as *const c_void,
            4,
        )
        .is_ok();
        if win11 {
            let border: u32 = 0x0024_2322; // #222324 as COLORREF (0x00BBGGRR)
            let _ = DwmSetWindowAttribute(
                parent,
                DWMWA_BORDER_COLOR,
                &border as *const _ as *const c_void,
                4,
            );
        }
        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(parent, &margins);
        win11
    }
}

/// Sizes the window to the design size at its DPI and centres it on its monitor.
fn place(parent: HWND, scale: f32) {
    let (w, h) = px_size(scale);
    unsafe {
        let mon = MonitorFromWindow(parent, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let (x, y) = if GetMonitorInfoW(mon, &mut mi).as_bool() {
            let r = mi.rcWork;
            (
                r.left + (r.right - r.left - w) / 2,
                r.top + (r.bottom - r.top - h) / 2,
            )
        } else {
            (CW_USEDEFAULT, CW_USEDEFAULT)
        };
        let _ = SetWindowPos(
            parent,
            None,
            x,
            y,
            w,
            h,
            SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
        );
    }
}

/// Hides every NSIS control (they keep working) and keeps ours on top.
fn hide_nsis(parent: HWND, overlay: HWND) {
    unsafe extern "system" fn each(child: HWND, lparam: LPARAM) -> BOOL {
        let (parent, overlay) = *(lparam.0 as *const (HWND, HWND));
        if child != overlay && GetParent(child).ok() == Some(parent) {
            let style = GetWindowLongPtrW(child, GWL_STYLE) as isize;
            if style & WS_CLIPSIBLINGS.0 as isize == 0 {
                SetWindowLongPtrW(child, GWL_STYLE, (style | WS_CLIPSIBLINGS.0 as isize) as _);
            }
            if IsWindowVisible(child).as_bool() {
                let _ = ShowWindow(child, SW_HIDE);
            }
        }
        BOOL(1)
    }
    let pair = (parent, overlay);
    unsafe {
        let _ = EnumChildWindows(
            Some(parent),
            Some(each),
            LPARAM(&pair as *const (HWND, HWND) as isize),
        );
        let _ = SetWindowPos(
            overlay,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// NSIS's progress bar on the install page, as a fraction.
fn nsis_progress(parent: HWND) -> Option<f32> {
    unsafe extern "system" fn each(child: HWND, lparam: LPARAM) -> BOOL {
        let mut name = [0u16; 32];
        let n = GetClassNameW(child, &mut name);
        if String::from_utf16_lossy(&name[..n.max(0) as usize])
            .eq_ignore_ascii_case("msctls_progress32")
        {
            *(lparam.0 as *mut HWND) = child;
            return BOOL(0);
        }
        BOOL(1)
    }
    let mut found = HWND::default();
    unsafe {
        let _ = EnumChildWindows(
            Some(parent),
            Some(each),
            LPARAM(&mut found as *mut HWND as isize),
        );
        if found.0.is_null() {
            return None;
        }
        let pos = SendMessageW(found, PBM_GETPOS, None, None).0 as f32;
        let high = SendMessageW(found, PBM_GETRANGE, Some(WPARAM(0)), None).0 as f32;
        (high > 0.0).then(|| (pos / high).clamp(0.0, 1.0))
    }
}

/// Whether the install stopped on an error: NSIS's `exec_flags.abort` (the fourth `int`).
fn nsis_failed(exec_flags: usize) -> bool {
    exec_flags != 0 && unsafe { *(exec_flags as *const i32).add(3) } != 0
}

fn start_animation(app: &mut App) {
    if !app.animating {
        app.animating = true;
        app.last = Instant::now();
        unsafe {
            SetTimer(Some(app.overlay), TIMER_ANIM, 15, None);
        }
    }
}

fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

/// What a click or key does, carried out after the app borrow ends (NSIS may call back in).
enum Effect {
    None,
    Minimize(HWND),
    Next(HWND),
    Cancel(HWND),
    PickFolder,
}

fn activate(app: &mut App, hit: Hit) -> Effect {
    let kind = app.ui.page.kind;
    match hit {
        Hit::Minimize => Effect::Minimize(app.parent),
        Hit::Check => {
            app.ui.check_on = !app.ui.check_on;
            invalidate(app.overlay);
            Effect::None
        }
        Hit::Close => match kind {
            Kind::Progress => Effect::None,
            Kind::Finish => {
                app.action = 0;
                Effect::Next(app.parent)
            }
            Kind::Done => Effect::Next(app.parent),
            _ => Effect::Cancel(app.parent),
        },
        Hit::Primary => match kind {
            Kind::Progress => Effect::None,
            Kind::Error => Effect::Cancel(app.parent),
            _ => {
                app.action = 1;
                Effect::Next(app.parent)
            }
        },
        Hit::Secondary => match kind {
            Kind::Welcome => Effect::PickFolder,
            Kind::Finish => {
                app.action = 0;
                Effect::Next(app.parent)
            }
            _ => Effect::Cancel(app.parent),
        },
    }
}

fn run(effect: Effect) {
    unsafe {
        match effect {
            Effect::None => {}
            Effect::Minimize(p) => {
                let _ = ShowWindow(p, SW_MINIMIZE);
            }
            Effect::Next(p) => {
                let _ = PostMessageW(Some(p), WM_COMMAND, WPARAM(1), LPARAM(0));
                // IDOK: Next
            }
            Effect::Cancel(p) => {
                let _ = PostMessageW(Some(p), WM_COMMAND, WPARAM(2), LPARAM(0));
                // IDCANCEL
            }
            Effect::PickFolder => pick_folder(),
        }
    }
}

/// Folder picker for "Change folder": the product folder is added unless the user picked it.
fn pick_folder() {
    let Some((parent, current, product, vars)) = with_app(|a| {
        (
            a.parent,
            a.vars.get(INST_INSTDIR),
            a.product.clone(),
            a.vars,
        )
    }) else {
        return;
    };
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(dialog) =
            CoCreateInstance::<_, IFileOpenDialog>(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
        else {
            return;
        };
        let _ = dialog.SetOptions(FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
        let start = current
            .rsplit_once('\\')
            .map(|(p, _)| p.to_owned())
            .unwrap_or_default();
        let wide: Vec<u16> = start.encode_utf16().chain(Some(0)).collect();
        if let Ok(item) =
            SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(wide.as_ptr()), None)
        {
            let _ = dialog.SetFolder(&item);
        }
        if dialog.Show(Some(parent)).is_err() {
            return;
        }
        let Ok(item) = dialog.GetResult() else { return };
        let Ok(name) = item.GetDisplayName(SIGDN_FILESYSPATH) else {
            return;
        };
        let mut path = name.to_string().unwrap_or_default();
        CoTaskMemFree(Some(name.0 as *const c_void));
        if path.is_empty() {
            return;
        }
        let last = path
            .trim_end_matches('\\')
            .rsplit('\\')
            .next()
            .unwrap_or("");
        if !last.eq_ignore_ascii_case(&product) {
            path = format!("{}\\{}", path.trim_end_matches('\\'), product);
        }
        vars.set(INST_INSTDIR, &path);
        with_app(|a| {
            a.ui.set_link(path.clone());
            invalidate(a.overlay);
        });
    }
}

fn client_dips(app: &App, x: i32, y: i32) -> (f32, f32) {
    (x as f32 / app.ui.scale, y as f32 / app.ui.scale)
}

fn lparam_point(l: LPARAM) -> (i32, i32) {
    (
        (l.0 & 0xffff) as i16 as i32,
        ((l.0 >> 16) & 0xffff) as i16 as i32,
    )
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            let (sx, sy) = lparam_point(lparam);
            let mut pt = POINT { x: sx, y: sy };
            let _ = ScreenToClient(hwnd, &mut pt);
            let over = with_app(|a| {
                let (x, y) = client_dips(a, pt.x, pt.y);
                a.ui.hit(x, y).is_some()
            });
            if over.unwrap_or(false) {
                LRESULT(HTCLIENT as isize)
            } else {
                // Let the parent answer: its whole client area is the caption.
                with_app(|a| {
                    if a.ui.hover.take().is_some() {
                        start_animation(a);
                    }
                });
                LRESULT(HTTRANSPARENT as isize)
            }
        }
        WM_SETCURSOR => {
            SetCursor(LoadCursorW(None, IDC_ARROW).ok());
            LRESULT(1)
        }
        WM_MOUSEMOVE => {
            let (x, y) = lparam_point(lparam);
            with_app(|a| {
                let (dx, dy) = client_dips(a, x, y);
                let hit = a.ui.hit(dx, dy);
                if hit != a.ui.hover {
                    a.ui.hover = hit;
                    start_animation(a);
                }
            });
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_app(|a| {
                if a.ui.hover.take().is_some() {
                    start_animation(a);
                }
            });
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let (x, y) = lparam_point(lparam);
            with_app(|a| {
                let (dx, dy) = client_dips(a, x, y);
                a.ui.pressed = a.ui.hit(dx, dy);
                a.ui.focus_visible = false;
                invalidate(hwnd);
            });
            SetCapture(hwnd);
            let _ = SetFocus(Some(hwnd));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = ReleaseCapture();
            let (x, y) = lparam_point(lparam);
            let effect = with_app(|a| {
                let (dx, dy) = client_dips(a, x, y);
                let hit = a.ui.hit(dx, dy);
                let pressed = a.ui.pressed.take();
                invalidate(hwnd);
                match (hit, pressed) {
                    (Some(h), Some(p)) if h == p => activate(a, h),
                    _ => Effect::None,
                }
            });
            run(effect.unwrap_or(Effect::None));
            LRESULT(0)
        }
        WM_GETDLGCODE => LRESULT(DLGC_WANTALLKEYS as isize),
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            let effect = with_app(|a| {
                if key == VK_TAB.0 {
                    let order = a.ui.focusable();
                    if order.is_empty() {
                        return Effect::None;
                    }
                    let back = GetKeyState(VK_SHIFT.0 as i32) < 0;
                    let i = a.ui.focus.and_then(|f| order.iter().position(|h| *h == f));
                    let n = order.len();
                    let next = match (i, back) {
                        (None, false) => 0,
                        (None, true) => n - 1,
                        (Some(i), false) => (i + 1) % n,
                        (Some(i), true) => (i + n - 1) % n,
                    };
                    a.ui.focus = Some(order[next]);
                    a.ui.focus_visible = true;
                    invalidate(hwnd);
                    Effect::None
                } else if key == VK_RETURN.0 {
                    // Enter presses the focused button, else the main one (as in dialogs).
                    let target = match a.ui.focus {
                        Some(Hit::Secondary) if a.ui.focus_visible => Hit::Secondary,
                        _ => Hit::Primary,
                    };
                    if a.ui.focusable().contains(&target) {
                        activate(a, target)
                    } else {
                        Effect::None
                    }
                } else if key == VK_SPACE.0 {
                    match a.ui.focus {
                        Some(h) if a.ui.focusable().contains(&h) => activate(a, h),
                        _ => Effect::None,
                    }
                } else if key == VK_ESCAPE.0 && a.ui.close_enabled() {
                    activate(a, Hit::Close)
                } else {
                    Effect::None
                }
            });
            run(effect.unwrap_or(Effect::None));
            LRESULT(0)
        }
        WM_SETFOCUS | WM_KILLFOCUS => {
            invalidate(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            let id = wparam.0;
            with_app(|a| {
                if id == TIMER_GUARD {
                    hide_nsis(a.parent, a.overlay);
                    if a.ui.page.kind == Kind::Progress && nsis_failed(a.exec_flags) {
                        // NSIS stopped on an error and waits for Cancel (now "Close").
                        let (title, sub, close) = a.error.clone();
                        a.ui.set_page(PageSpec {
                            kind: Kind::Error,
                            title,
                            sub,
                            primary: close,
                            ..PageSpec::default()
                        });
                        start_animation(a);
                    } else if a.ui.page.kind == Kind::Progress {
                        if let Some(p) = nsis_progress(a.parent) {
                            if (p - a.ui.progress).abs() > 1e-4 {
                                a.ui.progress = p;
                                start_animation(a);
                            }
                        }
                    }
                } else if id == TIMER_ANIM {
                    let now = Instant::now();
                    let dt = now.duration_since(a.last).as_secs_f32().min(0.05);
                    a.last = now;
                    let moving = a.ui.tick(dt);
                    invalidate(hwnd);
                    if !moving {
                        a.animating = false;
                        let _ = KillTimer(Some(hwnd), TIMER_ANIM);
                    }
                }
            });
            LRESULT(0)
        }
        WM_APP_STATUS => {
            if let Some(s) = STATUS.lock().ok().and_then(|mut s| s.take()) {
                with_app(|a| {
                    a.ui.status = s;
                    invalidate(hwnd);
                });
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            with_app(|a| {
                let (w, h) = a.ui.pixel_size();
                if a.frame.width() != w || a.frame.height() != h {
                    a.frame = Pixmap::new(w, h).unwrap();
                }
                a.ui.render(&mut a.frame);
                a.bits.resize((w * h) as usize, 0);
                for (dst, src) in a.bits.iter_mut().zip(a.frame.data().as_chunks::<4>().0) {
                    *dst =
                        0xff00_0000 | (src[0] as u32) << 16 | (src[1] as u32) << 8 | src[2] as u32;
                }
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: w as i32,
                        biHeight: -(h as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                SetDIBitsToDevice(
                    hdc,
                    0,
                    0,
                    w,
                    h,
                    0,
                    0,
                    0,
                    h,
                    a.bits.as_ptr() as *const c_void,
                    &bmi,
                    DIB_RGB_COLORS,
                );
            });
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn parent_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_NCCALCSIZE if wparam.0 != 0 => LRESULT(0),
        WM_NCHITTEST => {
            let r = DefSubclassProc(hwnd, msg, wparam, lparam);
            if r.0 == HTCLIENT as isize {
                LRESULT(HTCAPTION as isize)
            } else {
                r
            }
        }
        WM_NCACTIVATE => DefSubclassProc(hwnd, msg, wparam, LPARAM(-1)),
        WM_SIZE => {
            let overlay = HWND(OVERLAY.load(Ordering::Relaxed) as *mut c_void);
            if !overlay.0.is_null() {
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let _ = MoveWindow(overlay, 0, 0, r.right - r.left, r.bottom - r.top, true);
            }
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
        WM_DPICHANGED => {
            let new_dpi = ((wparam.0 >> 16) & 0xffff) as u32;
            let scale = new_dpi.max(96) as f32 / 96.0;
            let suggested = *(lparam.0 as *const RECT);
            with_app(|a| {
                a.ui.set_scale(scale);
            });
            let (w, h) = px_size(scale);
            let _ = SetWindowPos(
                hwnd,
                None,
                suggested.left,
                suggested.top,
                w,
                h,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            LRESULT(0)
        }
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "C" fn plugin_callback(msg: i32) -> usize {
    if msg == NSPIM_GUIUNLOAD {
        let app = APP.with(|a| a.try_borrow_mut().ok().and_then(|mut a| a.take()));
        if let Some(app) = app {
            let _ = KillTimer(Some(app.overlay), TIMER_ANIM);
            let _ = KillTimer(Some(app.overlay), TIMER_GUARD);
            let _ = RemoveWindowSubclass(app.parent, Some(parent_proc), 1);
            let _ = DestroyWindow(app.overlay);
            OVERLAY.store(0, Ordering::Relaxed);
        }
    }
    0
}

fn register_class(hinst: HINSTANCE) {
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(overlay_proc),
            hInstance: hinst,
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassExW(&wc);
    }
}

/// `aepd_ui::Init product version footer_lead footer_sub error_title error_sub close`: takes
/// over the installer window.
///
/// # Safety
/// Called by NSIS with its plug-in arguments.
#[no_mangle]
pub unsafe extern "C" fn Init(
    parent: HWND,
    string_size: i32,
    variables: *mut u16,
    stacktop: *mut *mut StackT,
    extra: *mut ExtraParameters,
) {
    let call = Call::new(string_size, variables, stacktop);
    let strings = Strings {
        product: call.pop(),
        version: call.pop(),
        footer_lead: call.pop(),
        footer_sub: call.pop(),
    };
    let error = (call.pop(), call.pop(), call.pop());
    if !extra.is_null() {
        if let Some(register) = (*extra).register_plugin_callback {
            register(module().0, plugin_callback);
        }
    }
    if parent.0.is_null() || APP.with(|a| a.borrow().is_some()) {
        return;
    }
    let scale = dpi(parent) as f32 / 96.0;
    let win11 = style_frame(parent);
    let product = strings.product.clone();
    let mut ui = Ui::new(scale, strings, !win11);
    ui.reduced_motion = reduced_motion();
    let hinst = HINSTANCE(module().0);
    register_class(hinst);
    let (w, h) = px_size(scale);
    let Ok(overlay) = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        CLASS,
        w!(""),
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
        0,
        0,
        w,
        h,
        Some(parent),
        None,
        Some(hinst),
        None,
    ) else {
        return;
    };
    OVERLAY.store(overlay.0 as isize, Ordering::Relaxed);
    let style = GetWindowLongPtrW(parent, GWL_STYLE) as isize;
    SetWindowLongPtrW(
        parent,
        GWL_STYLE,
        ((style | WS_CLIPCHILDREN.0 as isize)
            & !(WS_THICKFRAME.0 as isize | WS_MAXIMIZEBOX.0 as isize)) as _,
    );
    let _ = SetWindowSubclass(parent, Some(parent_proc), 1, 0);
    let frame = Pixmap::new(w as u32, h as u32).unwrap();
    APP.with(|a| {
        *a.borrow_mut() = Some(App {
            parent,
            overlay,
            ui,
            frame,
            bits: Vec::new(),
            last: Instant::now(),
            animating: false,
            action: 1,
            product,
            vars: call.vars(),
            error,
            exec_flags: if extra.is_null() {
                0
            } else {
                (*extra).exec_flags as usize
            },
        })
    });
    place(parent, scale);
    hide_nsis(parent, overlay);
    SetTimer(Some(overlay), TIMER_GUARD, 50, None);
}

/// `aepd_ui::Page kind title sub primary secondary check check_on`: shows a page.
///
/// # Safety
/// Called by NSIS with its plug-in arguments.
#[no_mangle]
pub unsafe extern "C" fn Page(
    _parent: HWND,
    string_size: i32,
    variables: *mut u16,
    stacktop: *mut *mut StackT,
    _extra: *mut ExtraParameters,
) {
    let call = Call::new(string_size, variables, stacktop);
    let spec = PageSpec {
        kind: Kind::parse(&call.pop()),
        title: call.pop(),
        sub: call.pop(),
        primary: call.pop(),
        secondary: call.pop(),
        check: call.pop(),
        check_on: call.pop() == "1",
    };
    let target = with_app(|a| {
        a.ui.set_page(spec);
        a.action = 1;
        hide_nsis(a.parent, a.overlay);
        start_animation(a);
        invalidate(a.overlay);
        a.overlay
    });
    if let Some(overlay) = target {
        let _ = SetFocus(Some(overlay));
    }
}

/// `aepd_ui::Status text`: the progress line; may be called from the install thread.
///
/// # Safety
/// Called by NSIS with its plug-in arguments.
#[no_mangle]
pub unsafe extern "C" fn Status(
    _parent: HWND,
    string_size: i32,
    variables: *mut u16,
    stacktop: *mut *mut StackT,
    _extra: *mut ExtraParameters,
) {
    let call = Call::new(string_size, variables, stacktop);
    let text = call.pop();
    if let Ok(mut s) = STATUS.lock() {
        *s = Some(text);
    }
    let overlay = OVERLAY.load(Ordering::Relaxed);
    if overlay != 0 {
        let _ = PostMessageW(
            Some(HWND(overlay as *mut c_void)),
            WM_APP_STATUS,
            WPARAM(0),
            LPARAM(0),
        );
    }
}

/// `aepd_ui::Result`: pushes the checkbox state, then the action ("1" = primary button), so
/// the script pops the action first.
///
/// # Safety
/// Called by NSIS with its plug-in arguments.
#[no_mangle]
pub unsafe extern "C" fn Result(
    _parent: HWND,
    string_size: i32,
    variables: *mut u16,
    stacktop: *mut *mut StackT,
    _extra: *mut ExtraParameters,
) {
    let call = Call::new(string_size, variables, stacktop);
    let (check, action) = with_app(|a| (a.ui.check_on, a.action)).unwrap_or((false, 1));
    call.push(if check { "1" } else { "0" });
    call.push(if action == 1 { "1" } else { "0" });
}
