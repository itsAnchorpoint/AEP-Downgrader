//! The installer window as a scene: a static background (light, grain, dots, the Fold art, the
//! header and the footer) and the current page on top, with hover, press, focus and an entrance
//! animation. Platform independent; `win.rs` feeds it input and puts the pixels on screen.
//!
//! Sizes follow the approved mock-up (.work/design/installer/mockup.html): a 640 x 420 window
//! in DIPs, content from x = 28, actions on one line 92 DIPs above the bottom edge.

use crate::paint::{
    self, box_shadow, draw_image, hex, inset_outline, mix, rgba, round_rect, solid, vgradient,
    Icon, R,
};
use crate::text::{fonts, shape, wrap, Font, Piece, Run, Style};
use std::sync::OnceLock;
use tiny_skia::{Color, Pixmap, Transform};

pub const W: f32 = 640.0;
pub const H: f32 = 420.0;
const X0: f32 = 28.0;
const TITLE_TOP: f32 = 74.0;
const END_BOTTOM: f32 = H - 92.0;
const SUB_WIDTH: f32 = 292.0;

const FG: u32 = 0xf3efe6;
const MUTED: u32 = 0x8d8a84;
const INK: u32 = 0x0b0b0c;
const LIME: u32 = 0xd6ff3d;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Kind {
    #[default]
    Welcome,
    Progress,
    Finish,
    Confirm,
    Done,
    Error,
}

impl Kind {
    pub fn parse(s: &str) -> Kind {
        match s {
            "progress" => Kind::Progress,
            "finish" => Kind::Finish,
            "confirm" => Kind::Confirm,
            "done" => Kind::Done,
            "error" => Kind::Error,
            _ => Kind::Welcome,
        }
    }
}

/// One page as the installer script describes it. `title` marks the highlighted words with
/// `[...]`, `sub` marks bold words with `**...**`. On a progress page `primary` is the first
/// status line.
#[derive(Clone, Default, Debug)]
pub struct PageSpec {
    pub kind: Kind,
    pub title: String,
    pub sub: String,
    pub primary: String,
    pub secondary: String,
    pub check: String,
    pub check_on: bool,
}

