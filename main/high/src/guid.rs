use crate::Reaper;
use reaper_low::raw::GUID;
use std::convert;

use reaper_medium::ReaperFunctions;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;
use std::str;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Guid {
    internal: GUID,
}

impl Guid {
    pub fn new(internal: GUID) -> Guid {
        Guid { internal }
    }

    pub fn to_string_with_braces(&self) -> String {
        let c_string = ReaperFunctions::get().guid_to_string(&self.internal);
        c_string.into_string().unwrap()
    }

    pub fn to_string_without_braces(&self) -> String {
        let mut s = self.to_string_with_braces();
        s.remove(0);
        s.truncate(36);
        s
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string_with_braces())
    }
}

impl From<&Guid> for CString {
    fn from(guid: &Guid) -> Self {
        ReaperFunctions::get().guid_to_string(&guid.internal)
    }
}

impl convert::TryFrom<&CStr> for Guid {
    type Error = &'static str;

    fn try_from(value: &CStr) -> Result<Guid, Self::Error> {
        ReaperFunctions::get()
            .string_to_guid(value)
            .map(Guid::new)
            .map_err(|_| "Invalid GUID")
    }
}
