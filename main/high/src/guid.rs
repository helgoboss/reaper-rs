use reaper_low::raw::GUID;

use crate::Reaper;

use reaper_medium::ReaperStringArg;
use std::fmt;
use std::fmt::Formatter;
use std::str;
use std::str::FromStr;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Guid {
    internal: GUID,
}

impl Guid {
    pub fn new(internal: GUID) -> Guid {
        Guid { internal }
    }

    pub fn from_string_with_braces<'a>(
        text: impl Into<ReaperStringArg<'a>>,
    ) -> Result<Guid, &'static str> {
        Reaper::get()
            .medium_reaper()
            .string_to_guid(text)
            .map(Guid::new)
            .map_err(|_| "invalid GUID")
    }

    pub fn from_string_without_braces(text: &str) -> Result<Guid, &'static str> {
        Self::from_string_with_braces(format!("{{{}}}", text).as_str())
    }

    pub fn to_string_with_braces(&self) -> String {
        Reaper::get()
            .medium_reaper()
            .guid_to_string(&self.internal)
            .into_string()
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

impl FromStr for Guid {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('{') {
            Guid::from_string_with_braces(s)
        } else {
            Guid::from_string_without_braces(s)
        }
    }
}
