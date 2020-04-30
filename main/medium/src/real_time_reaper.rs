use crate::{MidiInput, MidiInputDeviceId};
use std::ptr::NonNull;
use std::rc::Rc;

/// Contains the REAPER functions which must be called in the audio thread only.
///
/// # Design
///
/// Separating this from the main Reaper struct has the following advantages:
///
/// 1. While there's currently no way to make sure at compile time that a function is called in
/// the correct thread, structurally separating the functions should make things more clear.
/// Hopefully this will make it easier to spot incorrect usage or to avoid it in the first place.
///
/// 2. The main REAPER struct contains not just the REAPER function pointers but also some mutable
/// management data, e.g. data for keeping track of things registered via `plugin_register_*()`.
/// Therefore it can't just be copied. So in order to be able to use REAPER functions also from e.g.
/// the audio hook register, we would need to wrap it in `Arc` (not `Rc`, because we access it
/// from multiple threads). That's not enough though for most real-world cases. We probably want to
/// register/unregister things from the main thread not only in the beginning but also at a later
/// time. That means we need mutable access. So we end up with `Arc<Mutex<Reaper>>`. However, why
/// going through all that trouble and put up with possible performance issues if we can avoid it?
/// The RealtimeReaper contains nothing but function pointers, so it's completely standalone and
/// copyable. Memory overhead for one low-level Reaper copy is small (~800 * 8 byte = ~7 kB).
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
