#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pitch_shift::*;
use crate::bindings::root::{IReaperPitchShift, ReaSample};

impl IReaperPitchShift {
    pub fn set_srate(&self, srate: f64) {
        unsafe {
            IReaperPitchShift_set_srate(self as *const _ as _, srate);
        }
    }

    pub fn set_nch(&self, nch: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_set_nch(self as *const _ as _, nch);
        }
    }

    pub fn set_shift(&self, shift: f64) {
        unsafe {
            IReaperPitchShift_set_shift(self as *const _ as _, shift);
        }
    }

    pub fn set_formant_shift(&self, shift: f64) {
        unsafe {
            IReaperPitchShift_set_formant_shift(self as *const _ as _, shift);
        }
    }

    pub fn set_tempo(&self, tempo: f64) {
        unsafe {
            IReaperPitchShift_set_tempo(self as *const _ as _, tempo);
        }
    }

    pub fn Reset(&self) {
        unsafe {
            IReaperPitchShift_Reset(self as *const _ as _);
        }
    }

    pub fn GetBuffer(&self, size: ::std::os::raw::c_int) -> *mut ReaSample {
        unsafe { IReaperPitchShift_GetBuffer(self as *const _ as _, size) }
    }

    pub fn BufferDone(&self, input_filled: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_BufferDone(self as *const _ as _, input_filled);
        }
    }

    pub fn FlushSamples(&self) {
        unsafe {
            IReaperPitchShift_FlushSamples(self as *const _ as _);
        }
    }

    pub fn IsReset(&self) -> bool {
        unsafe { IReaperPitchShift_IsReset(self as *const _ as _) }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetSamples(
        &self,
        requested_output: ::std::os::raw::c_int,
        buffer: *mut ReaSample,
    ) -> ::std::os::raw::c_int {
        IReaperPitchShift_GetSamples(self as *const _ as _, requested_output, buffer)
    }

    pub fn SetQualityParameter(&self, parm: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_SetQualityParameter(self as *const _ as _, parm);
        }
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
        IReaperPitchShift_Extended(self as *const _ as _, call, parm1, parm2, parm3)
    }
}
