use crate::CommandId;
use reaper_rs_low::raw;
use reaper_rs_low::raw::gaccel_register_t;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_ushort;

pub struct MediumGaccelRegister {
    owned_desc: Cow<'static, CStr>,
    inner: gaccel_register_t,
}

pub struct MediumAccelerator {
    inner: raw::ACCEL,
}

impl MediumGaccelRegister {
    pub fn new(accel: MediumAccelerator, desc: Cow<'static, CStr>) -> MediumGaccelRegister {
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

impl MediumAccelerator {
    // TODO-low Make the combination of f_virt and key strongly-typed!
    pub fn new(f_virt: u8, key: u16, cmd: CommandId) -> MediumAccelerator {
        MediumAccelerator {
            inner: raw::ACCEL {
                fVirt: f_virt,
                key,
                // TODO-low Why are REAPER command IDs u32? They need to fit into a u16 here!
                cmd: cmd.get() as c_ushort,
            },
        }
    }
}

impl AsRef<raw::ACCEL> for MediumAccelerator {
    fn as_ref(&self) -> &raw::ACCEL {
        &self.inner
    }
}
