//! This module makes low-level structs available in the medium-level API if necessary. This is done
//! using different strategies, depending on the characteristics of the struct. Sometimes it's just
//! a type alias, sometimes a wrapper.  
use crate::{CommandId, SectionId};
use std::cmp::Ordering;

use reaper_low::raw;

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::os::raw::c_void;
use std::ptr::NonNull;

/// A simple opaque handle to something registered within REAPER.
///
/// Characteristics:
///
/// - Carries enough information to be able to unregister the register thing (passed to the correct
///   function).
/// - Carries type information so that this handle cannot just be passed to an "unregister" function
///   that's intended for unregistering a different type of thing.
/// - Has ID character: Is small, copyable and can be passed around freely, even if the type
///   parameter doesn't exhibit these properties.
/// - Its internals are hidden in order to allow non-breaking changes under the hood.
pub struct Handle<T>(NonNull<T>);

impl<T> Handle<T> {
    pub(crate) const fn new(inner: NonNull<T>) -> Self {
        Self(inner)
    }

    pub(crate) const fn get(self) -> NonNull<T> {
        self.0
    }

    pub(crate) const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    pub(crate) const fn cast<U>(self) -> Handle<U> {
        Handle(self.0.cast())
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

/// A more advanced handle which is returned from some session functions that register something.
///
/// This handle can be used to explicitly unregister the registered object and regain ownership of
/// the struct which has been passed in originally.
#[derive(Eq, PartialEq, Hash)]
pub struct RegistrationHandle<T> {
    /// (Thin) pointer for restoring the value stored in the session as its original type.
    ///
    /// In theory the stored trait object itself (`Box<dyn ...>`>) plus the generic parameter `T`
    /// would provide enough information to restore the value as its original type. But a trait
    /// object is stored as fat pointer and there's currently no stable way to cast a fat pointer
    /// back to a thin pointer, even we know the concrete type. That's why we also store the thin
    /// pointer here, which we have access to before casting to a trait object.
    medium_ptr: NonNull<T>,
    /// (Thin) pointer for unregistering the thing that has been passed to REAPER.
    reaper_ptr: Handle<c_void>,
}

// We might run into situations when it's necessary to promise Rust that passing handles to other
// threads is okay. And it is because methods which dereference the pointers are either unsafe or
// do a main thread check first.
unsafe impl<T> Send for RegistrationHandle<T> {}
unsafe impl<T> Sync for RegistrationHandle<T> {}

impl<T> Debug for RegistrationHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistrationHandle")
            .field("medium_ptr", &self.medium_ptr)
            .field("reaper_ptr", &self.reaper_ptr)
            .finish()
    }
}

impl<T> Clone for RegistrationHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RegistrationHandle<T> {}

impl<T> RegistrationHandle<T> {
    pub(crate) fn new(medium_ptr: NonNull<T>, reaper_ptr: Handle<c_void>) -> RegistrationHandle<T> {
        RegistrationHandle {
            medium_ptr,
            reaper_ptr,
        }
    }

    /// Restores the value as its original type and makes it owned by putting it into a box.
    ///
    /// Make sure you have leaked the other box after having taken it out from its storage.
    /// Otherwise there will be a double drop.
    pub(crate) unsafe fn restore_original(&self) -> Box<T> {
        Box::from_raw(self.medium_ptr.as_ptr())
    }

    pub(crate) fn reaper_handle(&self) -> Handle<c_void> {
        self.reaper_ptr
    }
}

// Case 1: Internals exposed: no | vtable: no
// ==========================================

