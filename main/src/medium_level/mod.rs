//! Provides all functions from `reaper_plugin_functions.h` with the following improvements:
//! - Snake-case function and parameter names
//! - Return values instead of output parameters
//! - No C strings
//! - Panics if function not available (we should make sure on plug-in load that all necessary
//!   functions are available, maybe provide "_available" functions for conditional execution)
mod control_surface;

use std::ffi::{CString, CStr};
use std::ptr::{null_mut, null};
use std::os::raw::{c_char, c_void};
use crate::low_level;
use crate::low_level::{ReaProject, MediaTrack};
use c_str_macro::c_str;
pub use crate::medium_level::control_surface::ControlSurface;
use crate::medium_level::control_surface::DelegatingControlSurface;

pub struct Reaper {
    pub low: low_level::Reaper
}

fn with_string_buffer<T>(max_size: usize, fill_buffer: impl FnOnce(*mut c_char, usize) -> T) -> (CString, T) {
    let vec: Vec<u8> = vec![1; max_size as usize];
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    let result = fill_buffer(raw, max_size);
    let string = unsafe { CString::from_raw(raw) };
    (string, result)
}

impl Reaper {
    pub fn new(low: low_level::Reaper) -> Reaper {
        Reaper { low }
    }

    pub fn enum_projects(&self, idx: i32, projfn_out_optional_sz: i32) -> (*mut ReaProject, Option<CString>) {
        return if projfn_out_optional_sz == 0 {
            let project = self.low.EnumProjects.unwrap()(idx, null_mut(), 0);
            (project, None)
        } else {
            let (file_path, project) = with_string_buffer(projfn_out_optional_sz as usize, |buffer, max_size| {
                self.low.EnumProjects.unwrap()(idx, buffer, max_size as i32)
            });

            (project, if file_path.as_bytes().len() == 0 { None } else { Some(file_path) })
        };
    }

    pub fn get_track(&self,
                     proj: *mut ReaProject,
                     trackidx: i32,
    ) -> *mut MediaTrack {
        self.low.GetTrack.unwrap()(proj, trackidx)
    }

    pub fn validate_ptr_2(&self, proj: *mut ReaProject, pointer: *mut c_void, ctypename: &CStr) -> bool {
        self.low.ValidatePtr2.unwrap()(proj, pointer, ctypename.as_ptr())
    }

    pub fn get_set_media_track_info(&self, tr: *mut MediaTrack, parmname: &CStr, set_new_value: *mut c_void) -> *mut c_void {
        self.low.GetSetMediaTrackInfo.unwrap()(tr, parmname.as_ptr(), set_new_value)
    }

    /// # Examples
    ///
    /// ## Passing literal with zero runtime overhead
    /// ```
    /// reaper.show_console_msg(c_str!("Hello from Rust!"))
    /// ```
    /// - Uses macro `c_str!` to create new 0-terminated static literal embedded in binary
    ///
    /// ## Passing 0-terminated literal with borrowing
    /// ```
    /// let literal = "Hello from Rust!\0";
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(literal.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the literal is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing 0-terminated owned string with borrowing
    /// ```
    /// let owned = String::from("Hello from Rust!\0");
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the String is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing not 0-terminated owned string with moving
    /// ```
    /// let owned = String::from("Hello from Rust!");
    /// reaper.show_console_msg(&CString::new(owned).unwrap())
    /// ```
    /// - Moves owned string for appending 0 byte (maybe increasing String capacity)
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Absolutely zero-overhead variations
    ///
    /// If you really need absolutely zero-overhead, you need to resort to unsafe functions. But
    /// this should be done only in situations when you are very constrained, e.g. in audio thread
    /// (which is forbidden to call most of the REAPER SDK functions anyway).
    ///
    /// Look into [from_vec_unchecked](CString::from_vec_unchecked) or
    /// [from_bytes_with_nul_unchecked](CStr::from_bytes_with_nul_unchecked) respectively.
    pub fn show_console_msg(&self, msg: &CStr) {
        self.low.ShowConsoleMsg.unwrap()(msg.as_ptr())
    }

    pub fn plugin_register(&self, name: &CStr, infostruct: *mut c_void) -> i32 {
        self.low.plugin_register.unwrap()(name.as_ptr(), infostruct)
    }

    pub fn install_control_surface(&self, control_surface: impl ControlSurface + 'static) {
        let delegating_control_surface = DelegatingControlSurface::new(control_surface);
        self.low.install_control_surface(delegating_control_surface);
    }

    pub fn register_control_surface(&self) {
        self.plugin_register(c_str!("csurf_inst"), self.low.get_cpp_control_surface());
    }

    pub fn unregister_control_surface(&self) {
        self.plugin_register(c_str!("-csurf_inst"), self.low.get_cpp_control_surface());
    }

    // TODO Rename
    pub fn convenient_get_media_track_info_string(&self, tr: *mut MediaTrack, parmname: &CStr) -> CString {
        let info = self.get_set_media_track_info(tr, parmname, null_mut());
        let info = info as *const c_char;
        let c_str = unsafe { CStr::from_ptr(info) };
        c_str.to_owned()
    }
}