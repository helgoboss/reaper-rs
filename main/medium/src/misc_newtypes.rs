//! This module defines various newtypes in order to achieve more type safety.
use crate::{ReaperStr, ReaperStringArg, TryFromGreaterError};
use derive_more::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

/// A command ID.
///
/// This uniquely identifies a command[^command] within a certain [`section`]. For built-in actions
/// this command ID is completely stable. For actions added by extensions it should be assumed that
/// the command ID is valid only within one REAPER session.
///
/// This is not  to be confused with the command index (the position in the action list) and the
/// command name (a globally unique string identifier for commands added by extensions which is
/// stable even across different REAPER sessions).
///
/// [`section`]: struct.KbdSectionInfo.html
///
/// [^command]: A command is a function that will be executed when a particular action is requested
/// to be run.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "u32")
)]
// c_ulong is u64 on Linux, but on Windows u32. We don't want the consumer to deal with those
// toolchain differences and therefore choose u32. Rationale: The REAPER header files represent
// command IDs usually as c_int, which is basically always i32. Also makes sense ... why would
// someone need 2^64 commands!
pub struct CommandId(pub(crate) u32);

impl CommandId {
    fn is_valid(value: u32) -> bool {
        value != 0
    }

    /// Creates a command ID.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is 0 (which is not a valid command ID).
    pub fn new(value: u32) -> CommandId {
        assert!(Self::is_valid(value), "0 is not a valid command ID");
        CommandId(value)
    }

    /// Creates a command ID without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is greater than 0.
    pub const unsafe fn new_unchecked(value: u32) -> CommandId {
        CommandId(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

impl TryFrom<u32> for CommandId {
    type Error = TryFromGreaterError<u32>;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "0 is not a valid command ID",
                value,
            ));
        }
        Ok(CommandId(value))
    }
}

/// A section ID.
///
/// This uniquely identifies a [`section`].
///
/// [`section`]: struct.KbdSectionInfo.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SectionId(pub(crate) u32);

