#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::{firewall, raw::MediaTrack};
use crate::raw;

use downcast_rs::Downcast;
use std::fmt::Debug;
use std::os::raw::c_void;
use std::ptr::{null, null_mut, NonNull};

/// This is the Rust analog to the C++ virtual base class `IReaperControlSurface`.
///
/// An implementation of this trait can be passed to [`create_cpp_to_rust_control_surface()`]. After
/// registering the returned C++ counterpart, REAPER will start invoking the callback methods.
///
/// # Design
///
/// ## Why do most methods here don't take `&mut self` as parameter?
///
/// **Short answer:** Because we follow the spirit of Rust here, which is to fail fast and thereby
/// prevent undefined behavior.
///
/// **Long answer:** Taking `self` as `&mut` in control surface methods would give us a dangerous
/// illusion of safety (safety as defined by Rust). It would tell Rust developers "It's safe here to
/// mutate the state of my control surface struct". But in reality it's not safe. Not because of
/// multi-threading (ControlSurfaces methods are invoked by REAPER's main thread only) but because
/// of reentrancy. That can happen quite easily, just think of this scenario: A track is changed,
/// REAPER notifies us about it by calling a ControlSurface method, thereby causing another change
/// in REAPER which in turn synchronously notifies our ControlSurface again while our first method
/// is still running ... and there you go: 2 mutable borrows of `self`. In a Rust-only world, Rust's
/// compiler wouldn't allow us to do that. But Rust won't save us here because the call comes from
/// "outside". By not having a `&mut self` reference, developers are forced to explicitly think
/// about this scenario. One can use a `RefCell` along with `borrow_mut()` to still mutate some
/// control surface state and failing fast whenever reentrancy happens - at runtime, by getting a
/// panic. This is not as good as failing fast at compile time but still much better than to run
/// into undefined behavior, which could cause hard-to-find bugs and crash REAPER - that's the last
/// thing we want! Panicking is not so bad. We can catch it before it reaches REAPER and therefore
/// let REAPER continue running. Ideally it's observed by the developer when he tests his plugin.
/// Then he can think about how to solve that issue. They might find out that it's okay and
/// therefore use some unsafe code to prevent the panic. They might find out that they want to check
/// for reentrancy by using `try_borrow_mut()`. Or they might find out that they want to
/// avoid this situation by just deferring the event handling to the next main loop cycle.
///
/// [`create_cpp_to_rust_control_surface()`]: fn.create_cpp_to_rust_control_surface.html
pub trait IReaperControlSurface: Debug + Downcast {
    fn GetTypeString(&self) -> *const ::std::os::raw::c_char {
        null()
    }

    fn GetDescString(&self) -> *const ::std::os::raw::c_char {
        null()
    }

    fn GetConfigString(&self) -> *const ::std::os::raw::c_char {
        null()
    }

    fn CloseNoReset(&self) {}

    fn Run(&mut self) {}

    fn SetTrackListChange(&self) {}

    fn SetSurfaceVolume(&self, _trackid: *mut MediaTrack, _volume: f64) {}

    fn SetSurfacePan(&self, _trackid: *mut MediaTrack, _pan: f64) {}

    fn SetSurfaceMute(&self, _trackid: *mut MediaTrack, _mute: bool) {}

    fn SetSurfaceSelected(&self, _trackid: *mut MediaTrack, _selected: bool) {}

    fn SetSurfaceSolo(&self, _trackid: *mut MediaTrack, _solo: bool) {}

    fn SetSurfaceRecArm(&self, _trackid: *mut MediaTrack, _recarm: bool) {}

    fn SetPlayState(&self, _play: bool, _pause: bool, _rec: bool) {}

    fn SetRepeatState(&self, _rep: bool) {}

    fn SetTrackTitle(&self, _trackid: *mut MediaTrack, _title: *const ::std::os::raw::c_char) {}

    fn GetTouchState(&self, _trackid: *mut MediaTrack, _isPan: ::std::os::raw::c_int) -> bool {
        false
    }

    fn SetAutoMode(&self, _mode: ::std::os::raw::c_int) {}

    fn ResetCachedVolPanStates(&self) {}

    fn OnTrackSelection(&self, _trackid: *mut MediaTrack) {}

    fn IsKeyDown(&self, _key: ::std::os::raw::c_int) -> bool {
        false
    }

    fn Extended(
        &self,
        _call: ::std::os::raw::c_int,
        _parm1: *mut ::std::os::raw::c_void,
        _parm2: *mut ::std::os::raw::c_void,
        _parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        0
    }
}

downcast_rs::impl_downcast!(IReaperControlSurface);

