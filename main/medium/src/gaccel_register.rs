use crate::{AcceleratorBehavior, AcceleratorKeyCode, CommandId, ReaperStr, ReaperStringArg};
use enumflags2::BitFlags;
use reaper_low::raw;
use reaper_low::raw::gaccel_register_t;
use std::borrow::Cow;
use std::os::raw::c_ushort;

/// A kind of action descriptor.
///
/// Contains action description, command ID and default key binding.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct OwnedGaccelRegister {
    owned_desc: Cow<'static, ReaperStr>,
    inner: gaccel_register_t,
}

impl OwnedGaccelRegister {
    /// Creates an action descriptor without key binding.
    pub fn without_key_binding(
        cmd: CommandId,
        desc: impl Into<ReaperStringArg<'static>>,
    ) -> OwnedGaccelRegister {
        let desc = desc.into().into_inner();
        let desc_ptr = desc.as_ptr();
        OwnedGaccelRegister {
            owned_desc: desc,
            inner: raw::gaccel_register_t {
                accel: raw::ACCEL {
                    fVirt: 0,
                    key: 0,
                    // TODO-low Why are REAPER command IDs u32? They need to fit into a u16 here!
                    cmd: cmd.get() as c_ushort,
                },
                desc: desc_ptr,
            },
        }
    }

    /// Creates an action descriptor with key binding.
    pub fn with_key_binding(
        cmd: CommandId,
        desc: impl Into<ReaperStringArg<'static>>,
        behavior: BitFlags<AcceleratorBehavior>,
        key_code: AcceleratorKeyCode,
    ) -> OwnedGaccelRegister {
        let desc = desc.into().into_inner();
        let desc_ptr = desc.as_ptr();
        OwnedGaccelRegister {
            owned_desc: desc,
            inner: raw::gaccel_register_t {
                accel: raw::ACCEL {
                    fVirt: behavior.bits(),
                    key: key_code.get(),
                    cmd: cmd.get() as c_ushort,
                },
                desc: desc_ptr,
            },
        }
    }
}

impl AsRef<raw::gaccel_register_t> for OwnedGaccelRegister {
    fn as_ref(&self) -> &gaccel_register_t {
        &self.inner
    }
}
