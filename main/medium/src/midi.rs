use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use crate::{MidiFrameOffset, SendMidiTime};
use reaper_low::raw::MIDI_event_t;
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
    /// [`get_midi_input()`]: struct.Reaper.html#method.get_midi_input
    pub fn get_read_buf(&self) -> BorrowedMidiEventList<'_> {
        let raw_evt_list = unsafe { self.0.as_ref().GetReadBuf() };
        BorrowedMidiEventList(unsafe { &*raw_evt_list })
    }
}

/// A list of MIDI events borrowed from REAPER.
//
// Internals exposed: no | vtable: yes (Rust => REAPER)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BorrowedMidiEventList<'a>(pub(crate) &'a raw::MIDI_eventlist);

impl<'a> BorrowedMidiEventList<'a> {
    /// Returns an iterator exposing the contained MIDI events.
    ///
    /// `bpos` is the iterator start position.
    // TODO-high Why does this consume self and is Copy?
    pub fn enum_items(self, bpos: u32) -> impl Iterator<Item = BorrowedMidiEvent<'a>> {
        EnumItems {
            raw_list: self.0,
            bpos: bpos as i32,
        }
    }

    /// Adds an item to this list of MIDI events.
    pub fn add_item(self, msg: BorrowedMidiEvent) {
        unsafe {
            self.0.AddItem(msg.0 as *const _ as _);
        }
    }
}

/// A MIDI event borrowed from REAPER.
// # Internals exposed: yes | vtable: no
// TODO-low Can be converted into an owned MIDI event in case it needs to live longer than REAPER
//  keeps  the event around.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BorrowedMidiEvent<'a>(&'a raw::MIDI_event_t);

/// A MIDI message borrowed from REAPER.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BorrowedMidiMessage<'a>(&'a raw::MIDI_event_t);

impl<'a> BorrowedMidiEvent<'a> {
    /// Wraps the given raw MIDI event reference.
    pub fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        BorrowedMidiEvent(raw_evt)
    }

    /// Returns the frame offset.
    pub fn frame_offset(self) -> MidiFrameOffset {
        MidiFrameOffset::new(self.0.frame_offset as u32)
    }

    /// Returns the actual message.
    pub fn message(self) -> BorrowedMidiMessage<'a> {
        BorrowedMidiMessage::new(self.0)
    }
}

impl<'a> AsRef<raw::MIDI_event_t> for BorrowedMidiEvent<'a> {
    fn as_ref(&self) -> &MIDI_event_t {
        self.0
    }
}

impl<'a> BorrowedMidiMessage<'a> {
    pub(super) fn new(raw_evt: &'a raw::MIDI_event_t) -> Self {
        BorrowedMidiMessage(raw_evt)
    }
}

struct EnumItems<'a> {
    raw_list: &'a raw::MIDI_eventlist,
    bpos: i32,
}

impl<'a> Iterator for EnumItems<'a> {
    type Item = BorrowedMidiEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_evt = unsafe { self.raw_list.EnumItems(&mut self.bpos as *mut c_int) };
        if raw_evt.is_null() {
            // No MIDI events left
            return None;
        }
        let evt = unsafe { BorrowedMidiEvent::new(&*raw_evt) };
        Some(evt)
    }
}

impl<'a> ShortMessage for BorrowedMidiMessage<'a> {
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

/// Pointer to a MIDI output device.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
//
// It's important that this type is not cloneable! Otherwise consumers could easily let it escape
// its intended usage scope (audio hook), which would lead to undefined behavior.
//
// Internals exposed: no | vtable: yes (Rust => REAPER)
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct MidiOutput(pub(crate) NonNull<raw::midi_Output>);

impl MidiOutput {
    /// Sends the given arbitrary MIDI message to this device at the given time.
    ///
    /// This must only be called in the real-time audio thread! See [`get_midi_output()`].
    ///
    /// [`get_midi_output()`]: struct.Reaper.html#method.get_midi_output
    pub fn send_msg(&self, msg: impl AsRef<raw::MIDI_event_t>, time: SendMidiTime) {
        unsafe {
            self.0
                .as_ref()
                .SendMsg(msg.as_ref() as *const _ as _, time.to_raw());
        }
    }

    /// Sends the given short message to this device at the given time.
    ///
    /// This must only be called in the real-time audio thread! See [`get_midi_output()`].
    ///
    /// [`get_midi_output()`]: struct.Reaper.html#method.get_midi_output
    pub fn send(&self, message: impl ShortMessage, time: SendMidiTime) {
        let bytes = message.to_bytes();
        unsafe {
            self.0
                .as_ref()
                .Send(bytes.0, bytes.1.get(), bytes.2.get(), time.to_raw());
        }
    }
}
