use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use crate::{DurationInSeconds, MidiFrameOffset, SendMidiTime};
use reaper_low::raw::MIDI_event_t;
use std::os::raw::c_int;
use std::ptr::NonNull;

/// Pointer to a PCM source.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PcmSource(pub(crate) NonNull<raw::PCM_source>);

impl PcmSource {
    /// Returns a pointer to the low-level PCM source.
    pub fn as_ptr(&self) -> *mut raw::PCM_source {
        self.0.as_ptr()
    }

    /// Returns the length of this source.
    pub unsafe fn get_length(&self) -> Option<DurationInSeconds> {
        let raw = self.0.as_ref();
        let length = raw.GetLength();
        if length < 0.0 {
            return None;
        }
        Some(DurationInSeconds::new(length))
    }
}
