use crate::high_level::Reaper;
use std::ffi::CString;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MidiOutputDevice {
    id: u32,
}

impl MidiOutputDevice {
    pub fn new(id: u32) -> Self {
        MidiOutputDevice { id }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> CString {
        Reaper::get()
            .medium
            .get_midi_output_name(self.id, 33)
            .name
            .unwrap()
    }

    // For REAPER < 5.94 this is the same like isConnected(). For REAPER >=5.94 it returns true if
    // the device ever existed, even if it's disconnected now.
    pub fn is_available(&self) -> bool {
        let result = Reaper::get().medium.get_midi_output_name(self.id, 2);
        result.is_present || result.name.is_some()
    }

    // Only returns true if the device is connected (= present)
    pub fn is_connected(&self) -> bool {
        // In REAPER 5.94 GetMIDIOutputName doesn't accept nullptr as name buffer on OS X
        Reaper::get()
            .medium
            .get_midi_output_name(self.id, 1)
            .is_present
    }
}
