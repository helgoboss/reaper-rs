use reaper_rs_low::raw;
use std::ptr::{null_mut, NonNull};

pub fn option_non_null_into<T>(option: Option<NonNull<T>>) -> *mut T {
    match option {
        None => null_mut(),
        Some(v) => v.as_ptr(),
    }
}

pub fn require_non_null<T>(ptr: *mut T) -> Result<NonNull<T>, ()> {
    if ptr.is_null() {
        Err(())
    } else {
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }
}

pub fn require_non_null_panic<T>(ptr: *mut T) -> NonNull<T> {
    assert!(
        !ptr.is_null(),
        "Raw pointer expected to be not null but was null"
    );
    unsafe { NonNull::new_unchecked(ptr) }
}

// One of the responsibilities of the medium-level API is to use identifiers which follow the Rust
// conventions. It just happens that some of the C++ classes already conform to Rust conventions,
// so we won't rename them.
pub type ReaProject = NonNull<raw::ReaProject>;
pub type MediaTrack = NonNull<raw::MediaTrack>;
pub type MediaItem = NonNull<raw::MediaItem>;
pub type MediaItemTake = NonNull<raw::MediaItem_Take>;
pub type PcmSource = NonNull<raw::PCM_source>;
pub type TrackEnvelope = NonNull<raw::TrackEnvelope>;
// Even we create IReaperControlSurface instances ourselves (not REAPER), we don't do it on
// Rust side but on C++ side. So a pointer wrapper is the right way to go here as well. We also
// remove the I from the name because it's not following Rust conventions.
pub type ReaperControlSurface = NonNull<raw::IReaperControlSurface>;
// This is unlike MediaTrack and Co. in that it points to a struct which is *not* opaque. Still, we
// need it as pointer and it has the same lifetime characteristics. The difference is that we add
// type-safe methods to it to lift the possibilities in the struct to medium-level API style. This
// is similar to our midi_Input wrapper in low-level REAPER (just that it doesn't lift the API to
// medium-level API style but restores low-level functionality).
define_ptr_wrapper!(KbdSectionInfo, raw::KbdSectionInfo);
pub type Hwnd = NonNull<raw::HWND__>;

impl KbdSectionInfo {
    pub unsafe fn action_list_cnt(&self) -> u32 {
        (*self.0).action_list_cnt as u32
    }

    pub unsafe fn get_action_by_index<'a>(&'a self, index: u32) -> Option<KbdCmd<'a>> {
        let array =
            std::slice::from_raw_parts((*self.0).action_list, (*self.0).action_list_cnt as usize);
        let raw_kbd_cmd = array.get(index as usize)?;
        Some(KbdCmd(raw_kbd_cmd))
    }
}

// There's no point in using references with lifetime annotations in `KbdSectionInfo` because it is
// impossible to track their lifetimes. However, we can start using lifetime annotations for
// KbdCmd because its lifetime can be related to the lifetime of the `KbdSectionInfo`.
pub struct KbdCmd<'a>(pub(super) &'a raw::KbdCmd);

impl<'a> KbdCmd<'a> {
    pub fn cmd(&self) -> u32 {
        self.0.cmd
    }
}
