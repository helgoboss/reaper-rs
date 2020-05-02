use crate::Hertz;
use reaper_rs_low::raw::audio_hook_register_t;
use reaper_rs_low::{firewall, raw};
use std::any::Any;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{null_mut, NonNull};

/// Consumers need to implement this trait in order to be called back in the audio thread.
///
/// See [`audio_reg_hardware_hook_add`].
///
/// [`audio_reg_hardware_hook_add`]: struct.Reaper.html#method.audio_reg_hardware_hook_add
pub trait MediumOnAudioBuffer {
    /// The actual callback function.
    ///
    /// It's called twice per frame, first with `is_post` being `false`, then `true`.
    fn call(&mut self, args: OnAudioBufferArgs);
}

#[derive(PartialEq, Debug)]
pub struct OnAudioBufferArgs<'a> {
    pub is_post: bool,
    pub len: u32,
    pub srate: Hertz,
    pub reg: &'a AudioHookRegister,
}

/// Provides access to the current audio buffer contents (not yet implemented).
//
// It's important that this type is not cloneable! Otherwise consumers could easily let it escape
// its intended usage scope (audio hook), which would lead to undefined behavior.
//
// We don't expose the user-defined data pointers. The first one is already exposed implicitly as
// `&mut self` in the callback function. The second one is unnecessary.
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct AudioHookRegister(pub(crate) NonNull<raw::audio_hook_register_t>);

impl AudioHookRegister {
    pub(crate) fn new(ptr: NonNull<raw::audio_hook_register_t>) -> AudioHookRegister {
        AudioHookRegister(ptr)
    }

    /// Returns the raw pointer.
    pub fn get(&self) -> NonNull<raw::audio_hook_register_t> {
        self.0
    }

    /// Returns the current number of input channels.
    pub fn input_nch(&self) -> u32 {
        unsafe { self.0.as_ref() }.input_nch as u32
    }

    /// Returns the current number of output channels.
    pub fn output_nch(&self) -> u32 {
        unsafe { self.0.as_ref() }.input_nch as u32
    }
}

pub(crate) type OnAudioBufferFn =
    extern "C" fn(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t);

pub(crate) extern "C" fn delegating_on_audio_buffer<T: MediumOnAudioBuffer>(
    is_post: bool,
    len: i32,
    srate: f64,
    reg: *mut audio_hook_register_t,
) {
    // TODO-low Check performance implications for firewall call
    firewall(|| {
        let reg = unsafe { NonNull::new_unchecked(reg) };
        let callback_struct: &mut T = decode_user_data(unsafe { reg.as_ref() }.userdata1);
        callback_struct.call(OnAudioBufferArgs {
            is_post,
            len: len as u32,
            srate: unsafe { Hertz::new_unchecked(srate) },
            reg: &AudioHookRegister::new(reg),
        });
    });
}

fn encode_user_data<U>(data: &Box<U>) -> *mut c_void {
    data.as_ref() as *const _ as *mut c_void
}

fn decode_user_data<'a, U>(data: *mut c_void) -> &'a mut U {
    assert!(!data.is_null());
    let data = data as *mut U;
    unsafe { &mut *data }
}

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
        let callback = Box::new(callback);
        MediumAudioHookRegister {
            inner: audio_hook_register_t {
                OnAudioBuffer: Some(delegating_on_audio_buffer::<T>),
                // boxed_callback_struct is not a fat pointer. Even if it would be, thanks to
                // generics the callback knows what's the concrete type and therefore can restore
                // the original type correctly without needing the vtable part of the fat
                // pointer.
                userdata1: encode_user_data(&callback),
                userdata2: null_mut(),
                input_nch: 0,
                output_nch: 0,
                GetBuffer: None,
            },
            owned_user_data_1: Some(callback),
            owned_user_data_2: None,
        }
    }
}

impl AsRef<raw::audio_hook_register_t> for MediumAudioHookRegister {
    fn as_ref(&self) -> &audio_hook_register_t {
        &self.inner
    }
}
