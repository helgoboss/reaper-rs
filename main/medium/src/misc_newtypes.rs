use derive_more::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct CommandId(pub(crate) u32);

impl CommandId {
    pub fn new(number: u32) -> CommandId {
        assert_ne!(number, 0, "0 is not a valid command ID");
        CommandId(number)
    }

    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl From<CommandId> for i32 {
    fn from(id: CommandId) -> Self {
        id.0 as i32
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct SectionId(pub(crate) u32);

impl SectionId {
    pub fn new(number: u32) -> SectionId {
        SectionId(number)
    }

    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl From<SectionId> for i32 {
    fn from(id: SectionId) -> Self {
        id.0 as i32
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct MidiInputDeviceId(pub(crate) u8);

// TODO-medium Consider creating all newtypes with macros for more consistency and less code:
//  - https://gitlab.com/williamyaoh/shrinkwraprs
//  - https://github.com/JelteF/derive_more
//  - https://github.com/DanielKeep/rust-custom-derive
impl MidiInputDeviceId {
    /// Creates the MIDI device ID. Panics if the given number is not a valid ID.
    pub fn new(number: u8) -> MidiInputDeviceId {
        assert!(number < 63, "MIDI device IDs must be <= 62");
        MidiInputDeviceId(number)
    }

    pub const fn get(&self) -> u8 {
        self.0
    }
}

impl From<MidiInputDeviceId> for i32 {
    fn from(id: MidiInputDeviceId) -> Self {
        id.0 as i32
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct MidiOutputDeviceId(pub(crate) u8);

impl MidiOutputDeviceId {
    /// Creates the MIDI device ID. Panics if the given number is not a valid ID.
    pub fn new(number: u8) -> MidiOutputDeviceId {
        MidiOutputDeviceId(number)
    }

    pub const fn get(&self) -> u8 {
        self.0
    }
}

impl From<MidiOutputDeviceId> for i32 {
    fn from(id: MidiOutputDeviceId) -> Self {
        id.0 as i32
    }
}

/// This value is **not** normalized in the classical sense of being in the unit interval 0.0..=1.0!
/// It can be > 1.0 (e.g. Wet param of ReaPitch).
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct ReaperNormalizedValue(pub(crate) f64);

impl ReaperNormalizedValue {
    pub const MIN: ReaperNormalizedValue = ReaperNormalizedValue(0.0);

    pub fn new(value: f64) -> ReaperNormalizedValue {
        assert!(ReaperNormalizedValue::MIN.get() <= value);
        ReaperNormalizedValue(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct Bpm(pub(crate) f64);

impl Bpm {
    pub const MIN: Bpm = Bpm(1.0);
    pub const MAX: Bpm = Bpm(960.0);

    pub fn new(value: f64) -> Bpm {
        assert!(Bpm::MIN.get() <= value && value <= Bpm::MAX.get());
        Bpm(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct PlaybackSpeedFactor(pub(crate) f64);

impl PlaybackSpeedFactor {
    pub const MIN: PlaybackSpeedFactor = PlaybackSpeedFactor(0.25);
    pub const MAX: PlaybackSpeedFactor = PlaybackSpeedFactor(4.0);

    pub fn new(value: f64) -> PlaybackSpeedFactor {
        assert!(PlaybackSpeedFactor::MIN.get() <= value && value <= PlaybackSpeedFactor::MAX.get());
        PlaybackSpeedFactor(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct Db(pub(crate) f64);

impl Db {
    /// Minimum value of this type. Corresponds to -inf dB. There's no maximum value because REAPER
    /// allows to exceed the soft maximum of 12 dB!
    pub const MIN: Db = Db(-1000.0);
    /// Don't know exactly what it means but this is a state possible in REAPER. Corresponds to 1.#R
    /// dB.
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);
    // -inf dB
    pub const MINUS_INF: Db = Db::MIN;
    // -150 dB
    pub const MINUS_150_DB: Db = Db(-150.0);
    // 0 dB
    pub const ZERO_DB: Db = Db(0.0);
    // 12 dB
    pub const TWELVE_DB: Db = Db(12.0);

    pub fn new(value: f64) -> Db {
        assert!(Db::MIN.get() <= value || value.is_nan());
        Db(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct VolumeSliderValue(pub(crate) f64);

impl VolumeSliderValue {
    /// Minimum value of this type. Corresponds to -inf dB. There's no maximum value because REAPER
    /// allows to exceed the soft maximum of 12 dB!
    pub const MIN: VolumeSliderValue = VolumeSliderValue(0.0);
    /// Don't know exactly what it means but this is a state possible in REAPER. Corresponds to 1.#R
    /// dB.
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);
    // -inf dB
    pub const MINUS_INF_DB: VolumeSliderValue = VolumeSliderValue::MIN;
    // -150 dB
    pub const MINUS_150_DB: VolumeSliderValue = VolumeSliderValue(2.5138729793972);
    // 0 dB
    pub const ZERO_DB: VolumeSliderValue = VolumeSliderValue(716.0);
    // 12 dB
    pub const TWELVE_DB: VolumeSliderValue = VolumeSliderValue(1000.0);

    pub fn new(value: f64) -> VolumeSliderValue {
        assert!(VolumeSliderValue::MIN.get() <= value || value.is_nan());
        VolumeSliderValue(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct ReaperVolumeValue(pub(crate) f64);

impl ReaperVolumeValue {
    /// Minimum value of this type. If the scale would be linear, this would be less than -150 dB.
    /// But it's not. In practice, REAPER considers this as equal to the MINUS_150_DB value.
    /// There's no maximum value because REAPER allows to exceed the soft maximum of 12 dB!
    pub const MIN: ReaperVolumeValue = ReaperVolumeValue(0.0);
    /// Don't know exactly what it means but this is a state possible in REAPER. Corresponds to 1.#R
    /// dB.
    pub const NAN: ReaperVolumeValue = ReaperVolumeValue(f64::NAN);
    /// Corresponds to -150 dB
    pub const MINUS_150_DB: ReaperVolumeValue = ReaperVolumeValue(3.1622776601684e-008);
    // Corresponds to 0 dB
    pub const ZERO_DB: ReaperVolumeValue = ReaperVolumeValue(1.0);
    // Corresponds to 12 dB
    pub const TWELVE_DB: ReaperVolumeValue = ReaperVolumeValue(3.981071705535);

    pub fn new(value: f64) -> ReaperVolumeValue {
        assert!(ReaperVolumeValue::MIN.get() <= value || value.is_nan());
        ReaperVolumeValue(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

impl From<ReaperVolumeValue> for f64 {
    fn from(v: ReaperVolumeValue) -> Self {
        v.0
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Display)]
pub struct ReaperPanValue(pub(crate) f64);

impl ReaperPanValue {
    pub const MIN: ReaperPanValue = ReaperPanValue(-1.0);
    pub const LEFT: ReaperPanValue = ReaperPanValue::MIN;
    pub const CENTER: ReaperPanValue = ReaperPanValue(0.0);
    pub const RIGHT: ReaperPanValue = ReaperPanValue::MAX;
    pub const MAX: ReaperPanValue = ReaperPanValue(1.0);

    pub fn new(value: f64) -> ReaperPanValue {
        assert!(ReaperPanValue::MIN.get() <= value && value <= ReaperPanValue::MAX.get());
        ReaperPanValue(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

impl From<ReaperPanValue> for f64 {
    fn from(v: ReaperPanValue) -> Self {
        v.0
    }
}
