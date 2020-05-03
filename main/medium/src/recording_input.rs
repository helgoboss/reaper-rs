use crate::MidiInputDeviceId;

use helgoboss_midi::Channel;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecordingInput {
    Mono(u32),
    ReaRoute(u32),
    Stereo(u32),
    Midi {
        device_id: Option<MidiInputDeviceId>,
        channel: Option<Channel>,
    },
}

impl From<RecordingInput> for u32 {
    fn from(input: RecordingInput) -> Self {
        use RecordingInput::*;
        match input {
            Mono(i) => i,
            ReaRoute(i) => 512 + i,
            Stereo(i) => 1024 + i,
            Midi { device_id, channel } => {
                let device_high = match device_id {
                    None => ALL_MIDI_DEVICES_FACTOR,
                    Some(id) => id.get() as u32,
                };
                let channel_low = match channel {
                    None => 0,
                    Some(ch) => u32::from(ch) + 1,
                };
                4096 + (device_high * 32 + channel_low)
            }
        }
    }
}

impl TryFrom<u32> for RecordingInput {
    type Error = ();

    fn try_from(rec_input_index: u32) -> Result<Self, Self::Error> {
        use RecordingInput::*;
        match rec_input_index {
            0..=511 => Ok(Mono(rec_input_index)),
            512..=1023 => Ok(ReaRoute(rec_input_index - 512)),
            1024..=4095 => Ok(Stereo(rec_input_index - 1024)),
            4096..=6128 => {
                let midi_index = rec_input_index - 4096;
                Ok(Midi {
                    device_id: {
                        let raw_device_id = midi_index / 32;
                        if raw_device_id == ALL_MIDI_DEVICES_FACTOR {
                            None
                        } else {
                            Some(MidiInputDeviceId::new(raw_device_id as u8))
                        }
                    },
                    channel: {
                        let channel_id = midi_index % 32;
                        if channel_id == 0 {
                            None
                        } else {
                            let ch = channel_id - 1;
                            ch.try_into().ok()
                        }
                    },
                })
            }
            _ => Err(()),
        }
    }
}

const ALL_MIDI_DEVICES_FACTOR: u32 = 63;
