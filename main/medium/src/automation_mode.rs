use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Global override of track automation modes.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GlobalAutomationModeOverride {
    /// All automation is bypassed.
    Bypass,
    /// Automation mode of all tracks is overridden by this one.
    Mode(AutomationMode),
}

/// Possible track automation modes.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum AutomationMode {
    TrimRead = 0,
    Read = 1,
    Touch = 2,
    Write = 3,
    Latch = 4,
    LatchPreview = 5,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn to_int() {
        assert_eq!(3, AutomationMode::Write.into());
    }

    #[test]
    fn from_int() {
        assert_eq!(AutomationMode::try_from(3), Ok(AutomationMode::Write));
        assert!(AutomationMode::try_from(7).is_err());
    }
}
