use crate::{ReaperStringArg};



use std::borrow::Cow;
use std::ffi::{CStr};



#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReaperVersion(Cow<'static, CStr>);

impl ReaperVersion {
    pub fn from(expression: impl Into<ReaperStringArg<'static>>) -> ReaperVersion {
        ReaperVersion(expression.into().into_inner())
    }
}
