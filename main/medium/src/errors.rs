use derive_more::*;

pub(crate) type ReaperFunctionResult<T> = Result<T, ReaperFunctionError>;

/// An error which can occur when executing a REAPER function.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "REAPER function failed: {}", message)]
pub struct ReaperFunctionError {
    message: &'static str,
}

impl ReaperFunctionError {
    pub(crate) fn new(message: &'static str) -> ReaperFunctionError {
        ReaperFunctionError { message }
    }
}

// ##### Conversion errors

/// An error which can occur when trying to convert a low-level FX index.
#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "FX index invalid")]
pub struct FxIndexInvalid;

/// An error which can occur when trying to convert a low-level raw representation to a medium-level
/// enum variant.
#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "conversion from raw representation failed")]
pub struct ConversionFromRawFailed;

/// An error which can occur when trying to convert a low-level recording input index.
#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "recording input index invalid")]
pub struct RecInputIndexInvalid;
