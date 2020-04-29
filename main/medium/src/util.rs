use super::{DelegatingControlSurface, MediumReaperControlSurface};
use crate::ReaperVersion;
use std::ffi::{CStr, CString};

pub(crate) fn concat_c_strs(first: &CStr, second: &CStr) -> CString {
    CString::new([first.to_bytes(), second.to_bytes()].concat()).unwrap()
}
