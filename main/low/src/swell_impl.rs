#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
use crate::{bindings::root, ReaperPluginContext, Swell, SwellFunctionPointers};

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

    /// Returns the plug-in context.
    pub fn plugin_context(&self) -> &ReaperPluginContext {
        self.plugin_context
            .as_ref()
            .expect("plug-in context not available on demo instances")
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn CreateDialogParam(
        &self,
        hinst: root::HINSTANCE,
        resid: *const ::std::os::raw::c_char,
        par: root::HWND,
        dlgproc: root::DLGPROC,
        param: root::LPARAM,
    ) -> root::HWND {
        #[cfg(target_os = "linux")]
        {
            self.SWELL_CreateDialog(
                // root::SWELL_curmodule_dialogresource_head only works with dialog gen
                std::ptr::null_mut(),
                resid,
                par,
                dlgproc,
                param,
            )
        }
        #[cfg(target_os = "windows")]
        {
            windows::CreateDialogParamA(hinst, resid, par, dlgproc, param)
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn SetWindowText(
        &self,
        hwnd: root::HWND,
        text: *const ::std::os::raw::c_char,
    ) -> root::BOOL {
        #[cfg(target_os = "linux")]
        {
            self.SetDlgItemText(hwnd, 0, text)
        }
        #[cfg(target_os = "windows")]
        {
            windows::SetWindowTextA(hwnd, text)
        }
    }
}

impl std::fmt::Debug for SwellFunctionPointers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwellFunctionPointers")
            .field("loaded_count", &self.loaded_count)
            .field("total_count", &Self::TOTAL_COUNT)
            .finish()
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
    extern "C" {
        pub fn SetWindowTextA(hwnd: root::HWND, text: *const ::std::os::raw::c_char) -> root::BOOL;
    }
}
