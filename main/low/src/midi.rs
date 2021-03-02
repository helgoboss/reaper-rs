#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_midi::*;
use super::bindings::root::{midi_Input, MIDI_event_t, MIDI_eventlist};
use crate::bindings::root::midi_Output;
use std::os::raw::c_int;

impl midi_Input {
    pub fn GetReadBuf(&self) -> *mut MIDI_eventlist {
        unsafe { midi_Input_GetReadBuf(self as *const _ as _) }
    }
}

impl MIDI_eventlist {
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn EnumItems(&self, bpos: *mut c_int) -> *mut MIDI_event_t {
        MIDI_eventlist_EnumItems(self as *const _ as _, bpos)
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn AddItem(&self, evt: *mut MIDI_event_t) {
        MIDI_eventlist_AddItem(self as *const _ as _, evt);
    }
}

impl midi_Output {
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn SendMsg(&self, msg: *mut MIDI_event_t, frame_offset: c_int) {
        midi_Output_SendMsg(self as *const _ as _, msg, frame_offset);
    }

    pub fn Send(&self, status: u8, d1: u8, d2: u8, frame_offset: c_int) {
        unsafe { midi_Output_Send(self as *const _ as *mut _, status, d1, d2, frame_offset) };
    }
}
