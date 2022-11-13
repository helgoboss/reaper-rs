use std::vec::IntoIter;

use crate::{CcShapeKind, PositionInPpq};

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct SourceMidiEvent {
    position_in_ppq: PositionInPpq,
    is_selected: bool,
    is_muted: bool,
    cc_shape_kind: CcShapeKind,
    /// Message can be as ordinary 3-bytes midi-message,
    /// as well as SysEx and custom messages, including lyrics and text.
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

/// Iterates over raw take midi data and builds SourceMediaEvent objects.
#[derive(Debug)]
pub struct SourceMidiEventBuilder {
    buf: IntoIter<u8>,
    current_ppq: u32,
}
impl SourceMidiEventBuilder {
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
impl Iterator for SourceMidiEventBuilder {
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
            cc_shape_kind: CcShapeKind::from_raw(flag & 0b11110000)
                .expect("Can not infer CcShapeKind, received from take."),
            is_selected: (flag & 1) != 0,
            is_muted: (flag & 2) != 0,
            message: Vec::from_iter(buf),
        })
    }
}

/// Iterates through SourceMediaEvent objects and builds raw midi data
/// to be passed to take. 
#[derive(Debug)]
pub struct SourceMidiEventConsumer {
    events: IntoIter<SourceMidiEvent>,
    last_ppq: u32,
    buf: Option<IntoIter<u8>>,
}
impl SourceMidiEventConsumer {
    /// Build iterator.
    /// 
    /// If sort is true — vector would be sorted by ppq_position.
    /// Be careful, this costs additional O(log n) operation in the worst case.
    pub fn new(mut events: Vec<SourceMidiEvent>, sort: bool) -> Self {
        if sort == true {
            events.sort_by_key(|ev| ev.get_position().get() as u32);
        }
        Self {
            events: events.into_iter(),
            last_ppq: 0,
            buf: None,
        }
    }

    /// Checks if some events are left and builds new buf for iteration.
    fn next_buf(&mut self) -> Option<i8> {
        match self.events.next() {
            None => None,
            Some(mut event) => {
                let size = event.get_message().len() + 9;
                let pos = event.get_position().get() as u32;
                let mut offset = (pos - self.last_ppq).to_le_bytes().to_vec();
                self.last_ppq = pos;
                let flag = (event.get_selected() as u8)
                    | ((event.get_muted() as u8) << 1)
                    | event.get_cc_shape_kind().to_raw();
                let mut length = event.get_message().len().to_le_bytes().to_vec();
                //
                let mut buf = Vec::with_capacity(size);
                buf.append(&mut offset);
                buf.push(flag);
                buf.append(&mut length);
                buf.append(event.get_message_mut());
                //
                self.buf = Some(buf.into_iter());
                // Some(i8)
                Some(self.buf.as_mut().unwrap().next().unwrap() as i8)
            }
        }
    }
}

impl Iterator for SourceMidiEventConsumer {
    type Item = i8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.buf.as_mut() {
            Some(buf) => match buf.next() {
                Some(next) => Some(next as i8),
                None => self.next_buf(),
            },
            None => self.next_buf(),
        }
    }
}
