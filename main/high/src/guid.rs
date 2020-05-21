use reaper_low::raw::GUID;
use std::convert;

use crate::Reaper;

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
        let c_string = Reaper::get().medium_reaper().guid_to_string(&self.internal);
        c_string.into_string()
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

impl convert::TryFrom<&CStr> for Guid {
    type Error = &'static str;

    fn try_from(value: &CStr) -> Result<Guid, Self::Error> {
        Reaper::get()
            .medium_reaper()
            .string_to_guid(value)
            .map(Guid::new)
            .map_err(|_| "Invalid GUID")
    }
}