macro_rules! ptr_wrapper {
    ($( #[doc = $doc:expr] )* $name:ident ($inner:ty)) => {
        $( #[doc = $doc] )*
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        pub struct $name(NonNull<$inner>);

        impl $name {
            pub fn new(ptr: *mut $inner) -> Option<Self> {
                NonNull::new(ptr).map(Self)
            }

            pub const fn as_ptr(self) -> *mut $inner {
                self.0.as_ptr()
            }
        }

        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}
    };
}

ptr_wrapper! {
    /// Pointer to a project.
    ReaProject(raw::ReaProject)
}

ptr_wrapper! {
    /// Pointer to a track in a project.
    MediaTrack(raw::MediaTrack)
}

ptr_wrapper! {
    /// Pointer to an item on a track.
    MediaItem(raw::MediaItem)
}

ptr_wrapper! {
    /// Pointer to a take in an item.
    MediaItemTake(raw::MediaItem_Take)
}

ptr_wrapper! {
/// Pointer to an envelope on a track.
    TrackEnvelope(raw::TrackEnvelope)
}

ptr_wrapper! {
    /// Pointer to a window (window handle).
    Hwnd(raw::HWND__)
}

ptr_wrapper! {
    /// Pointer to a brush.
    Hbrush(raw::HGDIOBJ__)
}

ptr_wrapper! {
    /// Pointer to a device context.
    Hdc(raw::HDC__)
}

ptr_wrapper! {
    /// Pointer to a menu (menu handle).
    Hmenu(raw::HMENU__)
}

ptr_wrapper! {
    /// Pointer to a module/instance (module/instance handle).
    Hinstance(c_void)
}

// Case 2: Internals exposed: yes | vtable: no
// ===========================================

/// Pointer to a section (in which actions can be registered).
///
/// One example of this is the REAPER main section which contains most of REAPER's actions.
//
// It's important that this can't be cloned or copied! Unlike MediaTrack and Co. we have a a
// function section_from_unique_id() which doesn't require unsafe code because it passes a
// guaranteed valid `&KbdSectionInfo` to a user-defined closure. The referred object
// (`KbdSectionInfo`) *must not* be copied, otherwise it would be possible to let the
// `KbdSectionInfo` escape the closure - without any unsafe code! Validity could not be guaranteed
// anymore.
//
// So if we just use it as reference, why don't we wrap a *reference* of `raw::KbdSectionInfo` in
// the first place? Then it would be clear that this type is borrow-only! Problem: We actually have
// an unsafe function that returns this type directly (not as a reference). It's marked as unsafe
// because returning it means that Rust loses control and consumers have to make sure themselves
// that the lifetime is long enough. Now, if this wrapper would wrap a reference instead of a raw
// pointer, we wouldn't be able to return a value at all because we can't return a reference created
// in a function. Besides, we wouldn't be able to give that reference a correct lifetime annotation.
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct KbdSectionInfo(pub(crate) NonNull<raw::KbdSectionInfo>);

impl KbdSectionInfo {
    /// Returns the number of actions in this section.
    pub fn action_list_cnt(&self) -> u32 {
        unsafe { self.0.as_ref() }.action_list_cnt as u32
    }

    /// Returns the unique ID of this section.
    pub fn unique_id(&self) -> SectionId {
        let raw = unsafe { self.0.as_ref().uniqueID };
        SectionId::new(raw as _)
    }

    /// Returns the action at the specified index.
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

    /// Returns the raw pointer.
    pub fn raw(&self) -> NonNull<raw::KbdSectionInfo> {
        self.0
    }
}

/// Borrowed action.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KbdCmd<'a>(pub(crate) &'a raw::KbdCmd);

impl<'a> KbdCmd<'a> {
    /// Returns the command ID of this action.
    pub fn cmd(self) -> CommandId {
        CommandId(self.0.cmd as _)
    }
}

pub(crate) fn require_media_track_panic(ptr: *mut raw::MediaTrack) -> MediaTrack {
    MediaTrack::new(ptr).expect("Raw MediaTrack expected to be not null but was null")
}

pub(crate) fn require_hwnd_panic(ptr: *mut raw::HWND__) -> Hwnd {
    Hwnd::new(ptr).expect("Raw HWND expected to be not null but was null")
}

// Case 3: Internals exposed: no | vtable: yes
// ===========================================

/// Pointer to a PCM sink.
pub type PcmSink = NonNull<raw::PCM_sink>;

/// Pointer to a PCM source.
pub type PcmSource = NonNull<raw::PCM_source>;

/// Pointer to a project state context.
pub type ProjectStateContext = NonNull<raw::ProjectStateContext>;

/// Pointer to a REAPER pitch shift instance.
pub type ReaperPitchShift = NonNull<raw::IReaperPitchShift>;

/// Pointer to a REAPER resample instance.
pub type ReaperResample = NonNull<raw::REAPER_Resample_Interface>;
