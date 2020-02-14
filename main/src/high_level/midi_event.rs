use crate::high_level::MidiInputDevice;

pub struct MidiEvent {
    frame_offset: i32,
    message: i32,
}

pub struct IncomingMidiEvent {
    device: MidiInputDevice,
    event: MidiEvent,
}