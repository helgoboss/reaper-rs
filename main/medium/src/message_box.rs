use crate::TryFromRawError;

/// Type of message box to be displayed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MessageBoxType {
    Okay,
    OkayCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
}

impl MessageBoxType {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(&self) -> i32 {
        use MessageBoxType::*;
        match self {
            Okay => 0,
            OkayCancel => 1,
            AbortRetryIgnore => 2,
            YesNoCancel => 3,
            YesNo => 4,
            RetryCancel => 5,
        }
    }
}

/// Message box result informing about the user's choice.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MessageBoxResult {
    Okay,
    Cancel,
    Abort,
    Retry,
    Ignore,
    Yes,
    No,
}

impl MessageBoxResult {
    /// Converts an integer as returned by the low-level API to an automation mode.
    pub fn try_from_raw(v: i32) -> Result<MessageBoxResult, TryFromRawError<i32>> {
        use MessageBoxResult::*;
        match v {
            1 => Ok(Okay),
            2 => Ok(Cancel),
            3 => Ok(Abort),
            4 => Ok(Retry),
            5 => Ok(Ignore),
            6 => Ok(Yes),
            7 => Ok(No),
            _ => Err(TryFromRawError::new(
                "couldn't convert to message box result",
                v,
            )),
        }
    }
}
