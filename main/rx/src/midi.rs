use crate::ReactiveEvent;
use helgoboss_midi::{RawShortMessage, ShortMessage, ShortMessageType};
use reaper_medium::{MidiInputDeviceId, OnAudioBufferArgs, RealTimeAudioThreadScope};
use rxrust::prelude::*;

pub struct MidiRxMiddleware {
    medium_reaper: reaper_medium::Reaper<RealTimeAudioThreadScope>,
    rx: MidiRx,
}

#[derive(Clone, Default)]
pub struct MidiRx {
    midi_message_received: LocalSubject<'static, MidiEvent<RawShortMessage>, ()>,
}

impl MidiRxMiddleware {
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
                    let input = if let Some(i) = input {
                        i
                    } else {
                        return;
                    };
                    let evt_list = input.get_read_buf();
                    for evt in evt_list {
                        let Ok(msg) = evt.message().to_short_message() else {
                            continue;
                        };
                        if msg.r#type() == ShortMessageType::ActiveSensing {
                            // TODO-low We should forward active sensing. Can be filtered out
                            // later.
                            continue;
                        }
                        let owned_evt = MidiEvent::new(evt.frame_offset(), msg);
                        subject.next(owned_evt);
                    }
                });
        }
    }
}

impl MidiRx {
    pub fn midi_message_received(&self) -> ReactiveEvent<MidiEvent<RawShortMessage>> {
        self.midi_message_received.clone()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct MidiEvent<M> {
    frame_offset: u32,
    msg: M,
}

impl<M> MidiEvent<M> {
    pub fn new(frame_offset: u32, msg: M) -> MidiEvent<M> {
        MidiEvent { frame_offset, msg }
    }
}
