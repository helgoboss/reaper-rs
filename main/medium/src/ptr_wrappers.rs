//! We obtain many pointers directly from REAPER and we can't
//! give them a sane lifetime annotation. They are "rather" static from the perspective of the
//! plug-in, yet they could come and go anytime, so 'static would be too optimistic. Annotating
//! with a lifetime 'a - correlated to another lifetime - would be impossible because we
//! don't have such another lifetime which can serve as frame of reference. So the best we
//! can do is wrapping pointers. For all opaque structs we do that simply by creating a type alias
//! to NonNull because NonNull maintains all the invariants we need (pointer not null) and opaque
//! structs don't have methods which need to be lifted to medium-level API style. For non-opaque
//! structs we wrap the NonNull in a newtype because we need to add medium-level API style methods.
use crate::CommandId;
use derive_more::Into;
use reaper_rs_low::raw;
use std::convert::Into;
use std::ptr::{null_mut, NonNull};

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

// # Internals exposed: no | vtable: no
//
// Strategy: Give pointer an idiomatic name
//
// The following types are relevant for us as pointers only because those structs are completely
// opaque (internals not exposed, not even a vtable). We don't give them a "Handle" suffix because
// we know there will probably never be a non-handle version of this! We don't create a newtype
// because the NonNull guarantee is all we need and according to medium-level API design, we will
// never provide any methods on them (no vtable, no convenience methods). Using a newtype just for
// reasons of symmetry would not be good because it would come with a cost (more code to write, less
// substitution possibilities) but no benefit.
pub type ReaProject = NonNull<raw::ReaProject>;
pub type MediaTrack = NonNull<raw::MediaTrack>;
pub type MediaItem = NonNull<raw::MediaItem>;
pub type MediaItemTake = NonNull<raw::MediaItem_Take>;
pub type TrackEnvelope = NonNull<raw::TrackEnvelope>;
pub type Hwnd = NonNull<raw::HWND__>;

// Internals exposed: yes | vtable: no
//
// Strategy: Create an idiomatic struct for creating by consumer & wrap pointer in a
// "Handle"-suffixed newtype (this newtype should expose the internals in an idiomatic way, this
// pointer is for direction Rust => REAPER while the idiomatic struct is for REAPER => Rust
// communication)
//
// TODO-medium Make newtypes already because we might expose internals in better way in future
// ALternative name: GaccelReg
pub type GaccelRegister = NonNull<raw::gaccel_register_t>;
// ALternative name: AudioHookReg
pub type AudioHookRegister = NonNull<raw::audio_hook_register_t>;
// This is unlike MediaTrack and Co. in that it points to a struct which is *not* opaque. Still, we
// need it as pointer and it has the same lifetime characteristics. The difference is that we add
// type-safe methods to it to lift the possibilities in the struct to medium-level API style. This
// is similar to our midi_Input wrapper in low-level REAPER (just that latter doesn't lift the API
// to medium-level API style but restores low-level functionality).
// It's important that this can't be cloned or copied! Unlike MediaTrack and Co. we have a
// a function section_from_unique_id() which doesn't require unsafe code because it
// passes a guaranteed valid &KbdSectionInfo to a user-defined closure. The referred object
// (KbdSectionInfo)  *must not* be copied, otherwise it's possible to let the KbdSectionInfo escape
// the closure - without any unsafe code! Validity can not be guaranteed anymore.
// Why don't we wrap a reference of raw::KbdSectionInfo? Then it would be clear that this type is
// borrow-only. Problem: If we return this type (from an unsafe function), we would have to assign a
// lifetime annotation - and whatever we would assign, it would be misleading because incorrect.
// But there's an even better reason: We wouldn't be able to return a value at all because we can't
// return a reference created in a function. We would need a separate owned version - which brings
// us back to the pointer wrapper. Having a pointer wrapper allows us to also return (e.g. for
// unsafe usage). Having a reference wrapper prevents this totally, that's not good.
// ALternative name: KbdSecInf
#[derive(Debug, Eq, Hash, PartialEq, Into)]
pub struct KbdSectionInfo(pub(crate) NonNull<raw::KbdSectionInfo>);
impl KbdSectionInfo {
    pub fn action_list_cnt(&self) -> u32 {
        unsafe { self.0.as_ref() }.action_list_cnt as u32
    }

    pub fn get_action_by_index(&self, index: u32) -> Option<KbdCmd<'_>> {
        let array = unsafe {
            std::slice::from_raw_parts(
                self.0.as_ref().action_list,
                self.0.as_ref().action_list_cnt as usize,
            )
        };
        let raw_kbd_cmd = array.get(index as usize)?;
        Some(KbdCmd(raw_kbd_cmd))
    }
}
// ALternative name: ???
pub struct KbdCmd<'a>(pub(crate) &'a raw::KbdCmd);
impl<'a> KbdCmd<'a> {
    pub fn cmd(&self) -> CommandId {
        CommandId(self.0.cmd)
    }
}

// # Internals exposed: no | vtable: yes (Rust <=> REAPER)
//
// Strategy: Create an idiomatic struct & wrap pointer in a "Handle"-suffixed newtype (this newtype
// should expose the vtable in an idiomatic way, this pointer is for direction Rust => REAPER while
// the idiomatic struct is for REAPER => Rust communication)
//
// TODO-medium Make newtypes already because we might expose vtable in future
// ALternative name: PcmSrc
pub type PcmSource = NonNull<raw::PCM_source>;

// # Internals exposed: no | vtable: yes (Rust <= REAPER)
//
// Even we create IReaperControlSurface instances ourselves (not REAPER), we don't do it on
// Rust side but on C++ side. So a pointer wrapper is the right way to go here as well. We also
// remove the I from the name because it's not following Rust conventions.
// TODO-medium Make newtypes already because we might expose vtable in future
// ALternative name: ReaperControlSurf
pub type ReaperControlSurface = NonNull<raw::IReaperControlSurface>;
