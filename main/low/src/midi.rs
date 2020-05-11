#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_midi::*;
use super::bindings::root::{midi_Input, MIDI_event_t, MIDI_eventlist};

impl midi_Input {
    pub fn GetReadBuf(&self) -> *mut MIDI_eventlist {
        unsafe { midi_Input_GetReadBuf(self as *const _ as *mut _) }
    }
}

impl MIDI_eventlist {
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn EnumItems(&self, bpos: *mut ::std::os::raw::c_int) -> *mut MIDI_event_t {
        MIDI_eventlist_EnumItems(self as *const _ as *mut _, bpos)
    }
}
