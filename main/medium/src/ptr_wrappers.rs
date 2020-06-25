//! This module makes low-level structs available in the medium-level API if necessary. This is done
//! using different strategies, depending on the characteristics of the struct. Sometimes it's just
//! a type alias, sometimes a wrapper.  
use crate::CommandId;

use reaper_low::raw;

use std::fmt;
use std::fmt::Debug;
use std::os::raw::c_void;
use std::ptr::NonNull;

/// Handle which is returned from some session functions that register something.
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
    reaper_ptr: NonNull<c_void>,
}

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
        Self {
            medium_ptr: self.medium_ptr,
            reaper_ptr: self.reaper_ptr,
        }
    }
}

impl<T> Copy for RegistrationHandle<T> {}

impl<T> RegistrationHandle<T> {
    pub(crate) fn new(
        medium_ptr: NonNull<T>,
        reaper_ptr: NonNull<c_void>,
    ) -> RegistrationHandle<T> {
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

    pub(crate) fn reaper_ptr(&self) -> NonNull<c_void> {
        self.reaper_ptr
    }
}

// Case 1: Internals exposed: no | vtable: no
// ==========================================

/// Pointer to a project.
pub type ReaProject = NonNull<raw::ReaProject>;
/// Pointer to a track in a project.
pub type MediaTrack = NonNull<raw::MediaTrack>;
/// Pointer to an item on a track.
pub type MediaItem = NonNull<raw::MediaItem>;
/// Pointer to a take in an item.
pub type MediaItemTake = NonNull<raw::MediaItem_Take>;
/// Pointer to an envelope on a track.
pub type TrackEnvelope = NonNull<raw::TrackEnvelope>;
/// Pointer to a window (window handle).
pub type Hwnd = NonNull<raw::HWND__>;
/// Pointer to a module/instance (module/instance handle).
pub type Hinstance = NonNull<c_void>;

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

pub(crate) fn require_non_null_panic<T>(ptr: *mut T) -> NonNull<T> {
    assert!(
        !ptr.is_null(),
        "Raw pointer expected to be not null but was null"
    );
    unsafe { NonNull::new_unchecked(ptr) }
}
