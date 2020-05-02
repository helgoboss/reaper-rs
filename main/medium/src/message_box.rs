use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Type of message box to be displayed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, IntoPrimitive)]
#[repr(i32)]
pub enum MessageBoxType {
    Ok = 0,
    OkCancel = 1,
    AbortRetryIgnore = 2,
    YesNoCancel = 3,
    YesNo = 4,
    RetryCancel = 5,
}

/// Message box result informing about the user's choice.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, TryFromPrimitive)]
#[repr(i32)]
pub enum MessageBoxResult {
    Ok = 1,
    Cancel = 2,
    Abort = 3,
    Retry = 4,
    Ignore = 5,
    Yes = 6,
    No = 7,
}
