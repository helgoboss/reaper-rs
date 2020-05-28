use derive_more::*;
use std::fmt::{Debug, Display};

/// An error which can occur when executing a REAPER function.
///
/// This is not an error caused by *reaper-rs*, but one reported by REAPER itself.
/// The error message is not very specific most of the time because REAPER functions usually don't
/// give information about the cause of the error.
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

pub(crate) type ReaperFunctionResult<T> = Result<T, ReaperFunctionError>;

/// An error which can occur when trying to convert a low-level type to a medium-level type.
///
/// This error is caused by *reaper-rs*, not by REAPER itself.
#[derive(Debug, Clone, Eq, PartialEq, Display)]
#[display(fmt = "conversion from raw value [{}] failed: {}", raw_value, message)]
pub struct TryFromRawError<R> {
    message: &'static str,
    raw_value: R,
}

impl<R: Copy> TryFromRawError<R> {
    pub(crate) fn new(message: &'static str, raw_value: R) -> TryFromRawError<R> {
        TryFromRawError { message, raw_value }
    }
}

impl<R: Copy + Display + Debug> std::error::Error for TryFromRawError<R> {}

/// An error which can occur when converting from a type with a greater value range to one with a
/// smaller one.
///
/// This error is caused by *reaper-rs*, not by REAPER itself.
#[derive(Debug, Clone, Eq, PartialEq, Display)]
#[display(fmt = "conversion from value [{}] failed: {}", value, message)]
pub struct TryFromGreaterError<V> {
    message: &'static str,
    value: V,
}

impl<V: Copy> TryFromGreaterError<V> {
    pub(crate) fn new(message: &'static str, value: V) -> TryFromGreaterError<V> {
        TryFromGreaterError { message, value }
    }
}

impl<R: Copy + Display + Debug> std::error::Error for TryFromGreaterError<R> {}
