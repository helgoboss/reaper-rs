#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pcm_source::*;
use super::bindings::root::{midi_Input, MIDI_event_t, MIDI_eventlist};
use crate::bindings::root::{midi_Output, PCM_source};
use std::os::raw::c_int;

impl PCM_source {
    pub fn GetLength(&self) -> f64 {
        unsafe { PCM_source_GetLength(self as *const _ as _) }
    }
}