impl SectionId {
    /// Creates a section ID.
    pub fn new(number: u32) -> SectionId {
        SectionId(number)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

/// A marker or region ID.
///
/// This uniquely identifies a marker or region. Zero is also a valid ID.
/// Region IDs and marker IDs are two separate ID spaces.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BookmarkId(pub(crate) u32);

impl BookmarkId {
    /// Creates a marker ID.
    pub fn new(number: u32) -> BookmarkId {
        BookmarkId(number)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

/// An OS-dependent color.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NativeColor(pub(crate) u32);

impl NativeColor {
    /// Creates a native color.
    pub fn new(number: u32) -> NativeColor {
        NativeColor(number)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

/// A MIDI input device ID.
///
/// This uniquely identifies a MIDI input device according to the REAPER MIDI device preferences.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "u8")
)]
pub struct MidiInputDeviceId(pub(crate) u8);

impl MidiInputDeviceId {
    fn is_valid(value: u8) -> bool {
        value < 63
    }

    /// Creates the MIDI input device ID.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not a valid ID (must be <= 62).
    pub fn new(value: u8) -> MidiInputDeviceId {
        assert!(
            Self::is_valid(value),
            format!("MIDI input device IDs must be <= 62, got {}", value)
        );
        MidiInputDeviceId(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

impl TryFrom<u8> for MidiInputDeviceId {
    type Error = TryFromGreaterError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "MIDI input device IDs must be <= 62",
                value,
            ));
        }
        Ok(MidiInputDeviceId(value))
    }
}

/// A MIDI output device ID.
///
/// This uniquely identifies a MIDI output device according to the REAPER MIDI device preferences.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MidiOutputDeviceId(pub(crate) u8);

impl MidiOutputDeviceId {
    /// Creates the MIDI output device ID.
    pub fn new(number: u8) -> MidiOutputDeviceId {
        MidiOutputDeviceId(number)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

/// This represents a particular value of an FX parameter in "REAPER-normalized" form.
///
/// Please note that this value is **not** always normalized in the classical sense of being in the
/// unit interval 0.0..=1.0! Mostly it is and this is definitely the frame of reference. But there
/// are situations where it can be > 1.0. Turns out, it can even be a negative value! The meaning
/// depends on the particular FX.
///
/// Examples of FX parameters which can take values that are not in the unit interval:
/// - *ReaPitch* has a *Wet* parameter which has a "reasonable" maximum at 6 dB which corresponds to
///   the REAPER-normalized value 1.0. But this reasonable maximum can be exceeded, in which case it
///   can almost reach 2.0.
/// - *TAL Flanger* has a *Sync Speed* parameter which reports the min/max range as 0.0..=1.0 but
///   returns values between 0.0 and 8.0. It reports the range incorrectly.
/// - *Xfer Records LFO Tool* has an envelope point control that reports a value that is slightly
///   below zero when dragged all down. That's probably a bug.
/// - Because of a bug in REAPER <= 6.12 `SetParamNormalized`, it's possible that certain JS FX
///   parameter values end up as NaN, in Lua console displayed as "-1.#IND". E.g. happened to JS FX
///   "MIDI Note-On Delay" parameter "Poo". Bug has been reported.
///
/// Justin said that  0.0..=1.0 is the normal VST parameter range but that some ReaPlugs extend that
/// range when it's convenient (e.g. increasing the range from the initial version of the plug-in
/// or using values greater than 1.0 for volume when using gain etc.). He pointed out that
/// developers should prepare for anything.
///
/// We don't try to "fix" exotic values in medium-level API (e.g. setting negative values to zero
/// automatically) because there might be plug-ins which assign meaning to these special values and
/// then it would be a shame if we can't set or get them.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(from = "f64"))]
pub struct ReaperNormalizedFxParamValue(pub(crate) f64);

impl ReaperNormalizedFxParamValue {
    /// Creates a REAPER-normalized FX parameter value.
    pub fn new(value: f64) -> ReaperNormalizedFxParamValue {
        ReaperNormalizedFxParamValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl From<f64> for ReaperNormalizedFxParamValue {
    fn from(value: f64) -> Self {
        ReaperNormalizedFxParamValue(value)
    }
}

/// This represents a tempo measured in beats per minute.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct Bpm(pub(crate) f64);

impl Bpm {
    /// The minimum possible value (1.0 bpm).
    pub const MIN: Bpm = Bpm(1.0);

    /// The maximum possible value (960.0 bpm).
    pub const MAX: Bpm = Bpm(960.0);

    fn is_valid(value: f64) -> bool {
        Bpm::MIN.get() <= value && value <= Bpm::MAX.get()
    }

    /// Creates a BPM value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the BPM range supported by REAPER
    /// `(1.0..=960.0)`.
    pub fn new(value: f64) -> Bpm {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid Bpm value", value)
        );
        Bpm(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Bpm {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between 1.0 and 960.0",
                value,
            ));
        }
        Ok(Bpm(value))
    }
}

/// This represents a play rate measured as factor of the normal play speed.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct PlaybackSpeedFactor(pub(crate) f64);

impl PlaybackSpeedFactor {
    /// The minimum possible value (a quarter of the normal playback speed).
    pub const MIN: PlaybackSpeedFactor = PlaybackSpeedFactor(0.25);

    /// The normal playback speed.
    pub const NORMAL: PlaybackSpeedFactor = PlaybackSpeedFactor(1.00);

    /// The maximum possible value (four times the normal playback speed).
    pub const MAX: PlaybackSpeedFactor = PlaybackSpeedFactor(4.0);

    fn is_valid(value: f64) -> bool {
        PlaybackSpeedFactor::MIN.get() <= value && value <= PlaybackSpeedFactor::MAX.get()
    }

    /// Creates a playback speed factor.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the playback speed range supported by
    /// REAPER `(0.25..=4.00)`.
    pub fn new(value: f64) -> PlaybackSpeedFactor {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid PlaybackSpeedFactor", value)
        );
        PlaybackSpeedFactor(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for PlaybackSpeedFactor {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between 0.25 and 4.00",
                value,
            ));
        }
        Ok(PlaybackSpeedFactor(value))
    }
}

/// This represents a play rate measured as value between 0 and 1.
///
/// This corresponds to the position on the project play rate slider.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct NormalizedPlayRate(pub(crate) f64);

impl NormalizedPlayRate {
    /// The minimum possible value (a quarter of the normal play speed).
    pub const MIN: NormalizedPlayRate = NormalizedPlayRate(0.0);

    /// The normal playback speed.
    pub const NORMAL: NormalizedPlayRate = NormalizedPlayRate(0.2);

    /// The maximum possible value (four times the normal play speed).
    pub const MAX: NormalizedPlayRate = NormalizedPlayRate(1.0);

    fn is_valid(value: f64) -> bool {
        NormalizedPlayRate::MIN.get() <= value && value <= NormalizedPlayRate::MAX.get()
    }

    /// Creates a normalized play rate.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within `(0.00..=1.00)`.
    pub fn new(value: f64) -> NormalizedPlayRate {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid NormalizedPlayRate", value)
        );
        NormalizedPlayRate(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for NormalizedPlayRate {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between 0.0 and 1.0",
                value,
            ));
        }
        Ok(NormalizedPlayRate(value))
    }
}

/// This represents a frequency measured in hertz (how often something happens per second).
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct Hz(pub(crate) f64);

impl Hz {
    fn is_valid(value: f64) -> bool {
        0.0 < value
    }

