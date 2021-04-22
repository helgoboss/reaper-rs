use helgoboss_midi::{ShortMessage, U7};
use reaper_low::raw;

use crate::{MidiFrameOffset, SendMidiTime};
use reaper_low::raw::MIDI_event_t;
use ref_cast::RefCast;
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
    pub fn get_read_buf(&self) -> &BorrowedMidiEventList {
        let raw_evt_list = unsafe { self.0.as_ref().GetReadBuf() };
        if raw_evt_list.is_null() {
            panic!("GetReadBuf returned null");
        }
        unsafe { std::mem::transmute(raw_evt_list) }
    }
}

/// A list of MIDI events borrowed from REAPER.
//
// Internals exposed: no | vtable: yes (Rust => REAPER)
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedMidiEventList(pub(crate) raw::MIDI_eventlist);

impl BorrowedMidiEventList {
    /// Returns an iterator exposing the contained MIDI events.
    ///
    /// `bpos` is the iterator start position.
    pub fn enum_items(&self, bpos: u32) -> impl Iterator<Item = &BorrowedMidiEvent> {
        EnumItems {
            raw_list: &self.0,
            bpos: bpos as i32,
        }
    }

    /// Adds an item to this list of MIDI events.
    pub fn add_item(&self, msg: &BorrowedMidiEvent) {
        unsafe {
            self.0.AddItem(&msg.0 as *const _ as _);
        }
    }
}

/// A MIDI event borrowed from REAPER.
// # Internals exposed: yes | vtable: no
// TODO-low Can be converted into an owned MIDI event in case it needs to live longer than REAPER
//  keeps  the event around.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedMidiEvent(raw::MIDI_event_t);

/// A MIDI message borrowed from REAPER.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedMidiMessage(raw::MIDI_event_t);

impl BorrowedMidiEvent {
    /// Directly wraps a low-level MIDI event as a medium-level MIDI event.
    ///
    /// This is a cost-free conversion.
    pub fn new(raw: &raw::MIDI_event_t) -> &BorrowedMidiEvent {
        BorrowedMidiEvent::ref_cast(raw)
    }

    /// Returns the frame offset.
    pub fn frame_offset(&self) -> MidiFrameOffset {
        MidiFrameOffset::new(self.0.frame_offset as u32)
    }

    /// Returns the actual message.
    pub fn message(&self) -> &BorrowedMidiMessage {
        BorrowedMidiMessage::ref_cast(&self.0)
    }
}

impl AsRef<raw::MIDI_event_t> for BorrowedMidiEvent {
    fn as_ref(&self) -> &MIDI_event_t {
        &self.0
    }
}

struct EnumItems<'a> {
    raw_list: &'a raw::MIDI_eventlist,
    bpos: i32,
}

impl<'a> Iterator for EnumItems<'a> {
    type Item = &'a BorrowedMidiEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_evt = unsafe { self.raw_list.EnumItems(&mut self.bpos as *mut c_int) };
        if raw_evt.is_null() {
            // No MIDI events left
            return None;
        }
        let evt = unsafe { BorrowedMidiEvent::ref_cast(&*raw_evt) };
        Some(evt)
    }
}

impl ShortMessage for BorrowedMidiMessage {
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
