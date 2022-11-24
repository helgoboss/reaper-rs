use crate::{ReaperStr, ReaperString, ReaperStringArg};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;

pub fn concat_reaper_strs(first: &ReaperStr, second: &ReaperStr) -> ReaperString {
    ReaperString::new(
        CString::new([first.as_c_str().to_bytes(), second.as_c_str().to_bytes()].concat())
            .expect("impossible"),
    )
}

pub unsafe fn create_passing_c_str<'a>(ptr: *const c_char) -> Option<&'a ReaperStr> {
    if ptr.is_null() {
        return None;
    }
    Some(ReaperStr::from_ptr(ptr))
}

pub fn with_string_buffer<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (ReaperString, T) {
    // Using with_capacity() here wouldn't be correct because it leaves the vector length at zero.
    let vec: Vec<u8> = vec![0; max_size as usize];
    with_string_buffer_internal(vec, max_size, fill_buffer)
}

pub fn with_string_buffer_prefilled<'a, T>(
    prefill: impl Into<ReaperStringArg<'a>>,
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (ReaperString, T) {
    let mut vec = Vec::from(prefill.into().as_reaper_str().as_c_str().to_bytes());
    vec.resize(max_size as usize, 0);
    with_string_buffer_internal(vec, max_size, fill_buffer)
}

pub fn with_string_buffer_internal<T>(
    vec: Vec<u8>,
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (ReaperString, T) {
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    let result = fill_buffer(raw, max_size as i32);
    let string = unsafe { ReaperString::new(CString::from_raw(raw)) };
    (string, result)
}

pub fn create_string_buffer(max_size: u32) -> *mut i8 {
    let vec: Vec<u8> = vec![0; max_size as usize];
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    raw
}

pub fn with_buffer<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (Vec<u8>, T) {
    let (vec, raw) = create_buffer(max_size);
    let result = fill_buffer(raw, max_size as i32);
    (vec, result)
}

pub fn create_buffer(max_size: u32) -> (Vec<u8>, *mut i8) {
    let mut vec: Vec<u8> = vec![0; max_size as usize];
    let raw = vec.as_mut_ptr() as *mut c_char;
    (vec, raw)
}

/// We really need a box here in order to obtain a thin pointer. We must not consume it, that's why
/// we take it as reference.
#[allow(clippy::borrowed_box)]
pub fn encode_user_data<U>(data: &Box<U>) -> *mut c_void {
    data.as_ref() as *const _ as *mut c_void
}

pub fn decode_user_data<'a, U>(data: *mut c_void) -> &'a mut U {
    assert!(!data.is_null());
    let data = data as *mut U;
    unsafe { &mut *data }
}