    /// Creates a hertz value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value zero or negative.
    pub fn new(value: f64) -> Hz {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid Hz value", value)
        );
        Hz(value)
    }

    /// Creates a hertz value without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is greater than 0.0.
    pub unsafe fn new_unchecked(value: f64) -> Hz {
        Hz(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Hz {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be greater than 0.0",
                value,
            ));
        }
        Ok(Hz(value))
    }
}

/// This represents a position expressed as amount of seconds.
///
/// Sometimes this is a negative number, e.g. when it's a position on the timeline and a metronome
/// count-in is used or at the very beginning of the project (maybe because of rounding). Negative
/// project start values don't seem to cause negative position values though.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct PositionInSeconds(pub(crate) f64);

impl PositionInSeconds {
    fn is_valid(value: f64) -> bool {
        !value.is_infinite() && !value.is_nan()
    }

    /// Creates a value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is a special number.
    pub fn new(value: f64) -> PositionInSeconds {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid PositionInSeconds value", value)
        );
        PositionInSeconds(value)
    }

    /// Creates a PositionInSeconds value without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is a non-special number.
    pub unsafe fn new_unchecked(value: f64) -> PositionInSeconds {
        PositionInSeconds(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for PositionInSeconds {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "PositionInSeconds value must be non-special",
                value,
            ));
        }
        Ok(PositionInSeconds(value))
    }
}

/// This represents a duration expressed as positive amount of seconds.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct DurationInSeconds(pub(crate) f64);

impl DurationInSeconds {
    fn is_valid(value: f64) -> bool {
        value >= 0.0 && !value.is_infinite() && !value.is_nan()
    }

    /// Creates a value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is negative or a special number.
    pub fn new(value: f64) -> DurationInSeconds {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid DurationInSeconds value", value)
        );
        DurationInSeconds(value)
    }

    /// Creates a DurationInSeconds value without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is positive.
    pub unsafe fn new_unchecked(value: f64) -> DurationInSeconds {
        DurationInSeconds(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for DurationInSeconds {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "DurationInSeconds value must be positive",
                value,
            ));
        }
        Ok(DurationInSeconds(value))
    }
}

/// This represents a duration expressed as positive amount of beats.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct DurationInBeats(pub(crate) f64);

impl DurationInBeats {
    fn is_valid(value: f64) -> bool {
        value >= 0.0 && !value.is_infinite() && !value.is_nan()
    }

    /// Creates a value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is negative or a special number.
    pub fn new(value: f64) -> DurationInBeats {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid DurationInBeats value", value)
        );
        DurationInBeats(value)
    }

    /// Creates a DurationInBeats value without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is positive.
    pub unsafe fn new_unchecked(value: f64) -> DurationInBeats {
        DurationInBeats(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for DurationInBeats {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "DurationInBeats value must be positive",
                value,
            ));
        }
        Ok(DurationInBeats(value))
    }
}