/// Texts that stay for the life of the window.
#[derive(Clone, Default, Debug)]
pub struct Strings {
    pub product: String,
    pub version: String,
    pub footer_lead: String,
    pub footer_sub: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hit {
    Minimize,
    Close,
    Primary,
    Secondary,
    Check,
}

impl Hit {
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonStyle {
    Lime,
    Strong,
    Ghost,
}

struct Button {
    rect: R,
    style: ButtonStyle,
    label: Run,
    icon: Option<Icon>,
}

struct Check {
    rect: R,
    label: Run,
    baseline: f32,
}

#[derive(Default)]
struct Layout {
    title: Vec<(Run, f32)>,
    title_baseline: f32,
    plate: Option<R>,
    sub: Vec<Vec<Piece>>,
    sub_baselines: Vec<f32>,
    primary: Option<Button>,
    secondary: Option<Button>,
    check: Option<Check>,
    track: Option<R>,
    status_baseline: f32,
}

const MINIMIZE: R = R::new(W - 92.0, 0.0, 46.0, 36.0);
const CLOSE: R = R::new(W - 46.0, 0.0, 46.0, 36.0);

fn style(font: Font, size: f32, tracking: f32, color: u32) -> Style {
    Style {
        font,
        size,
        tracking,
        color: hex(color),
    }
}

/// Baseline that centres the em box of `font` at `size` on `center`.
fn centred_baseline(font: Font, size: f32, center: f32) -> f32 {
    let (a, d) = fonts().metrics(font, size);
    center + (a - d) / 2.0
}

/// Baseline of a CSS line box of `line_height` starting at `top`.
fn line_baseline(font: Font, size: f32, line_height: f32, top: f32) -> f32 {
    let (a, d) = fonts().metrics(font, size);
    top + (line_height - (a + d)) / 2.0 + a
}

fn art() -> &'static (Pixmap, Pixmap) {
    static ART: OnceLock<(Pixmap, Pixmap)> = OnceLock::new();
    ART.get_or_init(|| {
        (
            Pixmap::decode_png(include_bytes!("../../assets/fold.png")).expect("fold.png"),
            Pixmap::decode_png(include_bytes!("../../assets/mark.png")).expect("mark.png"),
        )
    })
}

/// Everything that does not change while the window is open, at scale `s`.
pub fn background(s: f32, strings: &Strings, own_border: bool) -> Pixmap {
    let w = (W * s).round() as u32;
    let h = (H * s).round() as u32;
    let mut px = Pixmap::new(w, h).unwrap();
    let lime = [214.0 / 255.0, 1.0, 61.0 / 255.0];
    let ell = |x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32| {
        (((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)).sqrt()
    };
    let mut seed: u32 = 0x9e37_79b9;
    let data = px.data_mut();
    for py in 0..h {
        for pxl in 0..w {
            let x = (pxl as f32 + 0.5) / s;
            let y = (py as f32 + 0.5) / s;
            let mut c = [11.0 / 255.0, 11.0 / 255.0, 12.0 / 255.0];
            let over = |c: &mut [f32; 3], col: [f32; 3], a: f32| {
                for i in 0..3 {
                    c[i] = c[i] * (1.0 - a) + col[i] * a;
                }
            };
            // Light from the top left, the glow behind the art, a faint glow from below.
            over(
                &mut c,
                [1.0; 3],
                0.075 * (1.0 - ell(x, y, 89.6, -58.8, 520.0, 300.0) / 0.70).max(0.0),
            );
            over(
                &mut c,
                lime,
                0.15 * (1.0 - ell(x, y, 484.0, 204.0, 230.0, 180.0) / 0.72).max(0.0),
            );
            over(
                &mut c,
                lime,
                0.05 * (1.0 - ell(x, y, 320.0, 504.0, 320.0, 140.0) / 0.70).max(0.0),
            );
            // Film grain (CSS: fractal noise, overlay blend at 9 %): +-4.5 % luminance.
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let n = (seed >> 8) as f32 / 16_777_216.0;
            let g = 1.0 + 0.045 * (2.0 * n - 1.0);
            for v in c.iter_mut() {
                *v *= g;
            }
            // Dot grid, 16 DIP pitch, faded out from the art.
            let gx = x - 1.0 - ((x - 1.0) / 16.0).round() * 16.0;
            let gy = y - 1.0 - ((y - 1.0) / 16.0).round() * 16.0;
            let dist = (gx * gx + gy * gy).sqrt();
            let cover = ((0.9 - dist) * s + 0.5).clamp(0.0, 1.0);
            if cover > 0.0 {
                let mask = 0.9 * (1.0 - ell(x, y, 473.6, 243.6, 332.8, 218.4)).max(0.0);
                over(&mut c, [1.0; 3], 0.07 * cover * mask);
            }
            // Vignette.
            let v = ell(x, y, 320.0, 168.0, 832.0, 420.0);
            let va = 0.55 * ((v - 0.55) / 0.45).clamp(0.0, 1.0);
            for ch in c.iter_mut() {
                *ch *= 1.0 - va;
            }
            let o = ((py * w + pxl) * 4) as usize;
            data[o] = (c[0] * 255.0).round().clamp(0.0, 255.0) as u8;
            data[o + 1] = (c[1] * 255.0).round().clamp(0.0, 255.0) as u8;
            data[o + 2] = (c[2] * 255.0).round().clamp(0.0, 255.0) as u8;
            data[o + 3] = 255;
        }
    }

    let (fold, mark) = art();
    let fold = paint::resize(fold, (368.0 * s).round() as u32, (274.0 * s).round() as u32);
    draw_image(
        &mut px,
        &fold,
        (300.0 * s).round() as i32,
        (66.0 * s).round() as i32,
        1.0,
    );
    let mark = paint::resize(mark, (22.0 * s).round() as u32, (22.0 * s).round() as u32);
    draw_image(
        &mut px,
        &mark,
        (24.0 * s).round() as i32,
        (18.0 * s).round() as i32,
        1.0,
    );

    // Header: product name and the version chip, centred on y = 29.
    let name = shape(&strings.product, style(Font::Onest800, 14.0, -0.01, FG));
    name.draw(
        &mut px,
        55.0,
        centred_baseline(Font::Onest800, 14.0, 29.0),
        s,
        None,
        Transform::identity(),
    );
    if !strings.version.is_empty() {
        let ver = shape(&strings.version, style(Font::Mono700, 10.5, 0.0, INK));
        let (a, d) = fonts().metrics(Font::Mono700, 10.5);
        let chip = R::new(55.0 + name.width + 11.0, 0.0, ver.width + 12.0, a + d + 2.0);
        let chip = R {
            y: 29.0 - chip.h / 2.0,
            ..chip
        };
        if let Some(p) = round_rect(chip, 5.0, s) {
            paint::fill(&mut px, &p, &solid(hex(LIME)), Transform::identity());
        }
        ver.draw(
            &mut px,
            chip.x + 6.0,
            chip.y + 1.0 + a,
            s,
            None,
            Transform::identity(),
        );
    }

    // Footer: a hairline, the lock and the privacy line.
    if let Some(p) = round_rect(R::new(24.0, 356.0, W - 48.0, 1.0), 0.0, s) {
        paint::fill(
            &mut px,
            &p,
            &solid(rgba(255, 255, 255, 0.07)),
            Transform::identity(),
        );
    }
    paint::icon(
        &mut px,
        Icon::Lock,
        24.0,
        372.0,
        17.0,
        2.4,
        hex(LIME),
        s,
        Transform::identity(),
    );
    let base = centred_baseline(Font::Onest800, 13.0, 380.5);
    let lead = shape(&strings.footer_lead, style(Font::Onest800, 13.0, -0.01, FG));
    lead.draw(&mut px, 50.0, base, s, None, Transform::identity());
    let sub = shape(
        &strings.footer_sub,
        style(Font::Onest700, 13.0, -0.01, MUTED),
    );
    sub.draw(
        &mut px,
        59.0 + lead.width,
        base,
        s,
        None,
        Transform::identity(),
    );

    if own_border {
        inset_outline(
            &mut px,
            R::new(0.0, 0.0, W, H),
            0.0,
            1.0 / s,
            rgba(255, 255, 255, 0.09),
            s,
            Transform::identity(),
        );
    }
    px
}

/// Splits `[...]` (title highlight) or `**...**` (bold) markup into (text, marked) pieces.
fn markup(text: &str, open: &str, close: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find(open) {
        if i > 0 {
            out.push((rest[..i].to_owned(), false));
        }
        let after = &rest[i + open.len()..];
        match after.find(close) {
            Some(j) => {
                out.push((after[..j].to_owned(), true));
                rest = &after[j + close.len()..];
            }
            None => {
                out.push((after.to_owned(), true));
                rest = "";
            }
        }
    }
    if !rest.is_empty() {
        out.push((rest.to_owned(), false));
    }
    out
}

fn button(label: &str, style_: ButtonStyle, x: f32, y: f32, icon: Option<Icon>) -> Button {
    let color = if style_ == ButtonStyle::Ghost {
        FG
    } else {
        INK
    };
    let label = shape(label, style(Font::Onest800, 15.0, -0.01, color));
    let w = 52.0 + label.width + if icon.is_some() { 26.0 } else { 0.0 };
    Button {
        rect: R::new(x, y, w, 44.0),
        style: style_,
        label,
        icon,
    }
}

fn layout(spec: &PageSpec, link_label: &str) -> Layout {
    let mut l = Layout::default();

    // Title: one line at 38 px, shrunk if it would not fit; the highlight is a lime plate with
    // 10 DIPs of padding and 2 DIPs of margin, 44.8 DIPs tall (CSS line-height 1.1 + 3 px).
    let pieces = markup(&spec.title, "[", "]");
    let measure = |size: f32| -> f32 {
        let mut x = 0.0;
        for (t, plate) in &pieces {
            if *plate {
                x += 22.0;
            }
            x += shape(t, style(Font::Onest800, size, -0.04, FG)).width;
        }
        x
    };
    let mut size = 38.0f32;
    let natural = measure(size);
    let max = W - 2.0 * X0;
    if natural > max {
        size = (size * max / natural).max(26.0);
    }
    let (a, d) = fonts().metrics(Font::Onest800, size);
    let has_plate = pieces.iter().any(|(_, p)| *p);
    let strut = size * 1.08;
    let plate_h = size * 1.1;
    let (title_h, baseline) = if has_plate {
        let above = (plate_h - (a + d)) / 2.0 + a;
        (plate_h + 3.0, TITLE_TOP + above)
    } else {
        (strut, TITLE_TOP + (strut - (a + d)) / 2.0 + a)
    };
    l.title_baseline = baseline;
    let mut x = X0;
    for (t, plate) in &pieces {
        if *plate {
            let run = shape(t, style(Font::Onest800, size, -0.04, INK));
            l.plate = Some(R::new(x + 2.0, TITLE_TOP, run.width + 20.0, plate_h + 3.0));
            l.title.push((run.clone(), x + 12.0));
            x += 22.0 + run.width;
        } else {
            let run = shape(t, style(Font::Onest800, size, -0.04, FG));
            l.title.push((run.clone(), x));
            x += run.width;
        }
    }

    // Subtitle: 16 DIPs below, 14 px, line height 1.45, bold words in the text colour.
    let spans: Vec<(String, Style)> = markup(&spec.sub, "**", "**")
        .into_iter()
        .map(|(t, bold)| {
            let st = if bold {
                style(Font::Onest700, 14.0, -0.005, FG)
            } else {
                style(Font::Onest600, 14.0, -0.005, MUTED)
            };
            (t, st)
        })
        .collect();
    let sub_top = TITLE_TOP + title_h + 16.0;
    let lh = 14.0 * 1.45;
    l.sub = if spec.sub.is_empty() {
        Vec::new()
    } else {
        wrap(&spans, SUB_WIDTH)
    };
    l.sub_baselines = (0..l.sub.len())
        .map(|i| line_baseline(Font::Onest600, 14.0, lh, sub_top + i as f32 * lh))
        .collect();

    // The end block sits on the bottom line.
    let actions_y = END_BOTTOM - 44.0;
    let check_y = actions_y - 40.0;
    match spec.kind {
        Kind::Welcome => {
            let b = button(
                &spec.primary,
                ButtonStyle::Lime,
                X0,
                actions_y,
                Some(Icon::Download),
            );
            let sx = b.rect.right() + 14.0;
            l.primary = Some(b);
            // "Change folder", or the chosen folder once picked: a quiet button, not a bare link,
            // so it reads as an option next to the one loud action.
            if !link_label.is_empty() {
                l.secondary = Some(button(
                    link_label,
                    ButtonStyle::Ghost,
                    sx,
                    actions_y,
                    Some(Icon::Folder),
                ));
            }
        }
        Kind::Progress => {
            l.track = Some(R::new(X0, END_BOTTOM - 16.0, SUB_WIDTH, 8.0));
            let (_, d13) = fonts().metrics(Font::Onest700, 13.0);
            l.status_baseline = END_BOTTOM - 26.0 - d13;
        }
        Kind::Finish | Kind::Confirm | Kind::Done | Kind::Error => {
            let primary_style = if spec.kind == Kind::Confirm || spec.kind == Kind::Error {
                ButtonStyle::Strong
            } else {
                ButtonStyle::Lime
            };
            let b = button(&spec.primary, primary_style, X0, actions_y, None);
            let sx = b.rect.right() + 14.0;
            l.primary = Some(b);
            if !spec.secondary.is_empty() {
                l.secondary = Some(button(
                    &spec.secondary,
                    ButtonStyle::Ghost,
                    sx,
                    actions_y,
                    None,
                ));
            }
            if !spec.check.is_empty() {
                l.check = Some(Check {
                    rect: R::new(X0, check_y, 20.0, 20.0),
                    label: shape(&spec.check, style(Font::Onest700, 14.0, 0.0, FG)),
                    baseline: centred_baseline(Font::Onest700, 14.0, check_y + 10.0),
                });
            }
        }
    }
    l
}

/// Shortens a path for the folder button: keeps the drive and the last folders.
pub fn shorten_path(path: &str, max_width: f32) -> String {
    let st = style(Font::Onest800, 15.0, -0.01, FG);
    if shape(path, st).width <= max_width {
        return path.to_owned();
    }
    let parts: Vec<&str> = path.split('\\').collect();
    for keep in (1..parts.len().saturating_sub(1)).rev() {
        let candidate = format!(
            "{}\\…\\{}",
            parts[0],
            parts[parts.len() - keep..].join("\\")
        );
        if shape(&candidate, st).width <= max_width {
            return candidate;
        }
    }
    format!("…\\{}", parts.last().unwrap_or(&""))
}

fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(5)
}

pub struct Ui {
    pub scale: f32,
    strings: Strings,
    own_border: bool,
    bg: Pixmap,
    layer: Pixmap,
    pub page: PageSpec,
    layout: Layout,
    link_label: String,
    pub check_on: bool,
    pub hover: Option<Hit>,
    amount: [f32; 5],
    pub pressed: Option<Hit>,
    pub focus: Option<Hit>,
    pub focus_visible: bool,
    enter: f32,
    pub progress: f32,
    shown: f32,
    pub status: String,
    pub reduced_motion: bool,
}

impl Ui {
    pub fn new(scale: f32, strings: Strings, own_border: bool) -> Ui {
        let bg = background(scale, &strings, own_border);
        let layer = Pixmap::new(bg.width(), bg.height()).unwrap();
        Ui {
            scale,
            strings,
            own_border,
            bg,
            layer,
            page: PageSpec::default(),
            layout: Layout::default(),
            link_label: String::new(),
            check_on: false,
            hover: None,
            amount: [0.0; 5],
            pressed: None,
            focus: None,
            focus_visible: false,
            enter: 1.0,
            progress: 0.0,
            shown: 0.0,
            status: String::new(),
            reduced_motion: false,
        }
    }

