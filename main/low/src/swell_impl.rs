#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::{bindings::root, Swell, SwellFunctionPointers};

// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Swell> = None;
static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();

// This impl block contains mainly functions which exist in SWELL as macros only.
impl Swell {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    pub fn make_available_globally(functions: Swell) {
        unsafe {
            INIT_INSTANCE.call_once(|| INSTANCE = Some(functions));
        }
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Swell {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("call `make_available_globally()` before using `get()`")
        }
    }

    /// Gives access to the SWELL function pointers.
    pub fn pointers(&self) -> &SwellFunctionPointers {
        &self.pointers
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    #[cfg(target_os = "linux")]
    pub unsafe fn CreateDialogParam(
        &self,
        _hinst: root::HINSTANCE,
        resid: *const ::std::os::raw::c_char,
        par: root::HWND,
        dlgproc: root::DLGPROC,
        param: root::LPARAM,
    ) -> root::HWND {
        self.SWELL_CreateDialog(
            root::SWELL_curmodule_dialogresource_head,
            resid,
            par,
            dlgproc,
            param,
        )
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    #[cfg(target_os = "windows")]
    pub unsafe fn CreateDialogParam(
        &self,
        hinst: root::HINSTANCE,
        resid: *const ::std::os::raw::c_char,
        par: root::HWND,
        dlgproc: root::DLGPROC,
        param: root::LPARAM,
    ) -> root::HWND {
        windows::CreateDialogParamA(hinst, resid, par, dlgproc, param)
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use crate::bindings::root;

    extern "C" {
        pub fn CreateDialogParamA(
            hinst: root::HINSTANCE,
            resid: *const ::std::os::raw::c_char,
            par: root::HWND,
            dlgproc: root::DLGPROC,
            param: root::LPARAM,
        ) -> root::HWND;
    }
}
