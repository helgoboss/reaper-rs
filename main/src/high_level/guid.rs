use crate::low_level::GUID;
use std::fmt;
use crate::high_level::Reaper;
use std::str;
use std::str::FromStr;
use std::fmt::{Formatter, Error};
use std::ffi::{CStr, CString};
use std::convert;
use std::convert::TryFrom;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Guid {
    internal: GUID
}

impl Guid {
    pub fn new(internal: GUID) -> Guid {
        Guid {
            internal
        }
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let c_string = Reaper::instance().medium.guid_to_string(&self.internal);
        write!(f, "{:?}", c_string)
    }
}

impl From<&Guid> for CString {
    fn from(guid: &Guid) -> Self {
        Reaper::instance().medium.guid_to_string(&guid.internal)
    }
}

impl convert::TryFrom<&CStr> for Guid {
    type Error = &'static str;

    fn try_from(value: &CStr) -> Result<Guid, Self::Error> {
        Reaper::instance().medium.string_to_guid(value)
            .map(|g| Guid::new(g))
            .ok_or("Invalid GUID")
    }
}