    pub fn pixel_size(&self) -> (u32, u32) {
        (self.bg.width(), self.bg.height())
    }

    pub fn set_scale(&mut self, scale: f32) {
        if (scale - self.scale).abs() < 1e-3 {
            return;
        }
        self.scale = scale;
        self.bg = background(scale, &self.strings, self.own_border);
        self.layer = Pixmap::new(self.bg.width(), self.bg.height()).unwrap();
    }

    pub fn set_page(&mut self, spec: PageSpec) {
        self.check_on = spec.check_on;
        self.link_label = spec.secondary.clone();
        if spec.kind == Kind::Progress {
            self.status = spec.primary.clone();
            self.progress = 0.0;
            self.shown = 0.0;
        }
        self.layout = layout(&spec, &self.link_label);
        self.page = spec;
        self.hover = None;
        self.pressed = None;
        self.focus = if self.layout.primary.is_some() {
            Some(Hit::Primary)
        } else {
            None
        };
        self.enter = if self.reduced_motion { 1.0 } else { 0.0 };
    }

    /// Replaces the folder button's text (after the user picked a folder).
    pub fn set_link(&mut self, text: String) {
        self.link_label = shorten_path(&text, 260.0);
        self.layout = layout(&self.page, &self.link_label);
    }

    pub fn close_enabled(&self) -> bool {
        self.page.kind != Kind::Progress
    }

