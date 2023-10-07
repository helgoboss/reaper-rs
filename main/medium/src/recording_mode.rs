use crate::Hidden;

/// Recording mode of a track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RecordingMode {
    Input,
    StereoOut,
    None,
    StereoOutWithLatencyCompensation,
    MidiOutput,
    MonoOut,
    MonoOutWithLatencyCompensation,
    MidiOverdub,
    MidiReplace,
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl RecordingMode {
    /// Converts an integer as returned by the low-level API to a recording mode.
    ///
    /// # Errors
    ///
    /// Fails if the given integer is not a valid recording mode index.
    pub fn from_raw(rec_mode_index: i32) -> RecordingMode {
        use RecordingMode::*;
        match rec_mode_index {
            0 => Input,
            1 => StereoOut,
            2 => None,
            3 => StereoOutWithLatencyCompensation,
            4 => MidiOutput,
            5 => MonoOut,
            6 => MonoOutWithLatencyCompensation,
            7 => MidiOverdub,
            8 => MidiReplace,
            i => Unknown(Hidden(i)),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use RecordingMode::*;
        match self {
            Input => 0,
            StereoOut => 1,
            None => 2,
            StereoOutWithLatencyCompensation => 3,
            MidiOutput => 4,
            MonoOut => 5,
            MonoOutWithLatencyCompensation => 6,
            MidiOverdub => 7,
            MidiReplace => 8,
            Unknown(i) => i.0 as i32,
        }
    }
}
