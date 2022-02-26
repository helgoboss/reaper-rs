#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::bindings::root::reaper_project_state_context::*;
use crate::{firewall, raw};
use std::os::raw::c_void;
use std::ptr::NonNull;

impl raw::ProjectStateContext {
    /// Attention: Not really usable yet due to the lack of the variadic parameter in AddLine.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn AddLine(&mut self, line: *const ::std::os::raw::c_char) {
        rust_to_cpp_ProjectStateContext_AddLine(self as *const _ as _, line);
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetLine(
        &mut self,
        buf: *mut ::std::os::raw::c_char,
        buflen: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        rust_to_cpp_ProjectStateContext_GetLine(self as *const _ as _, buf, buflen)
    }

    pub fn GetOutputSize(&mut self) -> ::std::os::raw::c_longlong {
        unsafe { rust_to_cpp_ProjectStateContext_GetOutputSize(self as *const _ as _) }
    }

    pub fn GetTempFlag(&mut self) -> ::std::os::raw::c_int {
        unsafe { rust_to_cpp_ProjectStateContext_GetTempFlag(self as *const _ as _) }
    }

    pub fn SetTempFlag(&mut self, flag: ::std::os::raw::c_int) {
        unsafe { rust_to_cpp_ProjectStateContext_SetTempFlag(self as *const _ as _, flag) }
    }
}

/// This is the Rust analog to the C++ virtual base class `ProjectStateContext`.
///
/// An implementation of this trait can be passed to [`create_cpp_to_rust_project_state_context()`].
///
/// Attention: Not really usable yet due to the lack of the variadic parameter in AddLine.
///
/// [`create_cpp_to_rust_project_state_context()`]: fn.create_cpp_to_rust_project_state_context.html
pub trait ProjectStateContext {
    fn AddLine(&mut self, line: *const ::std::os::raw::c_char);
    fn GetLine(
        &mut self,
        buf: *mut ::std::os::raw::c_char,
        buflen: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
    fn GetOutputSize(&mut self) -> ::std::os::raw::c_longlong;
    fn GetTempFlag(&mut self) -> ::std::os::raw::c_int;
    fn SetTempFlag(&mut self, flag: ::std::os::raw::c_int);
}

/// Creates a `ProjectStateContext` object on C++ side and returns a pointer to it.
///
/// This function is provided because Rust structs can't implement C++ virtual base classes.
///
/// # Example
///
/// See [`create_cpp_to_rust_control_surface()`]. Usage is very similar.
///
/// # Cleaning up
///
/// In order to avoid memory leaks, you must take care of removing the C++ counterpart
/// ProjectStateContext by calling [`delete_cpp_project_state_context()`].
///
/// # Safety
///
/// This function is highly unsafe. Better use the medium-level API instead.
///
/// [`delete_cpp_project_state_context()`]: fn.delete_cpp_project_state_context.html
/// [`create_cpp_to_rust_control_surface()`]: fn.create_cpp_to_rust_control_surface.html
pub unsafe fn create_cpp_to_rust_project_state_context(
    callback_target: NonNull<Box<dyn ProjectStateContext>>,
) -> NonNull<raw::ProjectStateContext> {
    let instance = crate::bindings::root::reaper_project_state_context::create_cpp_to_rust_project_state_context(
        callback_target.as_ptr() as *mut c_void,
    );
    NonNull::new_unchecked(instance)
}

/// Destroys a C++ `ProjectStateContext` object.
///
/// Intended to be used on pointers returned from [`create_cpp_to_rust_project_state_context()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`create_cpp_to_rust_project_state_context()`]: fn.create_cpp_to_rust_project_state_context.html
pub unsafe fn delete_cpp_project_state_context(context: NonNull<raw::ProjectStateContext>) {
    crate::bindings::root::reaper_project_state_context::delete_project_state_context(
        context.as_ptr(),
    );
}

#[no_mangle]
extern "C" fn cpp_to_rust_ProjectStateContext_AddLine(
    callback_target: *mut Box<dyn ProjectStateContext>,
    line: *const ::std::os::raw::c_char,
) {
    firewall(|| unsafe { &mut *callback_target }.AddLine(line));
}

#[no_mangle]
extern "C" fn cpp_to_rust_ProjectStateContext_GetLine(
    callback_target: *mut Box<dyn ProjectStateContext>,
    buf: *mut ::std::os::raw::c_char,
    buflen: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetLine(buf, buflen)).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_ProjectStateContext_GetOutputSize(
    callback_target: *mut Box<dyn ProjectStateContext>,
) -> ::std::os::raw::c_longlong {
    firewall(|| unsafe { &mut *callback_target }.GetOutputSize()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_ProjectStateContext_GetTempFlag(
    callback_target: *mut Box<dyn ProjectStateContext>,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetTempFlag()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_ProjectStateContext_SetTempFlag(
    callback_target: *mut Box<dyn ProjectStateContext>,
    flag: ::std::os::raw::c_int,
) {
    firewall(|| unsafe { &mut *callback_target }.SetTempFlag(flag));
}
