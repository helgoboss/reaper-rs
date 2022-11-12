// use crate::{CcShapeKind, MidiFrameOffset, PositionInPpq};

// #[derive(Debug, Clone)]
// pub struct MidiMessage {
//     size: u32,
//     bytes: Box<[u8]>,
// }
// impl MidiMessage {
//     pub fn size(&self) -> u32 {
//         self.size
//     }
//     pub fn get(self) -> Box<[u8]> {
//         self.bytes
//     }
//     pub fn set(&mut self, bytes: Box<[u8]>) {
//         self.bytes = bytes;
//     }
// }

// pub struct EnumSourceMidiEvent {
//     pub message: MidiMessage,
//     pub offset: MidiFrameOffset
// }
// impl EnumSourceMidiEvent{
//     pub fn new(message:MidiMessage, offset:MidiFrameOffset)->Self{
//         Self { message, offset }
//     }
// }

// pub enum SourceMidiEvent{
//     Cc(SourceCcEvent),
//     NoteOn(SourceNoteOnEvent),

// }

use helgoboss_midi::{U4, U7};

use crate::PositionInPpq;

#[derive(Debug)]
pub struct SourceMidiEvent<T> {
    position_in_ppq: PositionInPpq,
    is_selected: bool,
    is_muted: bool,
    event: T,
}
impl<T> SourceMidiEvent<T> {
    pub fn new(position_in_ppq: PositionInPpq, selected: bool, muted: bool, event: T) -> Self {
        Self {
            position_in_ppq,
            is_selected: selected,
            is_muted: muted,
            event,
        }
    }
}

#[derive(Debug)]
pub struct GenericMessage{
    pub size:u32,
    pub message: Vec<u8>
}

#[derive(Debug)]
pub struct CcMessage {
    pub channel_message: U4,
    pub channel: U4,
    pub cc_num: U7,
    pub value: U7,
}
// impl SourceMidiEvent_bck {
//     pub fn get_pos_in_ppq(&self) -> PositionInPpq {
//         self.position_in_ppq
//     }
//     pub fn set_pos_in_ppq(&mut self, position: PositionInPpq) {
//         self.position_in_ppq = position;
//     }
//     pub fn is_selected(&self) -> bool {
//         self.is_selected
//     }
//     pub fn set_selected(&mut self, selected: bool) {
//         self.is_selected = selected;
//     }
//     pub fn is_muted(&self) -> bool {
//         self.is_muted
//     }
//     pub fn set_muted(&mut self, muted: bool) {
//         self.is_muted = muted;
//     }
//     pub fn get_message(self) -> MidiMessage {
//         self.message
//     }
//     pub fn set_message(&mut self, message: MidiMessage) {
//         self.message = message
//     }
//     pub fn get_cc_shape(self) -> CcShape {
//         self.cc_shape
//     }
//     pub fn set_cc_shape(&mut self, cc_shape: CcShape) {
//         self.cc_shape = cc_shape;
//     }
// }

// pub struct CcShape {
//     kind: CcShapeKind,
//     tension: Option<f32>,
// }
// impl CcShape {
//     pub fn get_kind(&self) -> CcShapeKind {
//         self.kind
//     }
//     pub fn set_kind(&mut self, kind: CcShapeKind) {
//         self.kind = kind;
//     }
//     pub fn get_tension(&self) -> Option<f32> {
//         self.tension
//     }
//     pub fn set_tension(&mut self, tension: Option<f32>) {
//         self.tension = tension;
//     }
//     // pub fn from_events(events:)
// }
