use crate::high_level::midi_input_device::MidiInputDevice;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecordingInput {
    None,
    // TODO Audio inputs in detail
    Mono,
    ReaRoute,
    Stereo,
    Midi(MidiRecordingInput),
}

impl RecordingInput {
    pub fn from_rec_input_index(rec_input_index: i32) -> RecordingInput {
        match rec_input_index {
            i if i < 0 => RecordingInput::None,
            i if i < 512 => RecordingInput::Mono,
            i if i < 1024 => RecordingInput::ReaRoute,
            i if i < 4096 => RecordingInput::Stereo,
            _ => RecordingInput::Midi(MidiRecordingInput::new(rec_input_index)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MidiRecordingInput {
    // TODO Make u32?
    rec_input_index: i32
}

impl MidiRecordingInput {
    fn new(rec_input_index: i32) -> Self {
        MidiRecordingInput {
            rec_input_index
        }
    }

    // TODO Maybe use of_ prefix instead
    pub fn from_all_devices_and_channels() -> Self {
        Self::from_midi_rec_input_index(63 * 32)
    }

    pub fn from_all_channels_of_device(device: MidiInputDevice) -> Self {
        Self::from_midi_rec_input_index(device.get_id() * 32)
    }

    pub fn from_all_devices_with_channel(channel: u32) -> Self {
        Self::from_device_and_channel(MidiInputDevice::new(63), channel)
    }

    pub fn from_device_and_channel(device: MidiInputDevice, channel: u32) -> Self {
        Self::from_midi_rec_input_index(device.get_id() * 32 + (channel as i32) + 1)
    }

    pub fn from_midi_rec_input_index(midi_rec_input_index: i32) -> Self {
        Self::new(4096 + midi_rec_input_index)
    }

    pub fn get_rec_input_index(&self) -> i32 {
        self.rec_input_index
    }

    pub fn get_midi_rec_input_index(&self) -> i32 {
        self.rec_input_index - 4096
    }

    pub fn get_channel(&self) -> Option<u32> {
        let channel_id = self.get_midi_rec_input_index() % 32;
        if channel_id == 0 {
            None
        } else {
            Some(channel_id as u32 - 1)
        }
    }

    pub fn get_device(&self) -> Option<MidiInputDevice> {
        let raw_device_id = self.get_midi_rec_input_index() / 32;
        if raw_device_id == 63 {
            None
        } else {
            Some(MidiInputDevice::new(raw_device_id))
        }
    }
}