/// Creates an `IReaperControlSurface` object on C++ side and returns a pointer to it.
///
/// This function is provided because [`plugin_register()`] isn't going to work if you just pass it
/// a Rust struct as in `reaper.plugin_register("csurf_inst", my_rust_struct)`. Rust structs can't
/// implement C++ virtual base classes.
///
/// **This function doesn't yet register the control surface!** The usual REAPER C++ way to register
/// a control surface still applies. You need to pass the resulting pointer to
/// [`plugin_register()`].
///
/// # Example
///
/// ```no_run
/// # let reaper = reaper_low::Reaper::default();
/// use reaper_low::{create_cpp_to_rust_control_surface, delete_cpp_control_surface, IReaperControlSurface};
/// use std::ffi::CString;
/// use std::ptr::NonNull;
/// use c_str_macro::c_str;
///
/// unsafe {
///     // Register
///     #[derive(Debug)]
///     struct MyControlSurface;
///     impl IReaperControlSurface for MyControlSurface {
///         fn SetTrackListChange(&self) {
///             println!("Tracks changed");
///         }
///     }
///     let rust_cs: Box<dyn IReaperControlSurface> = Box::new(MyControlSurface);
///     let thin_ptr_to_rust_cs: NonNull<_> = (&rust_cs).into();
///     let cpp_cs = create_cpp_to_rust_control_surface(thin_ptr_to_rust_cs);
///     reaper.plugin_register(c_str!("csurf_inst").as_ptr(), cpp_cs.as_ptr() as _);
///     // Unregister
///     reaper.plugin_register(c_str!("-csurf_inst").as_ptr(), cpp_cs.as_ptr() as _);
///     delete_cpp_control_surface(cpp_cs);
/// }
/// ```
///
/// # Cleaning up
///
/// If you register a control surface, you also must take care of unregistering it at
/// the end. This is especially important for VST plug-ins because they live shorter than a REAPER
/// session! **If you don't unregister the control surface before the VST plug-in is destroyed,
/// REAPER will crash** because it will attempt to invoke functions which are not loaded anymore.
///
/// In order to avoid memory leaks, you also must take care of removing the C++ counterpart
/// surface by calling [`delete_cpp_control_surface()`].
///
/// # Safety
///
/// This function is highly unsafe for all the reasons mentioned above. Better use the medium-level
/// API instead, which makes registering a breeze.
///
/// [`plugin_register()`]: struct.Reaper.html#method.plugin_register
/// [`delete_cpp_control_surface()`]: fn.remove_cpp_control_surface.html
pub unsafe fn create_cpp_to_rust_control_surface(
    callback_target: NonNull<Box<dyn IReaperControlSurface>>,
) -> NonNull<raw::IReaperControlSurface> {
    let instance =
        crate::bindings::root::reaper_control_surface::create_cpp_to_rust_control_surface(
            callback_target.as_ptr() as *mut c_void,
        );
    NonNull::new_unchecked(instance)
}

/// Destroys a C++ `IReaperControlSurface` object.
///
/// Intended to be used on pointers returned from [`create_cpp_to_rust_control_surface()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`create_cpp_to_rust_control_surface()`]: fn.create_cpp_to_rust_control_surface.html
pub unsafe fn delete_cpp_control_surface(surface: NonNull<raw::IReaperControlSurface>) {
    crate::bindings::root::reaper_control_surface::delete_control_surface(surface.as_ptr());
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_GetTypeString(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &*callback_target }.GetTypeString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_GetDescString(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &*callback_target }.GetDescString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_GetConfigString(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &*callback_target }.GetConfigString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_CloseNoReset(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) {
    firewall(|| unsafe { &*callback_target }.CloseNoReset());
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_Run(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) {
    // "Decoding" the thin pointer is not necessary right now because we have a static variable.
    // However, we leave it. Might come in handy one day to support multiple control surfaces
    // (see https://users.rust-lang.org/t/sending-a-boxed-trait-over-ffi/21708/6)
    firewall(|| unsafe { &mut *callback_target }.Run());
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetTrackListChange(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) {
    firewall(|| unsafe { &*callback_target }.SetTrackListChange());
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfaceVolume(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    volume: f64,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfaceVolume(trackid, volume));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfacePan(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    pan: f64,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfacePan(trackid, pan));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfaceMute(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    mute: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfaceMute(trackid, mute));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfaceSelected(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    selected: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfaceSelected(trackid, selected));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfaceSolo(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    solo: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfaceSolo(trackid, solo));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetSurfaceRecArm(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    recarm: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetSurfaceRecArm(trackid, recarm));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetPlayState(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    play: bool,
    pause: bool,
    rec: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetPlayState(play, pause, rec));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetRepeatState(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    rep: bool,
) {
    firewall(|| unsafe { &*callback_target }.SetRepeatState(rep));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetTrackTitle(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    title: *const ::std::os::raw::c_char,
) {
    firewall(|| unsafe { &*callback_target }.SetTrackTitle(trackid, title));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_GetTouchState(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    isPan: ::std::os::raw::c_int,
) -> bool {
    firewall(|| unsafe { &*callback_target }.GetTouchState(trackid, isPan)).unwrap_or(false)
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_SetAutoMode(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    mode: ::std::os::raw::c_int,
) {
    firewall(|| unsafe { &*callback_target }.SetAutoMode(mode));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_ResetCachedVolPanStates(
    callback_target: *mut Box<dyn IReaperControlSurface>,
) {
    firewall(|| unsafe { &*callback_target }.ResetCachedVolPanStates());
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_OnTrackSelection(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
) {
    firewall(|| unsafe { &*callback_target }.OnTrackSelection(trackid));
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_IsKeyDown(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    key: ::std::os::raw::c_int,
) -> bool {
    firewall(|| unsafe { &*callback_target }.IsKeyDown(key)).unwrap_or(false)
}

#[no_mangle]
extern "C" fn cpp_to_rust_IReaperControlSurface_Extended(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &*callback_target }.Extended(call, parm1, parm2, parm3)).unwrap_or(0)
}
