use crate::{ReaperStr, ReaperString};
use std::ffi::CString;

pub(crate) fn concat_reaper_strs(first: &ReaperStr, second: &ReaperStr) -> ReaperString {
    ReaperString::new(
        CString::new([first.as_c_str().to_bytes(), second.as_c_str().to_bytes()].concat())
            .expect("impossible"),
    )
}
