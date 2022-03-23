use crate::{AcceleratorBehavior, decode_user_data, encode_user_data, Hidden, Hwnd, VirtKey};
use reaper_low::{firewall, raw};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_int;
use std::ptr::NonNull;
use enumflags2::BitFlags;

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TranslateAccelArgs<'a> {
    // TODO-high medium-level message
    // loword(wparam) translates to key param: VirtKey (VK constants)
    // loword(lparam) translates to f_virt (some modifier bitmap, at least on macOS)
    pub msg: AccelMsg,
    pub ctx: &'a AcceleratorRegister,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AccelMsg {
    // TODO-high Can this really be null?
    pub window: Option<Hwnd>,
    // TODO-high Use enum KeyDown, KeyUp etc. and unknown, extract from loword
    pub message: AccelMsgKind,
    pub behavior: BitFlags<AcceleratorBehavior>,
    pub key: AcceleratorKey,
    /// Milliseconds since system started.
    pub time: u32,
    pub point: Point,
}

impl AccelMsg {
    pub(crate) fn from_raw(msg: &raw::MSG) -> Self {
        let behavior = BitFlags::from_bits_truncate(loword(msg.lParam) as u8);
        Self {
            window: Hwnd::new(msg.hwnd),
            message: AccelMsgKind::from_raw(msg.message),
            behavior,
            key: AcceleratorKey::from_raw(loword(msg.wParam as isize), behavior),
            time: msg.time,
            point: Point::from_raw(msg.pt)
        }
    }
}

fn loword(v: isize) -> u16 {
    (v & 0xffff) as _
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AccelMsgKind {
    KeyDown,
    KeyUp,
    Char,
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<u32>),
}

impl AccelMsgKind {
    pub(crate) fn from_raw(v: u32) -> Self {
        use AccelMsgKind::*;
        match v {
            raw::WM_KEYDOWN => KeyDown,
            raw::WM_KEYUP => KeyUp,
            raw::WM_CHAR => Char,
            v => Unknown(Hidden(v))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Accel {
    pub f_virt: BitFlags<AcceleratorBehavior>,
    pub key: AcceleratorKey,
    pub cmd: u16,
}

impl Accel {
    pub(crate) fn to_raw(&self) -> raw::ACCEL {
        raw::ACCEL {
            fVirt: self.f_virt.bits(),
            key: self.key.to_raw(),
            cmd: self.cmd
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AcceleratorKey {
    VirtKey(VirtKey),
    // TODO-high Can this be more than u8?
    Character(u8)
}

impl AcceleratorKey {
    pub(crate) fn from_raw(v: u16, f_virt: BitFlags<AcceleratorBehavior>) -> Self {
        if f_virt.contains(AcceleratorBehavior::VirtKey) {
            Self::VirtKey(VirtKey::new(v as i32))
        } else {
            Self::Character(v as u8)
        }
    }

    pub(crate) fn to_raw(&self) -> u16 {
        use AcceleratorKey::*;
        match self {
            // TODO-high Is VirtKey i32 too broad?
            VirtKey(key) => key.get() as u16,
            Character(ch) => *ch as u16,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub(crate)  fn from_raw(v: raw::POINT) -> Self {
        Self {
            x: v.x as u32,
            y: v.y as u32
        }
    }
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
        let msg = AccelMsg::from_raw(unsafe { &*msg });
        callback_struct
            .call(TranslateAccelArgs {
                msg,
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
