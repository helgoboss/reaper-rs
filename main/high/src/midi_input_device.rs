use reaper_medium::{MidiInputDeviceId, ReaperFunctions};
use std::ffi::CString;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MidiInputDevice {
    id: MidiInputDeviceId,
}

impl MidiInputDevice {
    pub fn new(id: MidiInputDeviceId) -> Self {
        MidiInputDevice { id }
    }

    pub fn get_id(self) -> MidiInputDeviceId {
        self.id
    }

    pub fn get_name(self) -> CString {
        ReaperFunctions::get()
            .get_midi_input_name(self.id, 33)
            .name
            .unwrap()
    }

    // For REAPER < 5.94 this is the same like isConnected(). For REAPER >=5.94 it returns true if
    // the device ever existed, even if it's disconnected now.
    pub fn is_available(self) -> bool {
        let result = ReaperFunctions::get().get_midi_input_name(self.id, 2);
        result.is_present || result.name.is_some()
    }

    // Only returns true if the device is connected (= present)
    pub fn is_connected(self) -> bool {
        // In REAPER 5.94 GetMIDIInputName doesn't accept nullptr as name buffer on OS X
        ReaperFunctions::get()
            .get_midi_input_name(self.id, 1)
            .is_present
    }
}
