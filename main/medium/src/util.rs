use crate::{ReaperStr, ReaperString};
use std::ffi::CString;
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
