//! The NSIS plug-in interface (Unicode NSIS 3): the string stack, the variables and the
//! callback that keeps the plug-in loaded until the installer exits.

use std::ffi::c_void;
use windows::Win32::Foundation::HWND;

#[repr(C)]
pub struct StackT {
    next: *mut StackT,
    text: [u16; 1],
}

pub type PluginCallback = unsafe extern "C" fn(msg: i32) -> usize;

#[repr(C)]
pub struct ExtraParameters {
    pub exec_flags: *mut c_void,
    pub execute_code_segment: Option<unsafe extern "system" fn(i32, HWND) -> i32>,
    pub validate_filename: Option<unsafe extern "system" fn(*mut u16)>,
    pub register_plugin_callback:
        Option<unsafe extern "system" fn(*mut c_void, PluginCallback) -> i32>,
}

/// `NSPIM_GUIUNLOAD`: the installer window is about to close.
pub const NSPIM_GUIUNLOAD: i32 = 1;

/// `$INSTDIR` in the variable table (`$0`-`$9`, `$R0`-`$R9`, `$CMDLINE`, `$INSTDIR`, ...).
pub const INST_INSTDIR: usize = 21;

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
    fn GlobalFree(mem: *mut c_void) -> *mut c_void;
}
const GPTR: u32 = 0x0040;

/// One plug-in call: its stack and variables.
pub struct Call {
    stacktop: *mut *mut StackT,
    size: usize,
    vars: *mut u16,
}

impl Call {
    /// # Safety
    /// The pointers must be the ones NSIS passed to the exported function.
    pub unsafe fn new(string_size: i32, variables: *mut u16, stacktop: *mut *mut StackT) -> Call {
        Call {
            stacktop,
            size: string_size.max(1) as usize,
            vars: variables,
        }
    }

    /// Pops a string; an empty stack gives "".
    pub fn pop(&self) -> String {
        unsafe {
            if self.stacktop.is_null() || (*self.stacktop).is_null() {
                return String::new();
            }
            let th = *self.stacktop;
            let text = std::ptr::addr_of!((*th).text) as *const u16;
            let mut len = 0;
            while len < self.size && *text.add(len) != 0 {
                len += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(text, len));
            *self.stacktop = (*th).next;
            GlobalFree(th as *mut c_void);
            s
        }
    }

    pub fn push(&self, s: &str) {
        unsafe {
            if self.stacktop.is_null() {
                return;
            }
            let bytes = std::mem::size_of::<StackT>() + self.size * 2;
            let th = GlobalAlloc(GPTR, bytes) as *mut StackT;
            if th.is_null() {
                return;
            }
            let text = std::ptr::addr_of_mut!((*th).text) as *mut u16;
            for (i, u) in s.encode_utf16().take(self.size - 1).enumerate() {
                *text.add(i) = u;
            }
            (*th).next = *self.stacktop;
            *self.stacktop = th;
        }
    }

    pub fn vars(&self) -> Vars {
        Vars {
            ptr: self.vars,
            size: self.size,
        }
    }
}

/// The variable table; stays valid for the life of the installer.
#[derive(Clone, Copy)]
pub struct Vars {
    ptr: *mut u16,
    size: usize,
}

unsafe impl Send for Vars {}

impl Vars {
    pub fn get(&self, index: usize) -> String {
        if self.ptr.is_null() {
            return String::new();
        }
        unsafe {
            let p = self.ptr.add(index * self.size);
            let mut len = 0;
            while len < self.size && *p.add(len) != 0 {
                len += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
        }
    }

    pub fn set(&self, index: usize, value: &str) {
        if self.ptr.is_null() {
            return;
        }
        unsafe {
            let p = self.ptr.add(index * self.size);
            let mut n = 0;
            for u in value.encode_utf16().take(self.size - 1) {
                *p.add(n) = u;
                n += 1;
            }
            *p.add(n) = 0;
        }
    }
}
