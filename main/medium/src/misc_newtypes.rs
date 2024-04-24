//! This module defines various newtypes in order to achieve more type safety.
use crate::{ReaperStr, ReaperStringArg, TryFromGreaterError};
use derive_more::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::num::NonZeroI32;

pub use reaper_common_types::Bpm;
pub use reaper_common_types::Db;
pub use reaper_common_types::DurationInBeats;
pub use reaper_common_types::DurationInSeconds;
pub use reaper_common_types::Hz;
pub use reaper_common_types::LinearVolumeValue as ReaperVolumeValue;
pub use reaper_common_types::PanValue as ReaperPanValue;
pub use reaper_common_types::PositionInBeats;
pub use reaper_common_types::PositionInPulsesPerQuarterNote;
pub use reaper_common_types::PositionInQuarterNotes;
pub use reaper_common_types::PositionInSeconds;

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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
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

impl Default for CommandId {
    fn default() -> Self {
        CommandId(1)
    }
}

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
pub struct NativeColor(pub(crate) i32);

impl NativeColor {
    /// Creates a native color.
    pub fn new(number: i32) -> NativeColor {
        NativeColor(number)
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub const fn to_raw(self) -> i32 {
        self.0
    }
}

/// A MIDI input device ID.
///
/// This uniquely identifies a MIDI input device according to the REAPER MIDI device preferences.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MidiInputDeviceId(pub(crate) u8);

impl MidiInputDeviceId {
    /// Maximum number of MIDI input devices.
    pub const MAX_DEVICE_COUNT: u8 = 129;

