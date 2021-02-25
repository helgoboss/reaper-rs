use crate::Hidden;

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
    pub fn to_raw(self) -> i32 {
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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl MessageBoxResult {
    /// Converts an integer as returned by the low-level API to an automation mode.
    pub fn from_raw(v: i32) -> MessageBoxResult {
        use MessageBoxResult::*;
        match v {
            1 => Okay,
            2 => Cancel,
            3 => Abort,
            4 => Retry,
            5 => Ignore,
            6 => Yes,
            7 => No,
            x => Unknown(Hidden(x)),
        }
    }
}