    fn rect(&self, hit: Hit) -> Option<R> {
        let l = &self.layout;
        match hit {
            Hit::Minimize => Some(MINIMIZE),
            Hit::Close => Some(CLOSE),
            Hit::Primary => l.primary.as_ref().map(|b| b.rect),
            Hit::Secondary => l.secondary.as_ref().map(|b| b.rect),
            Hit::Check => l
                .check
                .as_ref()
                .map(|c| R::new(c.rect.x - 4.0, c.rect.y - 4.0, c.label.width + 38.0, 28.0)),
        }
    }

    /// The interactive element at a point in DIPs.
    pub fn hit(&self, x: f32, y: f32) -> Option<Hit> {
        [
            Hit::Minimize,
            Hit::Close,
            Hit::Primary,
            Hit::Secondary,
            Hit::Check,
        ]
        .into_iter()
        .filter(|h| *h != Hit::Close || self.close_enabled())
        .find(|h| self.rect(*h).is_some_and(|r| r.contains(x, y)))
    }

    /// Keyboard focus order.
    pub fn focusable(&self) -> Vec<Hit> {
        [Hit::Check, Hit::Primary, Hit::Secondary]
            .into_iter()
            .filter(|h| self.rect(*h).is_some())
            .collect()
    }

    /// Advances animations by `dt` seconds; returns whether anything still moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        if self.enter < 1.0 {
            self.enter = (self.enter + dt / 0.32).min(1.0);
            moving = true;
        }
        let rate = if self.reduced_motion {
            1.0
        } else {
            (dt / 0.15).min(1.0)
        };
        for (i, h) in [
            Hit::Minimize,
            Hit::Close,
            Hit::Primary,
            Hit::Secondary,
            Hit::Check,
        ]
        .iter()
        .enumerate()
        {
            let target = if self.hover == Some(*h) { 1.0 } else { 0.0 };
            let a = &mut self.amount[i];
            let step = (target - *a).signum() * rate;
            if (target - *a).abs() <= rate {
                *a = target;
            } else {
                *a += step;
                moving = true;
            }
        }
        if self.page.kind == Kind::Progress {
            let diff = self.progress - self.shown;
            if diff.abs() > 0.001 {
                let k = if self.reduced_motion {
                    1.0
                } else {
                    1.0 - (-dt / 0.15).exp()
                };
                self.shown += diff * k + diff.signum() * (dt * 0.2).min(diff.abs());
                if (self.progress - self.shown) * diff <= 0.0 {
                    self.shown = self.progress;
                }
                moving = true;
            }
        }
        moving
    }

    pub fn render(&mut self, out: &mut Pixmap) {
        out.data_mut().copy_from_slice(self.bg.data());
        let e = ease_out_quint(self.enter);
        if e >= 1.0 {
            self.draw_page(out, 0.0);
        } else {
            let mut layer = std::mem::replace(&mut self.layer, Pixmap::new(1, 1).unwrap());
            layer.fill(Color::TRANSPARENT);
            self.draw_page(&mut layer, (1.0 - e) * 10.0);
            draw_image(out, &layer, 0, 0, e);
            self.layer = layer;
        }
        self.draw_controls(out);
    }

    fn amount(&self, h: Hit) -> f32 {
        self.amount[h.index()]
    }

    fn focus_ring(&self, px: &mut Pixmap, h: Hit, rect: R, radius: f32, t: Transform) {
        if self.focus_visible && self.focus == Some(h) {
            inset_outline(
                px,
                rect.inset(-4.0),
                radius + 4.0,
                2.0,
                hex(LIME),
                self.scale,
                t,
            );
        }
    }

    fn draw_page(&self, px: &mut Pixmap, oy: f32) {
        let s = self.scale;
        let l = &self.layout;
        let t0 = Transform::from_translate(0.0, oy * s);

        if let Some(plate) = l.plate {
            box_shadow(
                px,
                R {
                    y: plate.y + oy,
                    ..plate
                },
                0.0,
                8.0,
                24.0,
                -8.0,
                rgba(214, 255, 61, 0.55),
                s,
                1.0,
            );
            if let Some(p) = round_rect(plate, 11.0, s) {
                let paint = vgradient(
                    R {
                        y: plate.y + oy,
                        ..plate
                    },
                    s,
                    hex(0xe6ff78),
                    hex(0xd2fb36),
                    Some(rgba(255, 255, 255, 0.55)),
                );
                paint::fill(px, &p, &paint, t0);
            }
        }
        for (run, x) in &l.title {
            run.draw(
                px,
                *x,
                l.title_baseline + oy,
                s,
                None,
                Transform::identity(),
            );
        }
        for (line, base) in l.sub.iter().zip(&l.sub_baselines) {
            for piece in line {
                piece
                    .run
                    .draw(px, X0 + piece.x, base + oy, s, None, Transform::identity());
            }
        }

        if let Some(c) = &l.check {
            let h = self.amount(Hit::Check);
            let r = R {
                y: c.rect.y + oy,
                ..c.rect
            };
            if self.check_on {
                if let Some(p) = round_rect(r, 6.0, s) {
                    paint::fill(
                        px,
                        &p,
                        &vgradient(
                            r,
                            s,
                            hex(0xe6ff78),
                            hex(0xd2fb36),
                            Some(rgba(255, 255, 255, 0.55)),
                        ),
                        Transform::identity(),
                    );
                }
                paint::icon(
                    px,
                    Icon::Check,
                    r.x + 3.5,
                    r.y + 3.5,
                    13.0,
                    3.6,
                    hex(INK),
                    s,
                    Transform::identity(),
                );
            } else {
                inset_outline(
                    px,
                    r,
                    6.0,
                    2.0,
                    rgba(243, 239, 230, 0.28 + 0.2 * h),
                    s,
                    Transform::identity(),
                );
            }
            c.label.draw(
                px,
                r.right() + 10.0,
                c.baseline + oy,
                s,
                None,
                Transform::identity(),
            );
            self.focus_ring(px, Hit::Check, r, 6.0, Transform::identity());
        }

        for (b, hit) in [(&l.primary, Hit::Primary), (&l.secondary, Hit::Secondary)] {
            if let Some(b) = b {
                self.draw_button(px, b, hit, oy);
            }
        }

        if let Some(track) = l.track {
            let track = R {
                y: track.y + oy,
                ..track
            };
            let status = shape(&self.status, style(Font::Onest700, 13.0, -0.01, MUTED));
            status.draw(
                px,
                X0,
                l.status_baseline + oy,
                s,
                None,
                Transform::identity(),
            );
            let pct = shape(
                &format!("{}%", (self.shown * 100.0).round() as i32),
                style(Font::Mono700, 12.0, 0.0, FG),
            );
            pct.draw(
                px,
                track.right() - pct.width,
                l.status_baseline + oy,
                s,
                None,
                Transform::identity(),
            );
            if let Some(p) = round_rect(track, 4.0, s) {
                paint::fill(
                    px,
                    &p,
                    &vgradient(
                        track,
                        s,
                        rgba(255, 255, 255, 0.08),
                        rgba(255, 255, 255, 0.08),
                        Some(rgba(0, 0, 0, 0.4)),
                    ),
                    Transform::identity(),
                );
            }
            let fw = (track.w * self.shown.clamp(0.0, 1.0)).max(if self.shown > 0.0 {
                track.h
            } else {
                0.0
            });
            if fw > 0.0 {
                let fill = R { w: fw, ..track };
                box_shadow(
                    px,
                    fill,
                    0.0,
                    0.0,
                    14.0,
                    0.0,
                    rgba(214, 255, 61, 0.45),
                    s,
                    1.0,
                );
                if let Some(p) = round_rect(fill, 4.0, s) {
                    paint::fill(
                        px,
                        &p,
                        &paint::hgradient(fill, s, hex(0xc6f02a), hex(0xe6ff78)),
                        Transform::identity(),
                    );
                }
                if let Some(p) = round_rect(
                    R::new(fill.x + 3.0, fill.y, (fill.w - 6.0).max(0.0), 1.0),
                    0.5,
                    s,
                ) {
                    paint::fill(
                        px,
                        &p,
                        &solid(rgba(255, 255, 255, 0.4)),
                        Transform::identity(),
                    );
                }
            }
        }
    }

    fn draw_button(&self, px: &mut Pixmap, b: &Button, hit: Hit, oy: f32) {
        let s = self.scale;
        let h = self.amount(hit);
        let lift = if b.style == ButtonStyle::Lime { h } else { 0.0 };
        let r = R {
            y: b.rect.y + oy - lift,
            ..b.rect
        };
        let (cx, cy) = ((r.x + r.w / 2.0) * s, (r.y + r.h / 2.0) * s);
        let k = if self.pressed == Some(hit) && self.hover == Some(hit) {
            0.97
        } else {
            1.0
        };
        let t = Transform::from_translate(cx, cy)
            .pre_scale(k, k)
            .pre_translate(-cx, -cy);
        match b.style {
            ButtonStyle::Lime => {
                box_shadow(
                    px,
                    r,
                    0.0,
                    10.0,
                    26.0,
                    -10.0,
                    rgba(214, 255, 61, 0.65),
                    s,
                    1.0,
                );
                if let Some(p) = round_rect(r, 22.0, s) {
                    let paint = vgradient(
                        r,
                        s,
                        mix(hex(0xe6ff78), hex(0xeeff9a), h),
                        mix(hex(0xd2fb36), hex(0xdcff52), h),
                        Some(rgba(255, 255, 255, 0.55)),
                    );
                    paint::fill(px, &p, &paint, t);
                }
            }
            ButtonStyle::Strong => {
                box_shadow(px, r, 0.0, 8.0, 20.0, -8.0, rgba(0, 0, 0, 0.7), s, 1.0);
                if let Some(p) = round_rect(r, 22.0, s) {
                    let paint = vgradient(
                        r,
                        s,
                        mix(hex(0xfaf7f0), hex(0xffffff), h),
                        mix(hex(0xe6e0d3), hex(0xf1ece2), h),
                        Some(rgba(255, 255, 255, 0.9)),
                    );
                    paint::fill(px, &p, &paint, t);
                }
            }
            ButtonStyle::Ghost => {
                if let Some(p) = round_rect(r, 22.0, s) {
                    paint::fill(px, &p, &solid(rgba(255, 255, 255, 0.05 * h)), t);
                }
                inset_outline(px, r, 22.0, 2.0, rgba(243, 239, 230, 0.18 + 0.16 * h), s, t);
            }
        }
        let content = b.label.width + if b.icon.is_some() { 26.0 } else { 0.0 };
        let mut x = r.x + (r.w - content) / 2.0;
        if let Some(icon) = b.icon {
            let (color, width) = if b.style == ButtonStyle::Ghost {
                (hex(FG), 2.4)
            } else {
                (hex(INK), 2.8)
            };
            paint::icon(px, icon, x, r.y + 13.5, 17.0, width, color, s, t);
            x += 26.0;
        }
        b.label.draw(
            px,
            x,
            centred_baseline(Font::Onest800, 15.0, r.y + 22.0),
            s,
            None,
            t,
        );
        self.focus_ring(px, hit, r, 22.0, Transform::identity());
    }

    fn draw_controls(&self, px: &mut Pixmap) {
        let s = self.scale;
        for (hit, r) in [(Hit::Minimize, MINIMIZE), (Hit::Close, CLOSE)] {
            let enabled = hit != Hit::Close || self.close_enabled();
            let h = if enabled { self.amount(hit) } else { 0.0 };
            if h > 0.0 {
                let bg = if hit == Hit::Close {
                    rgba(196, 43, 28, h)
                } else {
                    rgba(255, 255, 255, 0.08 * h)
                };
                if let Some(p) = round_rect(r, 0.0, s) {
                    paint::fill(px, &p, &solid(bg), Transform::identity());
                }
            }
            let mut color = if hit == Hit::Close {
                mix(hex(MUTED), hex(0xffffff), h)
            } else {
                mix(hex(MUTED), hex(FG), h)
            };
            if !enabled {
                color = rgba(141, 138, 132, 0.35);
            }
            let icon = if hit == Hit::Close {
                Icon::Close
            } else {
                Icon::Minimize
            };
            paint::icon(
                px,
                icon,
                r.x + 18.0,
                r.y + 13.0,
                10.0,
                2.64,
                color,
                s,
                Transform::identity(),
            );
        }
    }
}
