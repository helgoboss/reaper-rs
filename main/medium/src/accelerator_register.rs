use crate::{decode_user_data, encode_user_data};
use reaper_low::{firewall, raw};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_int;
use std::ptr::NonNull;

/// Consumers need to implement this trait in order to be called back as part of the keyboard
/// processing.
///
/// See [`plugin_register_add_accelerator_register()`].
///
/// [`plugin_register_add_accelerator_register()`]: struct.ReaperSession.html#method.plugin_register_add_accelerator_register
pub trait TranslateAccel {
    /// The actual callback function.
    fn call(&mut self, args: TranslateAccelArgs) -> TranslateAccelResult;
}

#[derive(PartialEq, Debug)]
pub struct TranslateAccelArgs<'a> {
    // TODO-high medium-level message
    pub msg: raw::MSG,
    pub ctx: &'a AcceleratorRegister,
}

/// Describes what to do with the received keystroke.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TranslateAccelResult {
    /// Not our window.
    NotOurWindow,
    /// Eats the keystroke.
    Eat,
    /// Passes the keystroke on to the window.
    PassOnToWindow,
    /// Processes the event raw (macOS only).
    ProcessEventRaw,
    /// Passes the keystroke to the window, even if it is `WM_SYSKEY*`/`VK_MENU` which would
    /// otherwise be dropped (Windows only).
    ForcePassOnToWindow,
    /// Forces it to the main window's accel table (with the exception of `ESC`).
    ForceToMainWindowAccelTable,
    /// Forces it to the main window's accel table, even if in a text field (5.24+ or so).
    ForceToMainWindowAccelTableEvenIfTextField,
}

impl TranslateAccelResult {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use TranslateAccelResult::*;
        match self {
            NotOurWindow => 0,
            Eat => 1,
            PassOnToWindow => -1,
            ProcessEventRaw => -10,
            ForcePassOnToWindow => -20,
            ForceToMainWindowAccelTable => -666,
            ForceToMainWindowAccelTableEvenIfTextField => -667,
        }
    }
}

extern "C" fn delegating_translate_accel<T: TranslateAccel>(
    msg: *mut raw::MSG,
    ctx: *mut raw::accelerator_register_t,
) -> c_int {
    firewall(|| {
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        let callback_struct: &mut T = decode_user_data(unsafe { ctx.as_ref() }.user);
        callback_struct
            .call(TranslateAccelArgs {
                msg: unsafe { *msg },
                ctx: &AcceleratorRegister::new(ctx),
            })
            .to_raw()
    })
    .unwrap_or(0)
}

/// A record which lets one get a place in the keyboard processing queue.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
//
// It's important that this type is not cloneable! Otherwise consumers could easily let it escape
// its intended usage scope, which would lead to undefined behavior.
//
// We don't expose the user-defined data pointer. It's already exposed implicitly as `&mut self` in
// the callback function.
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct AcceleratorRegister(pub(crate) NonNull<raw::accelerator_register_t>);

impl AcceleratorRegister {
    pub(crate) fn new(ptr: NonNull<raw::accelerator_register_t>) -> Self {
        Self(ptr)
    }

    /// Returns the raw pointer.
    pub fn get(&self) -> NonNull<raw::accelerator_register_t> {
        self.0
    }
}

pub(crate) struct OwnedAcceleratorRegister {
    inner: raw::accelerator_register_t,
    callback: Box<dyn TranslateAccel>,
}

impl Debug for OwnedAcceleratorRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TranslateAccel doesn't generally implement Debug.
        f.debug_struct("OwnedAcceleratorRegister")
            .field("inner", &self.inner)
            .field("callback", &"<omitted>")
            .finish()
    }
}

impl OwnedAcceleratorRegister {
    pub fn new<T>(callback: Box<T>) -> Self
    where
        T: TranslateAccel + 'static,
    {
        Self {
            inner: raw::accelerator_register_t {
                translateAccel: Some(delegating_translate_accel::<T>),
                isLocal: true,
                user: encode_user_data(&callback),
            },
            callback,
        }
    }

    pub fn into_callback(self) -> Box<dyn TranslateAccel> {
        self.callback
    }
}

impl AsRef<raw::accelerator_register_t> for OwnedAcceleratorRegister {
    fn as_ref(&self) -> &raw::accelerator_register_t {
        &self.inner
    }
}
