use crate::low_level::raw::MIDI_event_t;
use crate::low_level::{midi_Input, midi_Output, MIDI_eventlist};
use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_int;

// TODO Doc
// This is like a MediaTrack object in that it wraps a raw pointer.
// TODO Can we check the lifetime of this in ValidatePtr2? How does this behave when the MIDI input
//  device is disconnected? What would get_read_buf() return? If it crashes, we should think about
//  making it unsafe or expect a closure when calling get_midi_input (latter is probably the way to
//  go). That closure would expect a reference of the MidiInput. => Well, we could probably check
//  the validity of the device if we check its presence via GetMIDIInputName with the appropriate
//  device ID?
pub struct MidiInput(midi_Input);

impl MidiInput {
    pub(super) fn new(raw_input: midi_Input) -> MidiInput {
        MidiInput(raw_input)
    }

    // This expects a function because the result (MIDI event list) is *very* temporary in nature.
    // If we would return a &MidiEventList, we wouldn't be able to find a sane lifetime
    // annotation. If we would return a pointer, we would require the consumer to enter unsafe
    // world to do anything useful with it. If we would return an owned event list, we would
    // waste performance because we would need to copy all events first. Latter would be
    // especially bad because this code code typically runs in the audio thread and therefore
    // has real-time requirements. Same reasoning like here: https://stackoverflow.com/questions/61106587
    pub fn get_read_buf<R>(&self, f: impl Fn(&MidiEventList) -> R) -> R {
        let raw_evt_list = self.0.GetReadBuf();
        f(&MidiEventList::new(&raw_evt_list))
    }
}

// This should be an unsized type (only usable as reference). There will maybe be a sized/owned
// counterpart in future.
pub struct MidiEventList<'a>(&'a MIDI_eventlist);

impl<'a> MidiEventList<'a> {
    // TODO Maybe from() would be a better name for all pointer wrappers.
    pub(super) fn new(raw_evt_list: &'a MIDI_eventlist) -> Self {
        MidiEventList(raw_evt_list)
    }

    pub fn enum_items(&'a self, bpos: u32) -> MidiEventListIterator<'a> {
        MidiEventListIterator {
            raw_list: self.0,
            bpos: bpos as i32,
        }
    }
}

pub struct MidiEventListIterator<'a> {
    raw_list: &'a MIDI_eventlist,
    bpos: i32,
}

impl<'a> Iterator for MidiEventListIterator<'a> {
    type Item = MidiEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_evt = unsafe { self.raw_list.EnumItems(&mut self.bpos as *mut c_int) };
        if raw_evt.is_null() {
            // No MIDI events left
            return None;
        }
        Some(MidiEvent::new(raw_evt))
    }
}

pub struct MidiEvent<'a>(*mut MIDI_event_t, PhantomData<&'a ()>);

impl<'a> MidiEvent<'a> {
    pub(super) fn new(raw_evt: *mut MIDI_event_t) -> Self {
        MidiEvent(raw_evt, PhantomData)
    }
}

pub struct MidiOutput(midi_Output);

impl MidiOutput {
    pub(super) fn new(raw_output: midi_Output) -> MidiOutput {
        MidiOutput(raw_output)
    }
}
