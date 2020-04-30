use crate::{MidiInput, MidiInputDeviceId};
use std::ptr::NonNull;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct RealTimeReaper {
    low: reaper_rs_low::Reaper,
}

impl RealTimeReaper {
    pub fn new(low: reaper_rs_low::Reaper) -> RealTimeReaper {
        RealTimeReaper { low }
    }

    // If the MIDI device is disconnected we wouldn't obtain it in the first place by
    // get_midi_input(). If we would try to call get_read_buf() on a cached instance of that
    // pointer, it would crash. Unlike with many other pointers returned by REAPER AFAIK there's
    // no way to check the validity of a midi_Input via ValidatePtr. So I think it would
    // *always* be unwise to cache a midi_Input ptr. There's also no need for that because we
    // have a single global ID (1 - 62) which we can use to quickly lookup the pointer any time.
    // Because of that we take a closure and pass a reference (https://stackoverflow.com/questions/61106587).
    // An alternative would have been to return the pointer wrapper. But then we would have to mark
    // this function as unsafe in order to make aware of the fact that operations on the result
    // could result in undefined behavior as soon as the current stack frame is left. If it turns
    // out that the function-taking approach is too restrictive in some cases (wouldn't know why),
    // we could always provide a second function get_midi_input_unchecked().
    pub fn get_midi_input<R>(
        &self,
        idx: MidiInputDeviceId,
        mut f: impl FnOnce(&MidiInput) -> R,
    ) -> Option<R> {
        let ptr = self.low.GetMidiInput(idx.into());
        if ptr.is_null() {
            return None;
        }
        NonNull::new(ptr).map(|nnp| f(&MidiInput(nnp)))
    }

    pub fn get_max_midi_inputs(&self) -> u32 {
        self.low.GetMaxMidiInputs() as u32
    }

    pub fn get_max_midi_outputs(&self) -> u32 {
        self.low.GetMaxMidiOutputs() as u32
    }

    // TODO-medium Add functions: IsInRealTimeAudio, Audio_IsPreBuffer, Audio_IsRunning,
    // kbd_OnMidiEvent, kbd_OnMidiList
}
