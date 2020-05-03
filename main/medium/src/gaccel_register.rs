use crate::CommandId;
use reaper_rs_low::raw;
use reaper_rs_low::raw::gaccel_register_t;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_ushort;

/// A kind of action descriptor.
///
/// Contains action description, command ID and default key binding.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
// TODO-medium Try the shortcut thing
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MediumGaccelRegister {
    owned_desc: Cow<'static, CStr>,
    inner: gaccel_register_t,
}

/// An accelerator, that's a structure which contains the command ID of an action and default
/// shortcuts.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MediumAccel {
    inner: raw::ACCEL,
}

impl MediumGaccelRegister {
    /// Creates an action descriptor with the given accelerator and description.
    pub fn new(accel: MediumAccel, desc: Cow<'static, CStr>) -> MediumGaccelRegister {
        let desc_ptr = desc.as_ptr();
        MediumGaccelRegister {
            owned_desc: desc,
            inner: raw::gaccel_register_t {
                accel: *accel.as_ref(),
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

impl MediumAccel {
    /// Creates an accelerator.
    // TODO-medium Make the combination of f_virt and key strongly-typed!
    pub fn new(f_virt: u8, key: u16, cmd: CommandId) -> MediumAccel {
        MediumAccel {
            inner: raw::ACCEL {
                fVirt: f_virt,
                key,
                // TODO-low Why are REAPER command IDs u32? They need to fit into a u16 here!
                cmd: cmd.get() as c_ushort,
            },
        }
    }
}

impl AsRef<raw::ACCEL> for MediumAccel {
    fn as_ref(&self) -> &raw::ACCEL {
        &self.inner
    }
}
