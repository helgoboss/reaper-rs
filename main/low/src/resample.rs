#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_resample::*;
use crate::bindings::root::{REAPER_Resample_Interface, ReaSample};
use crate::raw;
use std::ptr::NonNull;

impl REAPER_Resample_Interface {
    pub fn SetRates(&mut self, rate_in: f64, rate_out: f64) {
        unsafe {
            REAPER_Resample_Interface_SetRates(self as _, rate_in, rate_out);
        }
    }

    pub fn Reset(&mut self) {
        unsafe {
            REAPER_Resample_Interface_Reset(self as _);
        }
    }

    pub fn GetCurrentLatency(&mut self) -> f64 {
        unsafe { REAPER_Resample_Interface_GetCurrentLatency(self as _) }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn ResamplePrepare(
        &mut self,
        out_samples: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
        inbuffer: *mut *mut ReaSample,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_ResamplePrepare(self as _, out_samples, nch, inbuffer)
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn ResampleOut(
        &mut self,
        out: *mut ReaSample,
        nsamples_in: ::std::os::raw::c_int,
        nsamples_out: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_ResampleOut(self as _, out, nsamples_in, nsamples_out, nch)
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn Extended(
        &mut self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        REAPER_Resample_Interface_Extended(self as _, call, parm1, parm2, parm3)
    }
}

/// Destroys a C++ `REAPER_Resample_Interface` object.
///
/// Intended to be used on pointers returned by [`Resampler_Create()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`Resampler_Create()`]: struct.Reaper.html#method.Resampler_Create
pub unsafe fn delete_cpp_reaper_resample_interface(
    resample_interface: NonNull<raw::REAPER_Resample_Interface>,
) {
    crate::bindings::root::reaper_resample::delete_reaper_resample_interface(
        resample_interface.as_ptr(),
    );
}