/// This represents a position expressed as an amount of beats.
///
/// Can be negative, see [`PositionInSeconds`](struct.PositionInSeconds.html).
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct PositionInBeats(pub(crate) f64);

impl PositionInBeats {
    fn is_valid(value: f64) -> bool {
        !value.is_infinite() && !value.is_nan()
    }

    /// Creates a value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is a special number.
    pub fn new(value: f64) -> PositionInBeats {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid PositionInBeats value", value)
        );
        PositionInBeats(value)
    }

    /// Creates a PositionInBeats value without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is not a special number.
    pub unsafe fn new_unchecked(value: f64) -> PositionInBeats {
        PositionInBeats(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for PositionInBeats {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new("value must be non-special", value));
        }
        Ok(PositionInBeats(value))
    }
}

/// This represents a volume measured in decibel.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct Db(pub(crate) f64);

impl Db {
    /// The minimum possible value (= [`MINUS_INF`]).
    ///
    /// There's no maximum value because REAPER allows to exceed the "soft maximum" of 12 dB!
    ///
    /// [`MINUS_INF`]: #associatedconstant.MINUS_INF
    pub const MIN: Db = Db::MINUS_INF;

    /// The not-a-number volume ([`f64::NAN`] = 1.#R dB).
    ///
    /// See [`ReaperVolumeValue::NAN`].
    ///
    /// [`ReaperVolumeValue::NAN`]: struct.ReaperVolumeValue.html#associatedconstant.NAN
    /// [`f64::NAN`]: std/primitive.f64.html#associatedconstant.NAN
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);

    /// The negative infinity volume (-1000.0 = -inf dB).
    pub const MINUS_INF: Db = Db(-1000.0);

    /// The "soft minimum" volume (-150.0 dB).
    pub const MINUS_150_DB: Db = Db(-150.0);

    /// The "unaltered" volume (0.0 dB).
    pub const ZERO_DB: Db = Db(0.0);

    /// The "soft maximum" volume (12.0 dB).
    pub const TWELVE_DB: Db = Db(12.0);

    fn is_valid(value: f64) -> bool {
        Db::MIN.get() <= value || value.is_nan()
    }

    /// Creates a decibel value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the decibel range supported by REAPER
    /// `(-1000.0..)`.
    pub fn new(value: f64) -> Db {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid Db value", value)
        );
        Db(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Db {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be greater than or equal to -1000.0",
                value,
            ));
        }
        Ok(Db(value))
    }
}

/// This represents a volume measured as fader position.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct VolumeSliderValue(pub(crate) f64);

impl VolumeSliderValue {
    /// The minimum possible value (= [`MINUS_INF`]).
    ///
    /// There's no maximum value because REAPER allows to exceed the "soft maximum" of 12 dB!
    ///
    /// [`MINUS_INF`]: #associatedconstant.MINUS_INF
    pub const MIN: VolumeSliderValue = VolumeSliderValue::MINUS_INF_DB;

    /// The not-a-number volume ([`f64::NAN`] = 1.#R dB).
    ///
    /// See [`ReaperVolumeValue::NAN`].
    ///
    /// [`ReaperVolumeValue::NAN`]: struct.ReaperVolumeValue.html#associatedconstant.NAN
    /// [`f64::NAN`]: std/primitive.f64.html#associatedconstant.NAN
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);

    /// The negative infinity volume (0.0 = -inf dB).
    pub const MINUS_INF_DB: VolumeSliderValue = VolumeSliderValue(0.0);

    /// The "soft minimum" volume (2.5138729793972 = -150.0 dB).
    pub const MINUS_150_DB: VolumeSliderValue = VolumeSliderValue(2.513_872_979_397_2);

    /// The "unaltered" volume (716.0 = 0.0 dB).
    pub const ZERO_DB: VolumeSliderValue = VolumeSliderValue(716.0);

    /// The "soft maximum" volume (1000.0 = 12.0 dB).
    pub const TWELVE_DB: VolumeSliderValue = VolumeSliderValue(1000.0);

    fn is_valid(value: f64) -> bool {
        VolumeSliderValue::MIN.get() <= value || value.is_nan()
    }

    /// Creates a volume slider value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(0.0..)`.
    pub fn new(value: f64) -> VolumeSliderValue {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid VolumeSliderValue", value)
        );
        VolumeSliderValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for VolumeSliderValue {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new("value must be positive", value));
        }
        Ok(VolumeSliderValue(value))
    }
}

