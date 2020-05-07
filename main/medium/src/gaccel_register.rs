use crate::{CommandId, ReaperStringArg};
use reaper_low::raw;
use reaper_low::raw::gaccel_register_t;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_ushort;

/// A kind of action descriptor.
///
/// Contains action description, command ID and default key binding.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MediumGaccelRegister {
    owned_desc: Cow<'static, CStr>,
    inner: gaccel_register_t,
}

impl MediumGaccelRegister {
    /// Creates an action descriptor without key binding.
    pub fn without_key_binding(
        cmd: CommandId,
        desc: impl Into<ReaperStringArg<'static>>,
    ) -> MediumGaccelRegister {
        let desc = desc.into().into_inner();
        let desc_ptr = desc.as_ptr();
        MediumGaccelRegister {
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
}

impl AsRef<raw::gaccel_register_t> for MediumGaccelRegister {
    fn as_ref(&self) -> &gaccel_register_t {
        &self.inner
    }
}
