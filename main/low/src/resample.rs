#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_resample::*;
use crate::bindings::root::{REAPER_Resample_Interface, ReaSample};

impl REAPER_Resample_Interface {
    pub fn SetRates(&self, rate_in: f64, rate_out: f64) {
        unsafe {
            REAPER_Resample_Interface_SetRates(self as *const _ as _, rate_in, rate_out);
        }
    }

    pub fn Reset(&self) {
        unsafe {
            REAPER_Resample_Interface_Reset(self as *const _ as _);
        }
    }

    pub fn GetCurrentLatency(&self) -> f64 {
        unsafe { REAPER_Resample_Interface_GetCurrentLatency(self as *const _ as _) }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn ResamplePrepare(
        &self,
        out_samples: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
        inbuffer: *mut *mut ReaSample,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_ResamplePrepare(self as *const _ as _, out_samples, nch, inbuffer)
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn ResampleOut(
        &self,
        out: *mut ReaSample,
        nsamples_in: ::std::os::raw::c_int,
        nsamples_out: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_ResampleOut(
            self as *const _ as _,
            out,
            nsamples_in,
            nsamples_out,
            nch,
        )
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn Extended(
        &self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_Extended(self as *const _ as _, call, parm1, parm2, parm3)
    }
}
