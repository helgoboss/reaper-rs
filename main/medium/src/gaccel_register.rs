use crate::CommandId;
use reaper_rs_low::raw;
use reaper_rs_low::raw::gaccel_register_t;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_ushort;

pub struct GaccelRegister2 {
    owned_desc: Cow<'static, CStr>,
    inner: gaccel_register_t,
}

pub struct Accelerator {
    inner: raw::ACCEL,
}

impl GaccelRegister2 {
    pub fn new(accel: Accelerator, desc: Cow<'static, CStr>) -> GaccelRegister2 {
        let desc_ptr = desc.as_ptr();
        GaccelRegister2 {
            owned_desc: desc,
            inner: raw::gaccel_register_t {
                accel: *accel.as_ref(),
                desc: desc_ptr,
            },
        }
    }
}

impl AsRef<raw::gaccel_register_t> for GaccelRegister2 {
    fn as_ref(&self) -> &gaccel_register_t {
        &self.inner
    }
}

impl Accelerator {
    // TODO-low Make the combination of f_virt and key strongly-typed!
    pub fn new(f_virt: u8, key: u16, cmd: CommandId) -> Accelerator {
        Accelerator {
            inner: raw::ACCEL {
                fVirt: f_virt,
                key,
                // TODO-low Why are REAPER command IDs u32? They need to fit into a u16 here!
                cmd: cmd.get() as c_ushort,
            },
        }
    }
}

impl AsRef<raw::ACCEL> for Accelerator {
    fn as_ref(&self) -> &raw::ACCEL {
        &self.inner
    }
}
