use crate::TryFromRawError;

/// Global override of track automation modes.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GlobalAutomationModeOverride {
    /// All automation is bypassed.
    Bypass,
    /// Automation mode of all tracks is overridden by this one.
    Mode(AutomationMode),
}

/// Automation mode of a track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AutomationMode {
    TrimRead,
    Read,
    Touch,
    Write,
    Latch,
    LatchPreview,
}

impl AutomationMode {
    /// Converts an integer as returned by the low-level API to an automation mode.
    pub fn try_from_raw(v: i32) -> Result<AutomationMode, TryFromRawError<i32>> {
        use AutomationMode::*;
        match v {
            0 => Ok(TrimRead),
            1 => Ok(Read),
            2 => Ok(Touch),
            3 => Ok(Write),
            4 => Ok(Latch),
            5 => Ok(LatchPreview),
            _ => Err(TryFromRawError::new(
                "couldn't convert to automation mode",
                v,
            )),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use AutomationMode::*;
        match self {
            TrimRead => 0,
            Read => 1,
            Touch => 2,
            Write => 3,
            Latch => 4,
            LatchPreview => 5,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_int() {
        assert_eq!(3, AutomationMode::Write.to_raw());
    }

    #[test]
    fn from_int() {
        assert_eq!(AutomationMode::try_from_raw(3), Ok(AutomationMode::Write));
        assert!(AutomationMode::try_from_raw(7).is_err());
    }
}
