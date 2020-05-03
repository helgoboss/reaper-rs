#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_rs_midi::*;
use super::bindings::root::{midi_Input, MIDI_event_t, MIDI_eventlist};

// TODO-doc
// The difference from MediaTrack and ReaProject is that this is a virtual base class in C++ which
// has public methods. We want to make those public methods accessible in the low-level API already
// because the goal of the low-level API is to be on par with the C++ REAPER API (meaning that
// everything possible with C++ REAPER API is possible with the reaper-rs low-level API as well
// and with same style and same namings, as far as possible). Therefore we crate a wrapper which
// takes care of calling the pure virtual functions by delegating to our C++ glue code. This is
// similar to IReaperControlSurface but the calls go in the other direction (from plug-in to
// REAPER).
impl midi_Input {
    // TODO-doc
    pub fn GetReadBuf(&self) -> *mut MIDI_eventlist {
        unsafe { midi_Input_GetReadBuf(self as *const _ as *mut _) }
    }
}

impl MIDI_eventlist {
    // TODO-doc
    // At this point we could start working with references and introduce lifetime annotations. But
    // by design this is out of scope of the low-level API. Medium-level API has the responsibility
    // to make things safe to use, low-level API consequently has an unsafe "pointer nature".
    pub unsafe fn EnumItems(&self, bpos: *mut ::std::os::raw::c_int) -> *mut MIDI_event_t {
        unsafe { MIDI_eventlist_EnumItems(self as *const _ as *mut _, bpos) }
    }
}
