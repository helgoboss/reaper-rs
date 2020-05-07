use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use std::os::raw::c_int;
use std::ptr::NonNull;

/// Pointer to a MIDI input device.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
//
// It's important that this type is not cloneable! Otherwise consumers could easily let it escape
// its intended usage scope (audio hook), which would lead to undefined behavior.
//
// Internals exposed: no | vtable: yes (Rust => REAPER)
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct MidiInput(pub(crate) NonNull<raw::midi_Input>);

impl MidiInput {
    /// Returns the list of MIDI events which are currently in the buffer.
    ///
    /// This must only be called in the real-time audio thread! See [`get_midi_input()`].
    ///
    /// # Design
    ///
    /// In the past this function was unsafe and expected a closure which let the consumer do
    /// something with the event list. All of that is not necessary anymore since we ensure in
    /// [`get_midi_input()`] that we only ever publish valid [`MidiInput`] instances, and those only
    /// by a very short-lived reference that's not possible to cache anywhere. That makes it
    /// possible to bind the lifetime of the event list to the one of the [`MidiInput`] and
    /// everything is fine!
    ///
    /// Returning an owned event list would be wasteful because we would need to copy all events
    /// first. That would be especially bad because this code is supposed to run in the audio
    /// thread and therefore has real-time requirements.
    ///
    /// [`MidiInput`]: struct.MidiInput.html
    /// [`get_midi_input()`]: struct.ReaperFunctions.html#method.get_midi_input
    pub fn get_read_buf(&self) -> MidiEventList<'_> {
        let raw_evt_list = unsafe { self.0.as_ref().GetReadBuf() };
        MidiEventList::new(unsafe { &*raw_evt_list })
    }
}

/// A list of MIDI events borrowed from REAPER.
//
// Internals exposed: no | vtable: yes (Rust => REAPER)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MidiEventList<'a>(&'a raw::MIDI_eventlist);

impl<'a> MidiEventList<'a> {
    pub(super) fn new(raw_evt_list: &'a raw::MIDI_eventlist) -> Self {
        MidiEventList(raw_evt_list)
    }

    /// Returns an iterator exposing the contained MIDI events.
    ///
    /// `bpos` is the iterator start position.
    pub fn enum_items(&self, bpos: u32) -> impl Iterator<Item = MidiEvent<'a>> {
        EnumItems {
            raw_list: self.0,
            bpos: bpos as i32,
        }
    }
}

/// A MIDI event borrowed from REAPER.
// # Internals exposed: yes | vtable: no
// TODO-low Can be converted into an owned MIDI event in case it needs to live longer than REAPER
//  keeps  the event around.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MidiEvent<'a>(&'a raw::MIDI_event_t);

/// A MIDI message borrowed from REAPER.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MidiMessage<'a>(&'a raw::MIDI_event_t);

impl<'a> MidiEvent<'a> {
    pub(crate) unsafe fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        MidiEvent(raw_evt)
    }

    /// Returns the frame offset.
    ///
    /// Unit: 1/1024000 of a second, *not* sample frames!
    pub fn frame_offset(&self) -> u32 {
        self.0.frame_offset as u32
    }

    /// Returns the actual message.
    pub fn message(&self) -> MidiMessage<'a> {
        MidiMessage::new(self.0)
    }
}

impl<'a> MidiMessage<'a> {
    pub(super) fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        MidiMessage(raw_evt)
    }
}

struct EnumItems<'a> {
    raw_list: &'a raw::MIDI_eventlist,
    bpos: i32,
}

impl<'a> Iterator for EnumItems<'a> {
    type Item = MidiEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_evt = unsafe { self.raw_list.EnumItems(&mut self.bpos as *mut c_int) };
        if raw_evt.is_null() {
            // No MIDI events left
            return None;
        }
        let evt = unsafe { MidiEvent::new(&*raw_evt) };
        Some(evt)
    }
}

impl<'a> ShortMessage for MidiMessage<'a> {
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
