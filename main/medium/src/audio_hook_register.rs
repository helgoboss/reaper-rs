use crate::{AudioHookRegister, Hertz};
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
    /// The actual callback function. It's called twice per frame, first with `is_post` being
    /// `false`, then `true`.
    fn call(&mut self, args: OnAudioBufferArgs);
}

// TODO-medium It's cool to be able to use the user-defined data as self. But we still need to
//  offer access to other data contained in AudioHookRegister.
//  user-defined data is owned by us can be be manipulated ad lib by us. Other data has different
// nature:
//  - input_nch, output_nch => set by REAPER, can be different in each call
//  - GetBuffer() exposes samples
#[derive(PartialEq, Debug)]
pub struct OnAudioBufferArgs {
    pub is_post: bool,
    pub buffer_length: u32,
    pub sample_rate: Hertz,
    // pub reg: AudioHookRegister<U1, U2>,
}

pub(crate) extern "C" fn delegating_on_audio_buffer<T: MediumOnAudioBuffer>(
    is_post: bool,
    len: i32,
    srate: f64,
    reg: *mut audio_hook_register_t,
) {
    // TODO-low Check performance implications for firewall call
    firewall(|| {
        let reg: AudioHookRegister<_, ()> =
            AudioHookRegister::new(unsafe { NonNull::new_unchecked(reg) });
        T::call(
            reg.user_data_1(),
            OnAudioBufferArgs {
                is_post,
                buffer_length: len as u32,
                // TODO-medium Turn to new_unchecked as soon as we are pretty sure that it can only
                //  be > 0
                sample_rate: Hertz::new(srate),
            },
        );
    });
}

/// Consumers need to provide this struct to be called back from the audio thread.
///
/// See [`audio_reg_hardware_hook_add`].
///
/// [`audio_reg_hardware_hook_add`]: struct.Reaper.html#method.audio_reg_hardware_hook_add
#[derive(Debug)]
pub(crate) struct MediumAudioHookRegister {
    inner: raw::audio_hook_register_t,
    // Boxed because we need stable memory address in order to pass this to REAPER. `dyn  Any`
    // because we don't want this struct to be generic. It must be possible to keep instances of
    // this struct in a collection which carries different types of user data (because the consumer
    // might want to register multiple different audio hooks).
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
    pub(crate) fn new<T: MediumOnAudioBuffer + 'static>(callback: T) -> MediumAudioHookRegister {
        let boxed_callback_struct = Box::new(callback);
        MediumAudioHookRegister {
            inner: audio_hook_register_t {
                OnAudioBuffer: Some(delegating_on_audio_buffer::<T>),
                // boxed_callback_struct is not a fat pointer. Even if it would be, thanks to
                // generics the callback knows what's the concrete type and therefore can restore
                // the original pointer correctly without needing the vtable part of the fat
                // pointer.
                userdata1: boxed_callback_struct.as_ref() as *const _ as *mut c_void,
                userdata2: null_mut(),
                input_nch: 0,
                output_nch: 0,
                GetBuffer: None,
            },
            owned_user_data_1: Some(boxed_callback_struct),
            owned_user_data_2: None,
        }
    }
}

impl AsRef<raw::audio_hook_register_t> for MediumAudioHookRegister {
    fn as_ref(&self) -> &audio_hook_register_t {
        &self.inner
    }
}