/// This represents a volume measured in REAPER's native volume unit.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct ReaperVolumeValue(pub(crate) f64);

impl ReaperVolumeValue {
    /// The minimum possible value (0.0).
    ///
    /// If the scale would be linear, this would be less than -150 dB. But it's not. In practice,
    /// REAPER considers this as equal to the [`MINUS_150_DB`] value.
    ///
    /// There's no maximum value because REAPER allows to exceed the soft maximum of 12 dB!
    ///
    /// [`MINUS_150_DB`]: #associatedconstant.MINUS_150_DB
    pub const MIN: ReaperVolumeValue = ReaperVolumeValue(0.0);

    /// The not-a-number volume ([`f64::NAN`] = 1.#R dB).
    ///
    /// It's reasonable to assume that this isn't actually a valid value. However, REAPER doesn't
    /// prevent extensions from setting it, so you might run into it.
    ///
    /// [`f64::NAN`]: https://doc.rust-lang.org/std/f64/constant.NAN.html
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);

    /// The "soft minimum" volume (3.1622776601684e-008 = -150.0 dB).
    ///
    /// When setting a value, use [`MIN`] (0.0) instead because this is just an approximation.
    ///
    /// [`MIN`]: #associatedconstant.MIN
    pub const MINUS_150_DB: ReaperVolumeValue = ReaperVolumeValue(3.162_277_660_168_4e-_008);

    /// The "unaltered" volume (1.0 = 0.0 dB).
    pub const ZERO_DB: ReaperVolumeValue = ReaperVolumeValue(1.0);

    /// The "soft maximum" volume (3.981071705535 = 12.0 dB).
    pub const TWELVE_DB: ReaperVolumeValue = ReaperVolumeValue(3.981_071_705_535);

    fn is_valid(value: f64) -> bool {
        ReaperVolumeValue::MIN.get() <= value || value.is_nan()
    }

    /// Creates a REAPER volume value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(0.0..)`.
    pub fn new(value: f64) -> ReaperVolumeValue {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid ReaperVolumeValue", value)
        );
        ReaperVolumeValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for ReaperVolumeValue {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new("value must be positive", value));
        }
        Ok(ReaperVolumeValue(value))
    }
}

/// For being able to use it with `ValueChange`.
#[doc(hidden)]
impl From<ReaperVolumeValue> for f64 {
    fn from(v: ReaperVolumeValue) -> Self {
        v.0
    }
}

/// This represents a pan measured in REAPER's native pan unit.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
#[repr(transparent)]
pub struct ReaperPanValue(pub(crate) f64);

impl ReaperPanValue {
    /// The minimum possible value (= [`LEFT`]).
    ///
    /// [`LEFT`]: #associatedconstant.LEFT
    pub const MIN: ReaperPanValue = ReaperPanValue::LEFT;

    /// The "extreme" left value (-1.0).
    pub const LEFT: ReaperPanValue = ReaperPanValue(-1.0);

    /// The center value (0.0).
    pub const CENTER: ReaperPanValue = ReaperPanValue(0.0);

    /// The "extreme" right value (1.0).
    pub const RIGHT: ReaperPanValue = ReaperPanValue(1.0);

    /// The maximum possible value (= [`RIGHT`]).
    ///
    /// [`RIGHT`]: #associatedconstant.RIGHT
    pub const MAX: ReaperPanValue = ReaperPanValue::RIGHT;

    fn is_valid(value: f64) -> bool {
        ReaperPanValue::MIN.get() <= value && value <= ReaperPanValue::MAX.get()
    }

    /// Creates a pan value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(-1.0..=1.0)`.
    pub fn new(value: f64) -> ReaperPanValue {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid ReaperPanValue", value)
        );
        ReaperPanValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for ReaperPanValue {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between -1.0 and 1.0",
                value,
            ));
        }
        Ok(ReaperPanValue(value))
    }
}

