use crate::high_level::MidiInputDevice;
use crate::low_level::MIDI_event_t;
use helgoboss_midi::{MidiMessage, U7};

pub trait MidiEvent {
    type Msg: MidiMessage;

    fn get_frame_offset(&self) -> i32;

    fn get_message(&self) -> &Self::Msg;
}

// TODO-low Continue
#[allow(dead_code)]
pub struct MidiMessageReceivedPayload<Evt> {
    device: MidiInputDevice,
    event: Evt,
}

// Represents a borrowed reference to a MIDI event from REAPER. Cheap to copy because it's just a
// wrapper around MIDI_event_t. Can be converted into an owned MIDI event in case it needs to live
// longer than REAPER keeps the event around.
// TODO-low Don't hold a pointer but a reference. This activates Rust's lifetime checking which
// would be  super helpful in this case. However, we need a good way to use subjects with references
// in  rxRust before doing that. Currently as_mut_ref erases lifetime and is unsafe, so it's not
// better  than the current approach.
#[derive(Clone, Copy)]
pub struct BorrowedReaperMidiEvent(pub *const MIDI_event_t);

impl BorrowedReaperMidiEvent {
    fn inner(&self) -> &MIDI_event_t {
        unsafe { &*self.0 }
    }
}

impl MidiEvent for BorrowedReaperMidiEvent {
    type Msg = BorrowedReaperMidiEvent;

    fn get_frame_offset(&self) -> i32 {
        self.inner().frame_offset
    }

    fn get_message(&self) -> &Self::Msg {
        &self
    }
}

impl MidiMessage for BorrowedReaperMidiEvent {
    fn get_status_byte(&self) -> u8 {
        self.inner().midi_message[0]
    }

    fn get_data_byte_1(&self) -> U7 {
        unsafe { U7::new_unchecked(self.inner().midi_message[1]) }
    }

    fn get_data_byte_2(&self) -> U7 {
        unsafe { U7::new_unchecked(self.inner().midi_message[2]) }
    }
}

//impl ToOwned for BorrowedReaperMidiEvent {
//    type Owned = OwnedReaperMidiEvent;
//
//    fn to_owned(&self) -> Self::Owned {
//        let inner_copy = *self.inner();
//        OwnedReaperMidiEvent {
//            inner: inner_copy,
//            borrowed: BorrowedReaperMidiEvent(&inner_copy as *const _)
//        }
//    }
//}

// TODO-low Continue
#[allow(dead_code)]
pub struct OwnedReaperMidiEvent {
    inner: MIDI_event_t,
    borrowed: BorrowedReaperMidiEvent,
}

impl MidiEvent for OwnedReaperMidiEvent {
    type Msg = OwnedReaperMidiEvent;

    fn get_frame_offset(&self) -> i32 {
        self.inner.frame_offset
    }

    fn get_message(&self) -> &Self::Msg {
        &self
    }
}

impl MidiMessage for OwnedReaperMidiEvent {
    fn get_status_byte(&self) -> u8 {
        self.inner.midi_message[0]
    }

    fn get_data_byte_1(&self) -> U7 {
        unsafe { U7::new_unchecked(self.inner.midi_message[1]) }
    }

    fn get_data_byte_2(&self) -> U7 {
        unsafe { U7::new_unchecked(self.inner.midi_message[2]) }
    }
}

//impl Borrow<BorrowedReaperMidiEvent> for OwnedReaperMidiEvent {
//    fn borrow(&self) -> &BorrowedReaperMidiEvent {
//        &self.borrowed
//    }
//}
