use crate::AudioHookRegister;
use reaper_rs_low::raw::audio_hook_register_t;
use reaper_rs_low::{firewall, raw};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{null_mut, NonNull};

pub(crate) type OnAudioBufferFn =
    extern "C" fn(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t);

pub trait MediumOnAudioBuffer {
    type UserData1;
    type UserData2;

    fn call(
        is_post: bool,
        len: i32,
        srate: f64,
        reg: AudioHookRegister<Self::UserData1, Self::UserData2>,
    );
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
    // Boxed because we need stable memory address to be passed to REAPER
    owned_user_data_1: Option<Box<dyn Any>>,
    owned_user_data_2: Option<Box<dyn Any>>,
}

impl MediumAudioHookRegister {
    // TODO-low How to handle the second function and setting one to None?
    pub fn new<
        T: MediumOnAudioBuffer<UserData1 = UD1, UserData2 = UD2>,
        UD1: 'static,
        UD2: 'static,
    >(
        user_data_1: Option<UD1>,
        user_data_2: Option<UD2>,
    ) -> MediumAudioHookRegister {
        let user_data_1: Option<Box<dyn Any>> = match user_data_1 {
            None => None,
            Some(ud) => Some(Box::new(ud)),
        };
        let user_data_2: Option<Box<dyn Any>> = match user_data_2 {
            None => None,
            Some(ud) => Some(Box::new(ud)),
        };
        MediumAudioHookRegister {
            inner: audio_hook_register_t {
                OnAudioBuffer: Some(delegating_on_audio_buffer::<T>),
                userdata1: user_data_1
                    .as_ref()
                    .map(|ud| ud.as_ref() as *const _ as *mut c_void)
                    .unwrap_or(null_mut()),
                userdata2: user_data_2
                    .as_ref()
                    .map(|ud| ud.as_ref() as *const _ as *mut c_void)
                    .unwrap_or(null_mut()),
                input_nch: 0,
                output_nch: 0,
                GetBuffer: None,
            },
            owned_user_data_1: user_data_1,
            owned_user_data_2: user_data_2,
        }
    }
}

impl AsRef<raw::audio_hook_register_t> for MediumAudioHookRegister {
    fn as_ref(&self) -> &audio_hook_register_t {
        &self.inner
    }
}
