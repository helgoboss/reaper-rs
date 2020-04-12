use helgoboss_midi::Channel;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct MidiDeviceId(pub(super) u8);

// TODO Consider creating all newtypes with macros for more consistency and less code:
//  - https://gitlab.com/williamyaoh/shrinkwraprs
//  - https://github.com/JelteF/derive_more
//  - https://github.com/DanielKeep/rust-custom-derive
impl MidiDeviceId {
    /// Creates the MIDI device ID. Panics if the given number is not a valid ID.
    pub fn new(number: u8) -> MidiDeviceId {
        assert!(
            number != 63,
            "ID 63 is not a valid device ID because it represents all devices"
        );
        MidiDeviceId(number)
    }
}

impl From<MidiDeviceId> for u8 {
    fn from(id: MidiDeviceId) -> Self {
        id.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecordingInput {
    None,
    // TODO-low Audio inputs in detail
    //  record input, <0=no input, 0..n=mono hardware input, 512+n=rearoute input, &1024=stereo
    // input pair. &4096=MIDI input, if set then low 5 bits represent channel (0=all, 1-16=only
    // chan), next 6 bits represent physical input (63=all, 62=VKB)
    Mono,
    ReaRoute,
    Stereo,
    // TODO Don't make MidiRecordingInput an own type
    Midi(MidiRecordingInput),
}

impl RecordingInput {
    pub fn from_rec_input_index(rec_input_index: i32) -> RecordingInput {
        match rec_input_index {
            i if i < 0 => RecordingInput::None,
            i if i < 512 => RecordingInput::Mono,
            i if i < 1024 => RecordingInput::ReaRoute,
            i if i < 4096 => RecordingInput::Stereo,
            _ => RecordingInput::Midi(MidiRecordingInput::new(rec_input_index as u32)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MidiRecordingInput {
    rec_input_index: u32,
}

const ALL_MIDI_DEVICES_ID: u32 = 63;

impl MidiRecordingInput {
    fn new(rec_input_index: u32) -> Self {
        MidiRecordingInput { rec_input_index }
    }

    pub fn from_all_devices_and_channels() -> Self {
        Self::from_midi_rec_input_index(ALL_MIDI_DEVICES_ID * 32)
    }

    pub fn from_all_channels_of_device(device_id: MidiDeviceId) -> Self {
        Self::from_midi_rec_input_index(u8::from(device_id) as u32 * 32)
    }

    pub fn from_all_devices_with_channel(channel: u32) -> Self {
        Self::from_midi_rec_input_index(ALL_MIDI_DEVICES_ID * 32 + channel + 1)
    }

    pub fn from_device_and_channel(device_id: MidiDeviceId, channel: u32) -> Self {
        Self::from_midi_rec_input_index(u8::from(device_id) as u32 * 32 + channel + 1)
    }

    pub fn from_midi_rec_input_index(midi_rec_input_index: u32) -> Self {
        Self::new(4096 + midi_rec_input_index)
    }

    pub fn get_rec_input_index(&self) -> u32 {
        self.rec_input_index
    }

    pub fn get_midi_rec_input_index(&self) -> u32 {
        self.rec_input_index - 4096
    }

    // TODO-low In Rust get_ prefix is not idiomatic. On the other hand, the convention talks
    //  about exposing members only. Channel is not a member. However I also don't want to
    //  expose the information if it's a member or not. get_ has an advantage in IDEs and also
    //  prevents ambiguities if the noun can sound like a verb.
    pub fn get_channel(&self) -> Option<Channel> {
        let channel_id = self.get_midi_rec_input_index() % 32;
        if channel_id == 0 {
            return None;
        }
        let ch = channel_id - 1;
        ch.try_into().ok()
    }

    pub fn get_device_id(&self) -> Option<MidiDeviceId> {
        let raw_device_id = self.get_midi_rec_input_index() / 32;
        if raw_device_id == ALL_MIDI_DEVICES_ID {
            return None;
        }
        Some(MidiDeviceId::new(raw_device_id as u8))
    }
}
