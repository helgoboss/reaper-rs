use helgoboss_midi::{MidiMessage, U7};
use reaper_rs_low::raw;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::ptr::NonNull;

// TODO-doc
// This is like a MediaTrack object in that it wraps a raw pointer.
// TODO-medium Can we check the lifetime of this in ValidatePtr2? How does this behave when the MIDI
// input  device is disconnected? What would get_read_buf() return? If it crashes, we should think
// about  making it unsafe or expect a closure when calling get_midi_input (latter is probably the
// way to  go). That closure would expect a reference of the MidiInput. => Well, we could probably
// check  the validity of the device if we check its presence via GetMIDIInputName with the
// appropriate  device ID?
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct MidiInput(pub NonNull<raw::midi_Input>);

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct MidiOutput(pub NonNull<raw::midi_Output>);

impl MidiInput {
    // This expects a function because the result (MIDI event list) is *very* temporary in nature.
    // If we would return a &MidiEventList, we wouldn't be able to find a sane lifetime
    // annotation. If we would return a pointer, we would require the consumer to enter unsafe
    // world to do anything useful with it. If we would return an owned event list, we would
    // waste performance because we would need to copy all events first. Latter would be
    // especially bad because this code code typically runs in the audio thread and therefore
    // has real-time requirements. Same reasoning like here: https://stackoverflow.com/questions/61106587
    pub unsafe fn get_read_buf<R>(&self, mut f: impl FnOnce(&MidiEvtList) -> R) -> R {
        let raw_evt_list = self.0.as_ref().GetReadBuf();
        f(&MidiEvtList::new(&*raw_evt_list))
    }
}

pub struct MidiEvtList<'a>(&'a raw::MIDI_eventlist);

impl<'a> MidiEvtList<'a> {
    // TODO-medium Maybe from() would be a better name for all pointer wrappers.
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

// TODO-medium Check everything if conform with API conventions
//  (e.g. https://rust-lang.github.io/api-guidelines/naming.html)
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

impl<'a> MidiMessage for MidiMsg<'a> {
    fn get_status_byte(&self) -> u8 {
        self.0.midi_message[0]
    }

    fn get_data_byte_1(&self) -> U7 {
        unsafe { U7::new_unchecked(self.0.midi_message[1]) }
    }

    fn get_data_byte_2(&self) -> U7 {
        unsafe { U7::new_unchecked(self.0.midi_message[2]) }
    }
}
