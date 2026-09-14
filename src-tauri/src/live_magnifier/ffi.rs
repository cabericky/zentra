use std::ffi::c_void;

pub type MagInitFn = unsafe extern "system" fn() -> i32;
pub type MagUninitFn = unsafe extern "system" fn() -> i32;
pub type MagSetFullscreenTransformFn = unsafe extern "system" fn(f32, i32, i32) -> i32;

#[link(name = "kernel32")]
extern "system" {
    pub fn LoadLibraryW(lpLibFileName: *const u16) -> *mut c_void;
    pub fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
    pub fn FreeLibrary(hLibModule: *mut c_void) -> i32;
}

#[link(name = "winmm")]
extern "system" {
    pub fn timeBeginPeriod(uPeriod: u32) -> u32;
    pub fn timeEndPeriod(uPeriod: u32) -> u32;
}

/// Dynamically loaded Windows Magnification API handle and function pointers.
pub struct MagnificationApi {
    pub h_module: *mut c_void,
    pub init_fn: MagInitFn,
    pub uninit_fn: MagUninitFn,
    pub set_transform_fn: MagSetFullscreenTransformFn,
}

impl MagnificationApi {
    /// Attempts to dynamically load `Magnification.dll` and resolve function pointers.
    pub fn load() -> Option<Self> {
        unsafe {
            let dll_name: Vec<u16> = "Magnification.dll\0".encode_utf16().collect();
            let h_module = LoadLibraryW(dll_name.as_ptr());
            if h_module.is_null() {
                return None;
            }

            let init_fn: Option<MagInitFn> = std::mem::transmute(GetProcAddress(
                h_module,
                b"MagInitialize\0".as_ptr(),
            ));
            let uninit_fn: Option<MagUninitFn> = std::mem::transmute(GetProcAddress(
                h_module,
                b"MagUninitialize\0".as_ptr(),
            ));
            let set_transform_fn: Option<MagSetFullscreenTransformFn> = std::mem::transmute(GetProcAddress(
                h_module,
                b"MagSetFullscreenTransform\0".as_ptr(),
            ));

            if let (Some(init), Some(uninit), Some(set_tf)) = (init_fn, uninit_fn, set_transform_fn) {
                if init() == 0 {
                    log::warn!("MagInitialize failed");
                }
                Some(Self {
                    h_module,
                    init_fn: init,
                    uninit_fn: uninit,
                    set_transform_fn: set_tf,
                })
            } else {
                let _ = FreeLibrary(h_module);
                None
            }
        }
    }

    /// Sets the fullscreen magnification transform factor and pixel offset.
    pub fn set_transform(&self, mag: f32, x: i32, y: i32) {
        unsafe {
            let _ = (self.set_transform_fn)(mag, x, y);
        }
    }

    /// Safely uninitializes the magnification engine and frees the library.
    pub fn shutdown(self) {
        unsafe {
            let _ = (self.set_transform_fn)(1.0, 0, 0);
            let _ = (self.uninit_fn)();
            let _ = FreeLibrary(self.h_module);
        }
    }

    /// Static helper to reset the physical display to 1.0x full screen.
    pub fn reset_display() {
        if let Some(api) = Self::load() {
            api.shutdown();
        }
    }
}