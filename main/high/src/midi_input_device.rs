use crate::Reaper;
use reaper_medium::{MidiInput, MidiInputDeviceId, ReaperString};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct MidiInputDevice {
    id: MidiInputDeviceId,
}

impl MidiInputDevice {
    pub fn new(id: MidiInputDeviceId) -> Self {
        MidiInputDevice { id }
    }

    pub fn id(self) -> MidiInputDeviceId {
        self.id
    }

    pub fn name(self) -> ReaperString {
        Reaper::get()
            .medium_reaper()
            .get_midi_input_name(self.id, 64)
            .name
            .unwrap()
    }

    /// Must be called from real-time audio thread only!
    pub fn with_midi_input<R>(self, use_device: impl FnOnce(Option<&mut MidiInput>) -> R) -> R {
        Reaper::get()
            .medium_real_time_reaper
            .get_midi_input(self.id, use_device)
    }

    /// For REAPER < 5.94 this is the same like isConnected(). For REAPER >=5.94 it returns true if
    /// the device ever existed, even if it's disconnected now.
    pub fn is_available(self) -> bool {
        let result = Reaper::get()
            .medium_reaper()
            .get_midi_input_name(self.id, 2);
        result.is_present || result.name.is_some()
    }

    /// Returns true if the device is enabled and connected.
    pub fn is_open(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .get_midi_input_is_open(self.id)
    }

    /// Only returns true if the device is connected (= present)
    pub fn is_connected(self) -> bool {
        // In REAPER 5.94 GetMIDIInputName doesn't accept nullptr as name buffer on OS X
        Reaper::get()
            .medium_reaper()
            .get_midi_input_name(self.id, 1)
            .is_present
    }
}
