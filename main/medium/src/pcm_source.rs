use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use crate::util::{create_passing_c_str, with_string_buffer};
use crate::{
    DurationInSeconds, Hwnd, MediaItemTake, MidiFrameOffset, ReaperFunctionError,
    ReaperFunctionResult, ReaperStr, ReaperString, SendMidiTime,
};
use reaper_low::raw::MIDI_event_t;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::{null_mut, NonNull};

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
    /// # Errors
    ///
    /// Passes an error if this source doesn't return any type.
    pub unsafe fn get_type<R>(
        &self,
        use_type: impl FnOnce(ReaperFunctionResult<&ReaperStr>) -> R,
    ) -> R {
        let ptr = self.0.as_ref().GetType();
        let result = create_passing_c_str(ptr).ok_or(ReaperFunctionError::new("no type"));
        use_type(result)
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

    /// If this source represents pooled MIDI data, this will return information about it.
    ///
    /// # Errors
    ///
    /// Returns an error if not supported.
    pub unsafe fn ext_get_pooled_midi_id(&self) -> ReaperFunctionResult<ExtGetPooledMidiIdResult> {
        let mut user_count: MaybeUninit<i32> = MaybeUninit::zeroed();
        let mut first_user: MaybeUninit<*mut raw::MediaItem_Take> = MaybeUninit::zeroed();
        let (id, supported) = with_string_buffer(40, |buffer, max_size| {
            self.0.as_ref().Extended(
                raw::PCM_SOURCE_EXT_GETPOOLEDMIDIID as _,
                buffer as _,
                user_count.as_mut_ptr() as _,
                first_user.as_mut_ptr() as _,
            )
        });
        if supported == 0 {
            return Err(ReaperFunctionError::new(
                "PCM_SOURCE_EXT_GETPOOLEDMIDIID not supported by source",
            ));
        }
        Ok(ExtGetPooledMidiIdResult {
            id,
            // user_count: user_count.assume_init() as _,
            user_count: user_count.assume_init(),
            first_user: {
                let ptr = first_user.assume_init();
                NonNull::new(ptr).unwrap()
            },
        })
    }

    /// Writes the content of this source to the given file.
    ///
    /// Only currently supported by MIDI but in theory any source could support this.
    ///
    /// # Errors
    ///
    /// Returns an error if not supported.
    pub unsafe fn ext_export_to_file(&self, file_name: &Path) -> ReaperFunctionResult<()> {
        let file_name_str = file_name.to_str().expect("file name is not valid UTF-8");
        let file_name_reaper_string = ReaperString::from_str(file_name_str);
        let supported = self.0.as_ref().Extended(
            raw::PCM_SOURCE_EXT_EXPORTTOFILE as _,
            file_name_reaper_string.as_ptr() as _,
            null_mut(),
            null_mut(),
        );
        if supported == 0 {
            return Err(ReaperFunctionError::new(
                "PCM_SOURCE_EXT_EXPORTTOFILE not supported by source",
            ));
        }
        Ok(())
    }

    // /// Opens the editor for this source.
    // ///
    // /// # Errors
    // ///
    // /// Returns an error if not supported.
    // pub unsafe fn ext_open_editor(&self, hwnd: Hwnd, track_index: u32) ->
    // ReaperFunctionResult<()> {     let supported = self.0.as_ref().Extended(
    //         raw::PCM_SOURCE_EXT_OPENEDITOR as _,
    //         hwnd.as_ptr() as _,
    //         track_index as isize as _,
    //         null_mut(),
    //     );
    //     if supported == 0 {
    //         return Err(ReaperFunctionError::new(
    //             "PCM_SOURCE_EXT_OPENEDITOR not supported by source",
    //         ));
    //     }
    //     Ok(())
    // }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtGetPooledMidiIdResult {
    /// A GUID string with braces.
    // TODO-high Can this be empty?
    pub id: ReaperString,
    /// Number of takes which use this pooled MIDI data.
    // TODO-high Improve type
    pub user_count: i32,
    // TODO-high Can this be empty?
    pub first_user: MediaItemTake,
}
