use crate::AudioHookRegister;
use reaper_rs_low::raw::audio_hook_register_t;
use reaper_rs_low::{firewall, raw};
use std::ptr::{null_mut, NonNull};

pub(crate) type OnAudioBufferFn =
    extern "C" fn(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t);

pub trait MediumOnAudioBuffer {
    fn call(is_post: bool, len: i32, srate: f64, reg: AudioHookRegister);
}

pub(crate) extern "C" fn delegating_on_audio_buffer<T: MediumOnAudioBuffer>(
    is_post: bool,
    len: i32,
    srate: f64,
    reg: *mut audio_hook_register_t,
) {
    // TODO-low Check performance implications for firewall call
    firewall(|| {
        T::call(
            is_post,
            len,
            srate,
            AudioHookRegister::new(unsafe { NonNull::new_unchecked(reg) }),
        )
    });
}

pub struct MediumAudioHookRegister {
    inner: raw::audio_hook_register_t,
}

impl MediumAudioHookRegister {
    // TODO-low How to handle the second function and setting one to None?
    pub fn new<T: MediumOnAudioBuffer>() -> MediumAudioHookRegister {
        MediumAudioHookRegister {
            inner: audio_hook_register_t {
                OnAudioBuffer: Some(delegating_on_audio_buffer::<T>),
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
