use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

// Unsafe because it's not sure if the given pointer points to a value of type T.
pub unsafe fn get_ptr_content_as_copy<T: Copy>(ptr: *mut c_void) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    let ptr = ptr as *mut T;
    Some(*ptr)
}

// Unsafe because lifetime of returned string reference is unbounded and because it's not sure if
// the given pointer points to a C string.
pub unsafe fn get_ptr_content_as_c_str<'a>(ptr: *mut c_void) -> Option<&'a CStr> {
    if ptr.is_null() {
        return None;
    }
    let value = ptr as *const c_char;
    Some(CStr::from_ptr(value))
}
