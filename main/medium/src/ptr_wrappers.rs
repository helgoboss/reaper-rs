use crate::CommandId;
use derive_more::Into;
use reaper_rs_low::raw;
use std::convert::Into;
use std::marker::PhantomData;
use std::ptr::{null_mut, NonNull};

// # Internals exposed: no | vtable: no
//
// ## Strategy
//
// - Use NonNull pointers directly
// - Make them more accessible by using a public alias
//
// ## Explanation
//
// The following types are relevant to consumers, but only as pointers. Because those structs are
// completely opaque (internals not exposed, not even a vtable). We don't create a newtype because
// the NonNull guarantee is all we need and according to medium-level API design, we will never
// provide any methods on them (no vtable emulation, no convenience methods). Using a newtype just
// for reasons of symmetry would not be good because it comes with a cost (more code to write,
// less substitution possibilities) but no benefit.
pub type ReaProject = NonNull<raw::ReaProject>;
pub type MediaTrack = NonNull<raw::MediaTrack>;
pub type MediaItem = NonNull<raw::MediaItem>;
pub type MediaItemTake = NonNull<raw::MediaItem_Take>;
pub type TrackEnvelope = NonNull<raw::TrackEnvelope>;
pub type Hwnd = NonNull<raw::HWND__>;

// # Internals exposed: yes | vtable: no
//
// ## Strategy
//
// - Wrap NonNull pointer in a public newtype. This newtype should expose the internals in a way
//   which is idiomatic for Rust (like the rest of the medium-level API does).
// - If the consumer needs to be able to create such structs as well, create a new struct prefixed
//   with `Medium` which is completely owned and ideally wraps the the raw struct.

/// Represents an action description with a default key binding.
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct GaccelRegister(pub(crate) NonNull<raw::gaccel_register_t>);

impl GaccelRegister {
    /// C
    pub(crate) fn new(ptr: NonNull<raw::gaccel_register_t>) -> GaccelRegister {
        GaccelRegister(ptr)
    }

    pub(crate) fn get(&self) -> NonNull<raw::gaccel_register_t> {
        self.0
    }
}

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
// ALternative name: PcmSrc
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Into)]
pub struct PcmSource(pub(crate) NonNull<raw::PCM_source>);

impl PcmSource {
    pub fn new(ptr: NonNull<raw::PCM_source>) -> PcmSource {
        PcmSource(ptr)
    }

    pub(crate) fn get(&self) -> NonNull<raw::PCM_source> {
        self.0
    }
}

// # Internals exposed: no | vtable: yes (Rust <= REAPER)
//
// Even we create IReaperControlSurface instances ourselves (not REAPER), we don't do it on
// Rust side but on C++ side. So a pointer wrapper is the right way to go here as well. We also
// remove the I from the name because it's not following Rust conventions.
// ALternative name: ReaperControlSurf
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Into)]
pub struct ReaperControlSurface(pub(crate) NonNull<raw::IReaperControlSurface>);

impl ReaperControlSurface {
    pub fn new(ptr: NonNull<raw::IReaperControlSurface>) -> ReaperControlSurface {
        ReaperControlSurface(ptr)
    }

    pub(crate) fn get(&self) -> NonNull<raw::IReaperControlSurface> {
        self.0
    }
}

pub(crate) fn require_non_null<T>(ptr: *mut T) -> Result<NonNull<T>, ()> {
    if ptr.is_null() {
        Err(())
    } else {
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }
}

pub(crate) fn require_non_null_panic<T>(ptr: *mut T) -> NonNull<T> {
    assert!(
        !ptr.is_null(),
        "Raw pointer expected to be not null but was null"
    );
    unsafe { NonNull::new_unchecked(ptr) }
}
