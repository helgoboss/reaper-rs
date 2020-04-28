use reaper_rs_low::raw;
use reaper_rs_low::raw::audio_hook_register_t;
use std::ptr::null_mut;

pub(crate) type OnAudioBufferFn =
    extern "C" fn(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t);

pub struct MediumAudioHookRegister {
    inner: raw::audio_hook_register_t,
}

impl MediumAudioHookRegister {
    // TODO-medium create a medium-level abstraction of the function (using generics)
    pub fn new(on_audio_buffer: OnAudioBufferFn) -> MediumAudioHookRegister {
        MediumAudioHookRegister {
            inner: audio_hook_register_t {
                OnAudioBuffer: Some(on_audio_buffer),
                userdata1: null_mut(),
                userdata2: null_mut(),
                input_nch: 0,
                output_nch: 0,
                GetBuffer: None,
            },
        }
    }
}

impl AsRef<raw::audio_hook_register_t> for MediumAudioHookRegister {
    fn as_ref(&self) -> &audio_hook_register_t {
        &self.inner
    }
}
