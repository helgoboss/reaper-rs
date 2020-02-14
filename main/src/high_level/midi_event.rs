use crate::high_level::MidiInputDevice;
use wmidi::MidiMessage;

pub struct MidiEvent<'a> {
    frame_offset: i32,
    message: MidiMessage<'a>,
}

pub struct IncomingMidiEvent<'a> {
    device: MidiInputDevice,
    event: MidiEvent<'a>,
}