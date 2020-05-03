use crate::MidiInputDeviceId;
use derive_more::*;
use helgoboss_midi::Channel;
use std::convert::{TryFrom, TryInto};

/// Recording input of a track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RecordingInput {
    // TODO-medium Check if those are really indexes!
    Mono(u32),
    ReaRoute(u32),
    Stereo(u32),
    Midi {
        device_id: Option<MidiInputDeviceId>,
        channel: Option<Channel>,
    },
}

impl RecordingInput {
    pub(crate) fn try_from_raw(
        rec_input_index: i32,
    ) -> Result<RecordingInput, RecInputIndexInvalid> {
        use RecordingInput::*;
        let rec_input_index = rec_input_index as u32;
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
            _ => Err(RecInputIndexInvalid),
        }
    }

    pub(crate) fn to_raw(&self) -> i32 {
        use RecordingInput::*;
        let result = match *self {
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
        };
        result as i32
    }
}

/// An error which can be returned when trying to interpret a recording input index.
#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "recording input index invalid")]
pub(crate) struct RecInputIndexInvalid;

const ALL_MIDI_DEVICES_FACTOR: u32 = 63;
