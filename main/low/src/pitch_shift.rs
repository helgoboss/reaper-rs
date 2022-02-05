#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pitch_shift::*;
use crate::bindings::root::{IReaperPitchShift, ReaSample};
use crate::raw;
use std::ptr::NonNull;

impl IReaperPitchShift {
    pub fn set_srate(&mut self, srate: f64) {
        unsafe {
            IReaperPitchShift_set_srate(self as _, srate);
        }
    }

    pub fn set_nch(&mut self, nch: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_set_nch(self as _, nch);
        }
    }

    pub fn set_shift(&mut self, shift: f64) {
        unsafe {
            IReaperPitchShift_set_shift(self as _, shift);
        }
    }

    pub fn set_formant_shift(&mut self, shift: f64) {
        unsafe {
            IReaperPitchShift_set_formant_shift(self as _, shift);
        }
    }

    pub fn set_tempo(&mut self, tempo: f64) {
        unsafe {
            IReaperPitchShift_set_tempo(self as _, tempo);
        }
    }

    pub fn Reset(&mut self) {
        unsafe {
            IReaperPitchShift_Reset(self as _);
        }
    }

    pub fn GetBuffer(&mut self, size: ::std::os::raw::c_int) -> *mut ReaSample {
        unsafe { IReaperPitchShift_GetBuffer(self as _, size) }
    }

    pub fn BufferDone(&mut self, input_filled: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_BufferDone(self as _, input_filled);
        }
    }

    pub fn FlushSamples(&mut self) {
        unsafe {
            IReaperPitchShift_FlushSamples(self as _);
        }
    }

    pub fn IsReset(&mut self) -> bool {
        unsafe { IReaperPitchShift_IsReset(self as _) }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetSamples(
        &mut self,
        requested_output: ::std::os::raw::c_int,
        buffer: *mut ReaSample,
    ) -> ::std::os::raw::c_int {
        IReaperPitchShift_GetSamples(self as _, requested_output, buffer)
    }

    pub fn SetQualityParameter(&mut self, parm: ::std::os::raw::c_int) {
        unsafe {
            IReaperPitchShift_SetQualityParameter(self as _, parm);
        }
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
        IReaperPitchShift_Extended(self as _, call, parm1, parm2, parm3)
    }
}

/// Destroys a C++ `IReaperPitchShift` object.
///
/// Intended to be used on pointers returned by [`ReaperGetPitchShiftAPI()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`ReaperGetPitchShiftAPI()`]: struct.Reaper.html#method.ReaperGetPitchShiftAPI
pub unsafe fn delete_cpp_reaper_pitch_shift(pitch_shift: NonNull<raw::IReaperPitchShift>) {
    crate::bindings::root::reaper_pitch_shift::delete_reaper_pitch_shift(pitch_shift.as_ptr());
}
