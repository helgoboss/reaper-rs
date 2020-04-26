use helgoboss_midi::{ShortMessage, U7};
use reaper_rs_low::raw;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::ptr::NonNull;

// This is like a MediaTrack object in that it wraps a raw pointer.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct MidiInput(pub(crate) NonNull<raw::midi_Input>);

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct MidiOutput(pub(crate) NonNull<raw::midi_Output>);

impl MidiInput {
    // In the past this function was unsafe, didn't return the event list and expected a closure
    // instead that could do something with the event list. That's not necessary anymore since we
    // ensure in get_midi_input() that we only ever publish valid MidiInput instances, and those
    // only by a very short-lived reference that's not possible to cache anywhere. That makes it
    // possible to bind the lifetime of the event list to that MidiInput and everything is fine!
    // Thanks Rust! If we would return an owned event list, we would waste performance because we
    // would need to copy all events first. That would be especially bad because this code code
    // typically runs in the audio thread and therefore has real-time requirements.
    // Should we mark this as unsafe because it can crash if accessed wrongly from UI thread instead
    // of audio thread? I don't think so, then we would have to mark most functions as unsafe
    // because most functions can only be called from UI thread.
    // TODO-low In theory we could prevent undefined behavior by always checking the thread at
    //  first.
    pub fn get_read_buf(&self) -> MidiEvtList<'_> {
        let raw_evt_list = unsafe { self.0.as_ref() }.GetReadBuf();
        MidiEvtList::new(unsafe { &*raw_evt_list })
    }
}

pub struct MidiEvtList<'a>(&'a raw::MIDI_eventlist);

impl<'a> MidiEvtList<'a> {
    pub(super) fn new(raw_evt_list: &'a raw::MIDI_eventlist) -> Self {
        MidiEvtList(raw_evt_list)
    }

    pub fn enum_items(&self, bpos: u32) -> MidiEvtListIterator<'a> {
        MidiEvtListIterator {
            raw_list: self.0,
            bpos: bpos as i32,
        }
    }
}

pub struct MidiEvtListIterator<'a> {
    raw_list: &'a raw::MIDI_eventlist,
    bpos: i32,
}

impl<'a> Iterator for MidiEvtListIterator<'a> {
    type Item = MidiEvt<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_evt = unsafe { self.raw_list.EnumItems(&mut self.bpos as *mut c_int) };
        if raw_evt.is_null() {
            // No MIDI events left
            return None;
        }
        let evt = unsafe { MidiEvt::new(&*raw_evt) };
        Some(evt)
    }
}

// Represents a borrowed reference to a MIDI event from REAPER. Cheap to copy because it's just a
// wrapper around MIDI_event_t.
// TODO-low Can be converted into an owned MIDI event in case it needs to live longer than REAPER
//  keeps  the event around.
#[derive(Clone, Copy)]
pub struct MidiEvt<'a>(&'a raw::MIDI_event_t);

impl<'a> MidiEvt<'a> {
    pub unsafe fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        MidiEvt(raw_evt)
    }

    pub fn get_frame_offset(&self) -> u32 {
        self.0.frame_offset as u32
    }

    pub fn get_message(&self) -> MidiMsg<'a> {
        MidiMsg::new(self.0)
    }
}

impl<'a> From<MidiEvt<'a>> for &'a raw::MIDI_event_t {
    fn from(outer: MidiEvt<'a>) -> Self {
        outer.0
    }
}

pub struct MidiMsg<'a>(&'a raw::MIDI_event_t);

impl<'a> MidiMsg<'a> {
    pub(super) fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        MidiMsg(raw_evt)
    }
}

impl<'a> ShortMessage for MidiMsg<'a> {
    fn status_byte(&self) -> u8 {
        self.0.midi_message[0]
    }

    fn data_byte_1(&self) -> U7 {
        unsafe { U7::new_unchecked(self.0.midi_message[1]) }
    }

    fn data_byte_2(&self) -> U7 {
        unsafe { U7::new_unchecked(self.0.midi_message[2]) }
    }
}
