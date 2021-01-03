use crate::ReactiveEvent;
use helgoboss_midi::{RawShortMessage, ShortMessage, ShortMessageType};
use reaper_medium::{
    MidiFrameOffset, MidiInputDeviceId, OnAudioBufferArgs, RealTimeAudioThreadScope,
};
use rxrust::prelude::*;

pub struct MidiRxDriver {
    medium_reaper: reaper_medium::Reaper<RealTimeAudioThreadScope>,
    rx: MidiRx,
}

#[derive(Clone, Default)]
pub struct MidiRx {
    midi_message_received: LocalSubject<'static, MidiEvent<RawShortMessage>, ()>,
}

impl MidiRxDriver {
    pub fn on_audio_buffer(&mut self, args: OnAudioBufferArgs) {
        if args.is_post {
            return;
        }
        let subject = &mut self.rx.midi_message_received;
        if subject.subscribed_size() == 0 {
            return;
        }
        for i in 0..self.medium_reaper.get_max_midi_inputs() {
            self.medium_reaper
                .get_midi_input(MidiInputDeviceId::new(i as u8), |input| {
                    let evt_list = input.get_read_buf();
                    for evt in evt_list.enum_items(0) {
                        let msg = evt.message();
                        if msg.r#type() == ShortMessageType::ActiveSensing {
                            // TODO-low We should forward active sensing. Can be filtered out
                            // later.
                            continue;
                        }
                        let owned_msg: RawShortMessage = msg.to_other();
                        let owned_evt = MidiEvent::new(evt.frame_offset(), owned_msg);
                        subject.next(owned_evt);
                    }
                });
        }
    }
}

impl MidiRx {
    pub fn midi_message_received(&self) -> impl ReactiveEvent<MidiEvent<RawShortMessage>> {
        self.midi_message_received.clone()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct MidiEvent<M> {
    frame_offset: MidiFrameOffset,
    msg: M,
}

impl<M> MidiEvent<M> {
    pub fn new(frame_offset: MidiFrameOffset, msg: M) -> MidiEvent<M> {
        MidiEvent { frame_offset, msg }
    }
}
