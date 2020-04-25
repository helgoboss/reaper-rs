use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use helgoboss_midi::{U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr::null_mut;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReaperVersion {
    version_str: &'static CStr,
}

impl From<&'static CStr> for ReaperVersion {
    fn from(version_str: &'static CStr) -> Self {
        ReaperVersion { version_str }
    }
}
