#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root;
use super::bindings::root::reaper_rs_midi::*;

// TODO-doc
// The difference from MediaTrack and ReaProject is that this is a virtual base class in C++ which
// has public methods. We want to make those public methods accessible in the low-level API already
// because the goal of the low-level API is to be on par with the C++ REAPER API (meaning that
// everything possible with C++ REAPER API is possible with the reaper-rs low-level API as well
// and with same style and same namings, as far as possible). Therefore we crate a wrapper which
// takes care of calling the pure virtual functions by delegating to our C++ glue code. This is
// similar to IReaperControlSurface but the calls go in the other direction (from plug-in to
// REAPER).
pub struct midi_Input(*mut root::midi_Input);

impl midi_Input {
    // TODO-doc
    pub unsafe fn new(ptr: *mut root::midi_Input) -> midi_Input {
        midi_Input(ptr)
    }

    // TODO-doc
    pub fn GetReadBuf(&self) -> MIDI_eventlist {
        unsafe {
            let ptr = midi_Input_GetReadBuf(self.0);
            MIDI_eventlist::new(ptr)
        }
    }
}

// TODO-doc
pub struct MIDI_eventlist(*mut root::MIDI_eventlist);

impl MIDI_eventlist {
    // TODO-doc
    pub unsafe fn new(ptr: *mut root::MIDI_eventlist) -> MIDI_eventlist {
        MIDI_eventlist(ptr)
    }

    // TODO-doc
    // At this point we could start working with references and introduce lifetime annotations. But
    // by design this is out of scope of the low-level API. Medium-level API has the responsibility
    // to make things safe to use, low-level API consequently has an unsafe "pointer nature".
    pub unsafe fn EnumItems(&self, bpos: *mut ::std::os::raw::c_int) -> *mut root::MIDI_event_t {
        unsafe { MIDI_eventlist_EnumItems(self.0, bpos) }
    }
}

// TODO-doc
pub struct midi_Output(*mut root::midi_Output);

impl midi_Output {
    // TODO-doc
    pub unsafe fn new(ptr: *mut root::midi_Output) -> midi_Output {
        midi_Output(ptr)
    }
}