/// For being able to use it with `ValueChange`.
#[doc(hidden)]
impl From<ReaperPanValue> for f64 {
    fn from(v: ReaperPanValue) -> Self {
        v.0
    }
}

/// This represents a width measured in REAPER's native width unit.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
#[repr(transparent)]
pub struct ReaperWidthValue(pub(crate) f64);

impl ReaperWidthValue {
    /// The minimum possible value (-1.0).
    pub const MIN: ReaperWidthValue = ReaperWidthValue(-1.0);

    /// The center value (0.0).
    pub const CENTER: ReaperWidthValue = ReaperWidthValue(0.0);

    /// The maximum possible value (1.0).
    pub const MAX: ReaperWidthValue = ReaperWidthValue(1.0);

    fn is_valid(value: f64) -> bool {
        ReaperWidthValue::MIN.get() <= value && value <= ReaperWidthValue::MAX.get()
    }

    /// Creates a width value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(-1.0..=1.0)`.
    pub fn new(value: f64) -> ReaperWidthValue {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid ReaperWidthValue", value)
        );
        ReaperWidthValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for ReaperWidthValue {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between -1.0 and 1.0",
                value,
            ));
        }
        Ok(ReaperWidthValue(value))
    }
}

/// For being able to use it with `ValueChange`.
#[doc(hidden)]
impl From<ReaperWidthValue> for f64 {
    fn from(v: ReaperWidthValue) -> Self {
        v.0
    }
}

/// This represents a value that could either be a pan or a width value.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
#[repr(transparent)]
pub struct ReaperPanLikeValue(pub(crate) f64);

impl ReaperPanLikeValue {
    /// The minimum possible value (-1.0).
    pub const MIN: ReaperPanLikeValue = ReaperPanLikeValue(-1.0);

    /// The center value (0.0).
    pub const CENTER: ReaperPanLikeValue = ReaperPanLikeValue(0.0);

    /// The maximum possible value (1.0).
    pub const MAX: ReaperPanLikeValue = ReaperPanLikeValue(1.0);

    fn is_valid(value: f64) -> bool {
        ReaperPanLikeValue::MIN.get() <= value && value <= ReaperPanLikeValue::MAX.get()
    }

    /// Creates a pan-like value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(-1.0..=1.0)`.
    pub fn new(value: f64) -> ReaperPanLikeValue {
        assert!(
            Self::is_valid(value),
            format!("{} is not a valid ReaperPanLikeValue", value)
        );
        ReaperPanLikeValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Converts this into a pan value.
    pub fn as_pan_value(self) -> ReaperPanValue {
        ReaperPanValue(self.0)
    }

    /// Converts this into a width value.
    pub fn as_width_value(self) -> ReaperWidthValue {
        ReaperWidthValue(self.0)
    }
}

impl TryFrom<f64> for ReaperPanLikeValue {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between -1.0 and 1.0",
                value,
            ));
        }
        Ok(ReaperPanLikeValue(value))
    }
}

/// Represents a particular version of REAPER.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ReaperVersion<'a>(Cow<'a, ReaperStr>);

impl<'a> ReaperVersion<'a> {
    /// Creates a REAPER version.
    pub fn new(expression: impl Into<ReaperStringArg<'a>>) -> ReaperVersion<'a> {
        ReaperVersion(expression.into().into_inner())
    }

    /// Consumes this version and spits out the contained cow.
    pub fn into_inner(self) -> Cow<'a, ReaperStr> {
        self.0
    }
}

impl<'a> Display for ReaperVersion<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str())
    }
}

/// A MIDI frame offset.
///
/// This is a 1/1024000 of a second, *not* a sample frame!
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MidiFrameOffset(pub(crate) u32);

impl MidiFrameOffset {
    /// Creates a MIDI frame offset.
    pub fn new(value: u32) -> MidiFrameOffset {
        MidiFrameOffset(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        self.0 as i32
    }
}

/// Represents a value which can neither be accessed nor created by the consumer.
///
/// It's mainly used inside `Unknown` variants in order to enable forward compatibility without
/// information loss.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Hidden<T>(pub(crate) T);
