use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use crate::util::create_passing_c_str;
use crate::{DurationInSeconds, MidiFrameOffset, ReaperStr, SendMidiTime};
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

    /// Returns if this source is available.
    pub unsafe fn is_available(&self) -> bool {
        self.0.as_ref().IsAvailable()
    }

    /// Duplicates this source.
    pub unsafe fn duplicate(&self) -> Option<PcmSource> {
        let raw_duplicate = self.0.as_ref().Duplicate();
        NonNull::new(raw_duplicate).map(PcmSource)
    }

    /// Grants temporary access to the type of this source.
    ///
    /// `None` is an invalid result but it could happen with 3rd-party source implementations.
    pub unsafe fn get_type<R>(&self, use_type: impl FnOnce(Option<&ReaperStr>) -> R) -> R {
        let ptr = self.0.as_ref().GetType();
        use_type(create_passing_c_str(ptr))
    }

    /// Returns the parent source, if any.
    pub unsafe fn get_source(&self) -> Option<Self> {
        let ptr = self.0.as_ref().GetSource();
        NonNull::new(ptr).map(Self)
    }

    /// Grants temporary access to the file name of this source.
    ///
    /// `None` is a valid result. In that case it's not purely a file.
    pub unsafe fn get_file_name<R>(
        &self,
        use_file_name: impl FnOnce(Option<&ReaperStr>) -> R,
    ) -> R {
        let ptr = self.0.as_ref().GetFileName();
        use_file_name(create_passing_c_str(ptr))
    }
}
