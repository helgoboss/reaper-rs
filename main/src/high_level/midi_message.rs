use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryInto;

type Byte = u8;
type Nibble = u8;

pub trait MidiMessage {
    fn get_status_byte(&self) -> Byte;

    fn get_data_byte_1(&self) -> Byte;

    fn get_data_byte_2(&self) -> Byte;

    fn get_type(&self) -> MidiMessageType {
        let status_byte = self.get_status_byte();
        let high_status_byte_nibble = extract_high_nibble_from_byte(status_byte);
        if high_status_byte_nibble == 0xf {
            // System message. The complete status byte makes up the type.
            status_byte.try_into().expect("Unknown system message status byte")
        } else {
            // Channel message. Just the high nibble of the status byte makes up the type
            // (low nibble encodes channel).
            build_byte_from_nibbles(high_status_byte_nibble, 0).try_into()
                .expect("Unknown channel message nibble")
        }
    }

    // Returns false if the message type is NoteOn but the velocity is 0
    fn is_note_on(&self) -> bool {
        self.get_type() == MidiMessageType::NoteOn && self.get_velocity() > 0
    }

    fn get_velocity(&self) -> Nibble {
        self.get_data_byte_2()
    }
}

// The most low-level type of a MIDI message
#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MidiMessageType {
    // Channel messages = channel voice messages + channel mode messages (given value represents channel 0 status byte)
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyphonicKeyPressure = 0xa0,
    ControlChange = 0xb0,
    ProgramChange = 0xc0,
    ChannelPressure = 0xd0,
    PitchBendChange = 0xe0,
    // System exclusive messages
    SystemExclusiveStart = 0xf0,
    // System common messages
    MidiTimeCodeQuarterFrame = 0xf1,
    SongPositionPointer = 0xf2,
    SongSelect = 0xf3,
    TuneRequest = 0xf6,
    SystemExclusiveEnd = 0xf7,
    // System real-time messages (given value represents the complete status byte)
    TimingClock = 0xf8,
    Start = 0xfa,
    Continue = 0xfb,
    Stop = 0xfc,
    ActiveSensing = 0xfe,
    SystemReset = 0xff
}

fn extract_high_nibble_from_byte(byte: Byte) -> Nibble {
    (byte >> 4) & 0x0f
}

fn build_byte_from_nibbles(high_nibble: Nibble, low_nibble: Nibble) -> Byte {
    debug_assert!(high_nibble <= 0xf);
    debug_assert!(low_nibble <= 0xf);
    (high_nibble << 4) |low_nibble
}