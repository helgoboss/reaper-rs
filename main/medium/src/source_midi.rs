use std::vec::IntoIter;

use crate::{CcShapeKind, PositionInPpq};

#[derive(Debug)]
pub struct SourceMidiEvent {
    position_in_ppq: PositionInPpq,
    is_selected: bool,
    is_muted: bool,
    cc_shape_kind: CcShapeKind,
    message: Vec<u8>,
}
impl SourceMidiEvent {
    pub fn new(
        position_in_ppq: PositionInPpq,
        is_selected: bool,
        is_muted: bool,
        cc_shape_kind: CcShapeKind,
        message: Vec<u8>,
    ) -> Self {
        Self {
            position_in_ppq,
            is_selected,
            is_muted,
            cc_shape_kind,
            message,
        }
    }
    pub fn get_position(&self) -> PositionInPpq {
        self.position_in_ppq
    }
    pub fn set_position(&mut self, position: PositionInPpq) {
        self.position_in_ppq = position;
    }
    pub fn get_selected(&self) -> bool {
        self.is_selected
    }
    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }
    pub fn get_muted(&self) -> bool {
        self.is_muted
    }
    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
    }
    pub fn get_cc_shape_kind(&self) -> CcShapeKind {
        self.cc_shape_kind
    }
    pub fn set_cc_shape_kind(&mut self, cc_shape_kind: CcShapeKind) {
        self.cc_shape_kind = cc_shape_kind;
    }
    pub fn get_message(&self) -> &Vec<u8> {
        &self.message
    }
    pub fn get_message_mut(&mut self) -> &mut Vec<u8> {
        &mut self.message
    }
    pub fn set_message(&mut self, message: Vec<u8>) {
        self.message = message;
    }
}

pub struct SourceMidiEventIterator {
    buf: IntoIter<u8>,
    current_ppq: u32,
}
impl SourceMidiEventIterator {
    pub(crate) fn new(buf: Vec<u8>) -> Self {
        Self {
            buf: buf.into_iter(),
            current_ppq: 0,
        }
    }

    fn next_4(&mut self) -> Option<[u8; 4]> {
        match (
            self.buf.next(),
            self.buf.next(),
            self.buf.next(),
            self.buf.next(),
        ) {
            (Some(a), Some(b), Some(c), Some(d)) => Some([a, b, c, d]),
            _ => None,
        }
    }
}
impl Iterator for SourceMidiEventIterator {
    type Item = SourceMidiEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.next_4() {
            Some(value) => value,
            None => return None,
        };
        let offset = u32::from_le_bytes(result);
        let flag = self
            .buf
            .next()
            .expect("unexpectetly ended. Should be flag.");
        let length = u32::from_le_bytes(self.next_4().expect("should take length"));
        if length == 0 {
            return None;
        }
        self.current_ppq += offset;
        let buf = self.buf.by_ref().take(length as usize);
        Some(SourceMidiEvent {
            position_in_ppq: PositionInPpq::new(self.current_ppq as f64),
            cc_shape_kind: CcShapeKind::from_raw(flag & 0b11110000),
            is_selected: (flag & 1) != 0,
            is_muted: (flag & 2) != 0,
            message: Vec::from_iter(buf),
        })
    }
}
