//! This module defines various newtypes in order to achieve more type safety.
use crate::{ReaperStr, ReaperStringArg};
use derive_more::*;
use std::borrow::Cow;

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
// c_ulong is u64 on Linux, but on Windows u32. We don't want the consumer to deal with those
// toolchain differences and therefore choose u32. Rationale: The REAPER header files represent
// command IDs usually as c_int, which is basically always i32. Also makes sense ... why would
// someone need 2^64 commands!
pub struct CommandId(pub(crate) u32);

impl CommandId {
    /// Creates a command ID.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is 0 (which is not a valid command ID).
    pub fn new(value: u32) -> CommandId {
        assert_ne!(value, 0, "0 is not a valid command ID");
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

/// A section ID.
///
/// This uniquely identifies a [`section`].
///
/// [`section`]: struct.KbdSectionInfo.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
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

/// A MIDI input device ID.
///
/// This uniquely identifies a MIDI input device according to the REAPER MIDI device preferences.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct MidiInputDeviceId(pub(crate) u8);

impl MidiInputDeviceId {
    /// Creates the MIDI input device ID.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not a valid ID (must be <= 62).
    pub fn new(value: u8) -> MidiInputDeviceId {
        assert!(value < 63, "MIDI input device IDs must be <= 62");
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
/// Please note that this value is **not** normalized in the classical sense of being in the unit
/// interval 0.0..=1.0! It can be very well > 1.0 (e.g. the *Wet* param of *ReaPitch*). All this
/// type guarantees is that the value is > 0.0.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct ReaperNormalizedFxParamValue(pub(crate) f64);

impl ReaperNormalizedFxParamValue {
    /// The minimum possible value (0.0).
    pub const MIN: ReaperNormalizedFxParamValue = ReaperNormalizedFxParamValue(0.0);

    /// Creates a REAPER-normalized FX parameter value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is negative.
    pub fn new(value: f64) -> ReaperNormalizedFxParamValue {
        assert!(ReaperNormalizedFxParamValue::MIN.get() <= value);
        ReaperNormalizedFxParamValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a tempo measured in beats per minute.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct Bpm(pub(crate) f64);

impl Bpm {
    /// The minimum possible value (1.0 bpm).
    pub const MIN: Bpm = Bpm(1.0);

    /// The maximum possible value (960.0 bpm).
    pub const MAX: Bpm = Bpm(960.0);

    /// Creates a BPM value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the BPM range supported by REAPER
    /// `(1.0..=960.0)`.
    pub fn new(value: f64) -> Bpm {
        assert!(Bpm::MIN.get() <= value && value <= Bpm::MAX.get());
        Bpm(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a play rate measured as factor of the normal play speed.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct PlaybackSpeedFactor(pub(crate) f64);

impl PlaybackSpeedFactor {
    /// The minimum possible value (a quarter of the normal playback speed).
    pub const MIN: PlaybackSpeedFactor = PlaybackSpeedFactor(0.25);

    /// The normal playback speed.
    pub const NORMAL: PlaybackSpeedFactor = PlaybackSpeedFactor(1.00);

    /// The maximum possible value (four times the normal playback speed).
    pub const MAX: PlaybackSpeedFactor = PlaybackSpeedFactor(4.0);

    /// Creates a playback speed factor.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the playback speed range supported by
    /// REAPER `(0.25..=4.00)`.
    pub fn new(value: f64) -> PlaybackSpeedFactor {
        assert!(PlaybackSpeedFactor::MIN.get() <= value && value <= PlaybackSpeedFactor::MAX.get());
        PlaybackSpeedFactor(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a play rate measured as value between 0 and 1.
///
/// This corresponds to the position on the project play rate slider.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct NormalizedPlayRate(pub(crate) f64);

impl NormalizedPlayRate {
    /// The minimum possible value (a quarter of the normal play speed).
    pub const MIN: NormalizedPlayRate = NormalizedPlayRate(0.0);

    /// The normal playback speed.
    pub const NORMAL: NormalizedPlayRate = NormalizedPlayRate(0.2);

    /// The maximum possible value (four times the normal play speed).
    pub const MAX: NormalizedPlayRate = NormalizedPlayRate(1.0);

    /// Creates a normalized play rate.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within `(0.00..=1.00)`.
    pub fn new(value: f64) -> NormalizedPlayRate {
        assert!(NormalizedPlayRate::MIN.get() <= value && value <= NormalizedPlayRate::MAX.get());
        NormalizedPlayRate(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a frequency measured in hertz (how often something happens per second).
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct Hz(pub(crate) f64);

impl Hz {
    /// Creates a hertz value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value zero or negative.
    pub fn new(value: f64) -> Hz {
        assert!(0.0 < value);
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

/// This represents a volume measured in decibel.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
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

    /// Creates a decibel value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the decibel range supported by REAPER
    /// `(-1000.0..)`.
    pub fn new(value: f64) -> Db {
        assert!(Db::MIN.get() <= value || value.is_nan());
        Db(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a volume measured as fader position.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
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

    /// Creates a volume slider value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(0.0..)`.
    pub fn new(value: f64) -> VolumeSliderValue {
        assert!(VolumeSliderValue::MIN.get() <= value || value.is_nan());
        VolumeSliderValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// This represents a volume measured in REAPER's native volume unit.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
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

    /// Creates a REAPER volume value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(0.0..)`.
    pub fn new(value: f64) -> ReaperVolumeValue {
        assert!(ReaperVolumeValue::MIN.get() <= value || value.is_nan());
        ReaperVolumeValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
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

    /// Creates a pan value.
    ///
    /// # Panics
    ///
    /// This function panics if the given value is not within the range supported by REAPER
    /// `(-1.0..=1.0)`.
    pub fn new(value: f64) -> ReaperPanValue {
        assert!(ReaperPanValue::MIN.get() <= value && value <= ReaperPanValue::MAX.get());
        ReaperPanValue(value)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// For being able to use it with `ValueChange`.
#[doc(hidden)]
impl From<ReaperPanValue> for f64 {
    fn from(v: ReaperPanValue) -> Self {
        v.0
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
