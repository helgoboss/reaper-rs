use crate::{MidiInputDeviceId, TryFromRawError};

use helgoboss_midi::Channel;
use std::convert::{TryInto};

/// Recording input of a track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RecordingInput {
    /// Index refers to a single mono channel.
    Mono(u32),
    /// Index refers to a single ReaRoute mono channel.
    MonoReaRoute(u32),
    /// Index refers to the first of two channels in a stereo channel pair.
    Stereo(u32),
    /// Index refers to the first of two channels in a ReaRoute stereo channel pair.
    StereoReaRoute(u32),
    Midi {
        device_id: Option<MidiInputDeviceId>,
        channel: Option<Channel>,
    },
}

impl RecordingInput {
    /// Converts an integer as returned by the low-level API to a recording input.
    ///
    /// # Errors
    ///
    /// Fails if the given integer is not a valid recording input index.
    pub fn try_from_raw(rec_input_index: i32) -> Result<RecordingInput, TryFromRawError<i32>> {
        use RecordingInput::*;
        let v: u32 = rec_input_index.try_into().map_err(|_| {
            TryFromRawError::new("couldn't convert to recording input", rec_input_index)
        })?;
        match v {
            0..=511 => Ok(Mono(v)),
            512..=1023 => Ok(MonoReaRoute(v - 512)),
            1024..=1535 => Ok(Stereo(v - 1024)),
            1536..=2047 => Ok(StereoReaRoute(v - 1536)),
            2048..=4095 => Err(TryFromRawError::new(
                "couldn't convert to recording input",
                rec_input_index,
            )),
            4096..=6128 => {
                let midi_index = v - 4096;
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
            _ => Err(TryFromRawError::new(
                "couldn't convert to recording input",
                rec_input_index,
            )),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(&self) -> i32 {
        use RecordingInput::*;
        let result = match *self {
            Mono(i) => i,
            MonoReaRoute(i) => 512 + i,
            Stereo(i) => 1024 + i,
            StereoReaRoute(i) => 1536 + i,
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

const ALL_MIDI_DEVICES_FACTOR: u32 = 63;
