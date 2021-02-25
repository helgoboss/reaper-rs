use crate::Hidden;

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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl AutomationMode {
    /// Converts an integer as returned by the low-level API to an automation mode.
    pub fn from_raw(v: i32) -> AutomationMode {
        use AutomationMode::*;
        match v {
            0 => TrimRead,
            1 => Read,
            2 => Touch,
            3 => Write,
            4 => Latch,
            5 => LatchPreview,
            x => Unknown(Hidden(x)),
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
            Unknown(Hidden(x)) => x,
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
        assert_eq!(AutomationMode::from_raw(3), AutomationMode::Write);
        assert!(matches!(
            AutomationMode::from_raw(7),
            AutomationMode::Unknown(_)
        ));
    }
}
