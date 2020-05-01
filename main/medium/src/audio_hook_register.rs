use crate::AudioHookRegister;
use reaper_rs_low::raw::audio_hook_register_t;
use reaper_rs_low::{firewall, raw};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{null_mut, NonNull};

pub(crate) type OnAudioBufferFn =
    extern "C" fn(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t);

/// Consumers need to implement this trait if they want to be called back from the audio thread.
///
/// See [`audio_reg_hardware_hook_add`].
///
/// [`audio_reg_hardware_hook_add`]: struct.Reaper.html#method.audio_reg_hardware_hook_add
pub trait MediumOnAudioBuffer {
    /// Type of the first user-defined piece of data passed to the callback.
    type UserData1;
    /// Type of the second user-defined piece of data passed to the callback.
    type UserData2;

    /// The actual callback function. It's called twice per frame, first with `is_post` being
    /// `false`, then `true`.
    fn call(args: OnAudioBufferArgs<Self::UserData1, Self::UserData2>);
}

#[derive(PartialEq, Debug)]
pub struct OnAudioBufferArgs<U1, U2> {
    pub is_post: bool,
    pub buffer_length: u32,
    // TODO-medium Maybe introduce newtype that makes clear which unit this has
    pub sample_rate: f64,
    // TODO-medium In a similar use case (get_midi_input) a struct is passed by reference. Should
    // it  be the same here?
    pub reg: AudioHookRegister<U1, U2>,
}

pub(crate) extern "C" fn delegating_on_audio_buffer<T: MediumOnAudioBuffer>(
    is_post: bool,
    len: i32,
    srate: f64,
    reg: *mut audio_hook_register_t,
) {
    // TODO-low Check performance implications for firewall call
    firewall(|| {
        T::call(OnAudioBufferArgs {
            is_post,
            buffer_length: len as u32,
            sample_rate: srate,
            reg: AudioHookRegister::new(unsafe { NonNull::new_unchecked(reg) }),
        });
    });
}

/// Consumers need to provide this struct to be called back from the audio thread.
///
/// See [`audio_reg_hardware_hook_add`].
///
/// [`audio_reg_hardware_hook_add`]: struct.Reaper.html#method.audio_reg_hardware_hook_add
#[derive(Debug)]
pub struct MediumAudioHookRegister {
    inner: raw::audio_hook_register_t,
    // Boxed because we need stable memory address in order to pass this to REAPER
    owned_user_data_1: Option<Box<dyn Any>>,
    owned_user_data_2: Option<Box<dyn Any>>,
}

impl MediumAudioHookRegister {
    /// Creates an audio hook register.
    ///
    /// See [`audio_reg_hardware_hook_add`].
    ///
    /// # Design
    ///
    /// Taking ownership of the user-defined piece of data releases the API consumer of the burden
    /// of maintaining a stable memory address and ensuring correct lifetime.
    ///
    /// [`audio_reg_hardware_hook_add`]: struct.Reaper.html#method.audio_reg_hardware_hook_add
    // TODO-medium Maybe use the fact that we need to implement a struct anyway and make it
    //  via self.
    pub fn new<T: MediumOnAudioBuffer<UserData1 = U1, UserData2 = U2>, U1: 'static, U2: 'static>(
        user_data_1: Option<U1>,
        user_data_2: Option<U2>,
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
