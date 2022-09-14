use reaper_medium::ReaperFunctionError;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub(crate) type ReaperResult<T> = Result<T, ReaperError>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperError {
    message: &'static str,
}

impl ReaperError {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &'static str {
        self.message
    }
}

impl Error for ReaperError {}

impl Display for ReaperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}

impl From<ReaperError> for &'static str {
    fn from(e: ReaperError) -> Self {
        e.message
    }
}

impl From<ReaperFunctionError> for ReaperError {
    fn from(e: ReaperFunctionError) -> Self {
        Self::new(e.message())
    }
}

impl From<&'static str> for ReaperError {
    fn from(e: &'static str) -> Self {
        Self::new(e)
    }
}
