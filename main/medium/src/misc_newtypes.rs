use derive_more::*;

// TODO-medium 0 is used in some functions to represent "not found" and I think therefore not a
// valid command ID
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct CommandId(pub(crate) u32);

impl CommandId {
    pub fn new(number: u32) -> CommandId {
        assert_ne!(number, 0, "0 is not a valid command ID");
        CommandId(number)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<CommandId> for i32 {
    fn from(id: CommandId) -> Self {
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
}

impl MidiInputDeviceId {
    pub fn get(&self) -> u8 {
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
}

impl MidiOutputDeviceId {
    pub fn get(&self) -> u8 {
        self.0
    }
}

impl From<MidiOutputDeviceId> for i32 {
    fn from(id: MidiOutputDeviceId) -> Self {
        id.0 as i32
    }
}
