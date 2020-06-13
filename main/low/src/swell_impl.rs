#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
use crate::{bindings::root, PluginContext, Swell, SwellFunctionPointers};

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
    pub fn plugin_context(&self) -> &PluginContext {
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
        #[cfg(target_family = "unix")]
        {
            self.SWELL_CreateDialog(
                root::SWELL_curmodule_dialogresource_head,
                resid,
                par,
                dlgproc,
                param,
            )
        }
        #[cfg(target_family = "windows")]
        {
            #[allow(clippy::cast_ptr_alignment)]
            windows::CreateDialogParamW(hinst, resid as _, par, dlgproc, param)
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
        #[cfg(target_family = "unix")]
        {
            self.SetDlgItemText(hwnd, 0, text)
        }
        #[cfg(target_family = "windows")]
        {
            windows::SetWindowTextW(hwnd, utf8_to_16(text).as_ptr())
        }
    }

    /// Attention: Whereas the Windows original returns a length, this just returns success.
    ///
    /// In order to avoid surprises, on Windows it will behave like this, too.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetWindowText(
        &self,
        hwnd: root::HWND,
        lpString: root::LPSTR,
        nMaxCount: std::os::raw::c_int,
    ) -> root::BOOL {
        #[cfg(target_family = "unix")]
        {
            self.GetDlgItemText(hwnd, 0, lpString, nMaxCount)
        }
        #[cfg(target_family = "windows")]
        {
            with_utf16_to_8(lpString, nMaxCount, |buffer, max_size| {
                windows::GetWindowTextW(hwnd, buffer, max_size)
            })
        }
    }

    /// On Windows this is a constant but in SWELL this is a macro which translates to a function
    /// call.
    pub fn CF_TEXT(&self) -> root::UINT {
        #[cfg(target_family = "unix")]
        {
            unsafe { self.RegisterClipboardFormat(c_str_macro::c_str!("SWELL__CF_TEXT").as_ptr()) }
        }
        #[cfg(target_family = "windows")]
        1
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

#[cfg(target_family = "windows")]
mod windows {
    use crate::bindings::root;
    use std::os::raw::c_int;

    extern "C" {
        pub fn CreateDialogParamW(
            hinst: root::HINSTANCE,
            resid: *const u16,
            par: root::HWND,
            dlgproc: root::DLGPROC,
            param: root::LPARAM,
        ) -> root::HWND;
    }

    extern "C" {
        pub fn SetWindowTextW(hwnd: root::HWND, text: *const u16) -> root::BOOL;
    }

    extern "C" {
        pub fn GetWindowTextW(hwnd: root::HWND, lpString: *mut u16, nMaxCount: c_int) -> c_int;
    }
}

/// Converts the given UTF-8 C-style string (nul terminator) to an UTF-16 C-style string.
///
/// # Safety
///
/// You must ensure that the given string points to an UTF-8 encoded C-style string.
#[cfg(target_family = "windows")]
pub(crate) unsafe fn utf8_to_16(raw_utf8: *const std::os::raw::c_char) -> Vec<u16> {
    use std::ffi::{CStr, OsStr};
    use std::iter::once;
    // Assumes that the given pointer points to a C-style string.
    let utf8_c_str = CStr::from_ptr(raw_utf8);
    // Interpret that string as UTF-8-encoded. Fall back to replacement characters if not.
    let str = utf8_c_str.to_string_lossy();
    // Now reencode it as UTF-16.
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(str.as_ref())
        .encode_wide()
        .chain(once(0))
        .collect()
}

/// Creates a UTF-16 buffer (to be filled by the given function) and writes it as UTF-8 to the given
/// target buffer.
///
/// `max_size` must include nul terminator. The given function must return the actual string length
/// *without* nul terminator.
#[cfg(target_family = "windows")]
pub(crate) unsafe fn with_utf16_to_8(
    utf8_target_buffer: *mut std::os::raw::c_char,
    requested_max_size: std::os::raw::c_int,
    fill_utf16_source_buffer: impl FnOnce(*mut u16, i32) -> i32,
) -> root::BOOL {
    // TODO-medium Maybe use this vec initialization also in with_buffer
    let mut utf16_vec: Vec<u16> = Vec::with_capacity(requested_max_size as usize);
    // Returns length *without* nul terminator.
    let len = fill_utf16_source_buffer(utf16_vec.as_mut_ptr(), requested_max_size);
    if len == 0 {
        return 0;
    }
    utf16_vec.set_len(len as usize);
    // nul terminator will not be part of the string because len doesn't include it!
    let string = String::from_utf16_lossy(&utf16_vec);
    let c_string = match std::ffi::CString::new(string) {
        Ok(s) => s,
        Err(_) => {
            // String contained 0 byte. This would end a C-style string abruptly.
            return 0;
        }
    };
    let source_bytes = c_string.as_bytes_with_nul();
    let target_bytes =
        std::slice::from_raw_parts_mut(utf8_target_buffer, requested_max_size as usize);
    let source_bytes_signed = &*(source_bytes as *const [u8] as *const [i8]);
    target_bytes[..source_bytes.len()].copy_from_slice(source_bytes_signed);
    1
}