    /// Creates the MIDI input device ID.
    pub fn new(value: u8) -> MidiInputDeviceId {
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

/// A resample mode, backed by a positive integer.
///
/// This uniquely identifies a resample mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ResampleMode(pub(crate) u32);

impl ResampleMode {
    /// Creates the resample mode.
    pub fn new(number: u32) -> ResampleMode {
        ResampleMode(number)
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

/// A combination of pitch-shift mode and sub mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct FullPitchShiftMode {
    pub mode: PitchShiftMode,
    pub sub_mode: PitchShiftSubMode,
}

impl FullPitchShiftMode {
    /// Converts an integer as returned by the low-level API to a full pitch-shift mode.
    pub fn from_raw(v: i32) -> Option<Self> {
        if v < 0 {
            return None;
        }
        let v = v as u32;
        let full_mode = Self {
            mode: PitchShiftMode::new((v >> 2) & 0xFF),
            sub_mode: PitchShiftSubMode::new(v & 0xFF),
        };
        Some(full_mode)
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        let mode = self.mode.get();
        let sub_mode = self.sub_mode.get();
        ((mode << 2) | sub_mode) as i32
    }
}

/// A pitch shift mode, backed by a positive integer.
///
/// This uniquely identifies a pitch shift mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PitchShiftMode(pub(crate) u32);

impl PitchShiftMode {
    /// Creates the pitch shift mode.
    pub fn new(number: u32) -> PitchShiftMode {
        PitchShiftMode(number)
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

/// A pitch shift sub mode, backed by a positive integer.
///
/// This uniquely identifies a pitch shift sub mode within the parent pitch shift mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PitchShiftSubMode(pub(crate) u32);

impl PitchShiftSubMode {
    /// Creates the pitch shift sub mode.
    pub fn new(number: u32) -> PitchShiftSubMode {
        PitchShiftSubMode(number)
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

/// An item group ID.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "i32")
)]
pub struct ItemGroupId(pub(crate) NonZeroI32);

impl Default for ItemGroupId {
    fn default() -> Self {
        Self::MIN_POSITIVE
    }
}

impl ItemGroupId {
    pub const MIN_POSITIVE: Self = unsafe { Self::new_unchecked(1) };

    /// Creates an item group ID.
    pub fn new(value: i32) -> Option<ItemGroupId> {
        value.try_into().ok()
    }

    /// Returns the next valid group ID (skips zero).
    pub fn next(&self) -> ItemGroupId {
        let next = self.0.get() + 1;
        let next = if next == 0 { 1 } else { next };
        unsafe { Self::new_unchecked(next) }
    }

    /// Creates a command ID without bound checking.
    ///
    /// # Safety
    ///
    /// You must ensure that the given value is not 0.
    pub const unsafe fn new_unchecked(value: i32) -> ItemGroupId {
        ItemGroupId(NonZeroI32::new_unchecked(value))
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> i32 {
        self.0.get()
    }
}

impl TryFrom<i32> for ItemGroupId {
    type Error = TryFromGreaterError<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let inner = NonZeroI32::new(value).ok_or(TryFromGreaterError::new(
            "0 is not a valid item group ID",
            value,
        ))?;
        Ok(ItemGroupId(inner))
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

/// This represents a play rate measured as factor of the normal play speed.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct PlaybackSpeedFactor(pub(crate) f64);

impl Default for PlaybackSpeedFactor {
    fn default() -> Self {
        PlaybackSpeedFactor::NORMAL
    }
}

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
            "{value} is not a valid PlaybackSpeedFactor",
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
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
pub struct NormalizedPlayRate(pub(crate) f64);

impl Default for NormalizedPlayRate {
    fn default() -> Self {
        NormalizedPlayRate::NORMAL
    }
}

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
            "{value} is not a valid NormalizedPlayRate",
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

/// This represents a volume measured as fader position.
///
/// The scale should correspond to the one used in PGF8000 faders.
///
/// # Examples
///
/// - A value of 0.0 or very close corresponds to -inf dB
/// - A value of 3.1647785560398 corresponds to -144 dB (the first dB value not showed as -inf anymore in REAPER GUI)
/// - A value of 716 corresponds to 0.0 dB (unaltered volume)
/// - A value of 1000 corresponds to 12 dB
/// - Higher values are possible but harder to enter via GUI
///
/// # Usage
///
/// This is usually used for faders when mapping from dB to fader position and vice versa.
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
    /// [`VolumeSliderValue::NAN`]: struct.VolumeSliderValue.html#associatedconstant.NAN
    /// [`f64::NAN`]: std/primitive.f64.html#associatedconstant.NAN
    pub const NAN: VolumeSliderValue = VolumeSliderValue(f64::NAN);

    /// The negative infinity volume (0.0 = -inf dB).
    pub const MINUS_INF_DB: VolumeSliderValue = VolumeSliderValue(0.0);

    /// The old (?) "soft minimum" volume (2.5138729793972 = -150.0 dB).
    pub const MINUS_150_DB: VolumeSliderValue = VolumeSliderValue(2.5138729793972);

    /// The new (?) "soft minimum" volume (3.1647785560398 = -144.0 dB).
    pub const MINUS_144_DB: VolumeSliderValue = VolumeSliderValue(3.1647785560398);

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
            "{value} is not a valid VolumeSliderValue",
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
            "{value} is not a valid ReaperWidthValue",
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

/// This represents a fade curvature.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "f64")
)]
#[repr(transparent)]
pub struct FadeCurvature(pub(crate) f64);

impl FadeCurvature {
    /// The minimum possible value (-1.0).
    pub const MIN: FadeCurvature = FadeCurvature(-1.0);

    /// The center value (0.0).
    pub const LINEAR: FadeCurvature = FadeCurvature(0.0);

    /// The maximum possible value (1.0).
    pub const MAX: FadeCurvature = FadeCurvature(1.0);

    fn is_valid(value: f64) -> bool {
        FadeCurvature::MIN.get() <= value && value <= FadeCurvature::MAX.get()
    }

    /// Creates a fade curvature value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(-1.0..=1.0)`.
    pub fn new(value: f64) -> FadeCurvature {
        assert!(
            Self::is_valid(value),
            "{value} is not a valid FadeCurvature",
        );
        FadeCurvature(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for FadeCurvature {
    type Error = TryFromGreaterError<f64>;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !Self::is_valid(value) {
            return Err(TryFromGreaterError::new(
                "value must be between -1.0 and 1.0",
                value,
            ));
        }
        Ok(FadeCurvature(value))
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
            "{value} is not a valid ReaperPanLikeValue",
        );
        ReaperPanLikeValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Converts this into a pan value.
    pub fn as_pan_value(self) -> ReaperPanValue {
        unsafe { ReaperPanValue::new_unchecked(self.0) }
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

/// The frame rate used for MIDI events in [`crate::MidiInput::get_read_buf`] in Hertz.
pub const MIDI_INPUT_FRAME_RATE: Hz = unsafe { Hz::new_unchecked(1_024_000.0) };

// TODO-medium This is debatable. Yes, we don't want information loss. But hiding the value?
//  Too idealistic.
/// Represents a value which can neither be accessed nor created by the consumer.
///
/// It's mainly used inside `Unknown` variants in order to enable forward compatibility without
/// information loss.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Hidden<T>(pub(crate) T);
