use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use helgoboss_midi::{U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr::null_mut;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReaperVersion(Cow<'static, CStr>);

impl ReaperVersion {
    pub fn from(expression: impl Into<ReaperStringArg<'static>>) -> ReaperVersion {
        ReaperVersion(expression.into().into_cow())
    }
}
