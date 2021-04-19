#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pcm_source::*;
use super::bindings::root::{midi_Input, MIDI_event_t, MIDI_eventlist};
use crate::bindings::root::{midi_Output, PCM_source};
use std::os::raw::{c_char, c_int, c_void};

impl PCM_source {
    pub fn GetLength(&self) -> f64 {
        unsafe { PCM_source_GetLength(self as *const _ as _) }
    }

    pub fn IsAvailable(&self) -> bool {
        unsafe { PCM_source_IsAvailable(self as *const _ as _) }
    }

    pub fn Duplicate(&self) -> *mut PCM_source {
        unsafe { PCM_source_Duplicate(self as *const _ as _) }
    }

    pub fn GetType(&self) -> *const c_char {
        unsafe { PCM_source_GetType(self as *const _ as _) }
    }

    pub fn GetFileName(&self) -> *const c_char {
        unsafe { PCM_source_GetFileName(self as *const _ as _) }
    }

    pub fn GetSource(&self) -> *mut PCM_source {
        unsafe { PCM_source_GetSource(self as *const _ as _) }
    }

    pub fn Extended(
        &self,
        call: c_int,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> c_int {
        unsafe { PCM_source_Extended(self as *const _ as _, call, parm1, parm2, parm3) }
    }
}
