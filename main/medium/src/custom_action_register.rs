use crate::{AcceleratorBehavior, AcceleratorKeyCode, CommandId, ReaperStr, ReaperStringArg};
use enumflags2::BitFlags;
use reaper_low::raw;
use reaper_low::raw::custom_action_register_t;
use std::borrow::Cow;
use std::os::raw::c_ushort;

/// A kind of action descriptor.
///
/// Contains action description, command ID and default key binding.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct OwnedCustomActionRegister {
    owned_desc: Cow<'static, ReaperStr>,
    inner: custom_action_register_t,
}

impl OwnedCustomActionRegister {
    /// Creates an action descriptor without key binding.
    pub fn without_key_binding(
        desc: impl Into<ReaperStringArg<'static>>,
        section_id: i32,
    ) -> OwnedCustomActionRegister {
        let desc = desc.into().into_inner();
        let desc_ptr = desc.as_ptr();
        OwnedCustomActionRegister {
            owned_desc: desc,
            inner: raw::custom_action_register_t {
                uniqueSectionId: section_id,
                idStr: std::ptr::null(),
                name: desc_ptr,
                extra: std::ptr::null_mut(),
            },
        }
    }

    /// Creates an action descriptor with key binding.
    pub fn with_key_binding(
        desc: impl Into<ReaperStringArg<'static>>,
        section_id: i32,
    ) -> OwnedCustomActionRegister {
        let desc = desc.into().into_inner();
        let desc_ptr = desc.as_ptr();
        OwnedCustomActionRegister {
            owned_desc: desc,
            inner: raw::custom_action_register_t {
                uniqueSectionId: section_id,
                idStr: std::ptr::null(),
                name: desc_ptr,
                extra: std::ptr::null_mut(),
            },
        }
    }
}

impl AsRef<raw::custom_action_register_t> for OwnedCustomActionRegister {
    fn as_ref(&self) -> &custom_action_register_t {
        &self.inner
    }
}

