#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::{
    bindings::root::reaper_rs_control_surface::get_control_surface, firewall, raw::MediaTrack,
};
use crate::raw;

use std::ptr::{null, null_mut};
use std::sync::Once;

/// This is the Rust analog to the C++ virtual base class `IReaperControlSurface`. An implementation
/// of this trait can be passed to [`install_control_surface`](fn.install_control_surface.html).
/// As a consequence, REAPER will invoke the respective callback methods.
///
/// # Design
///
/// ## Why do most methods here don't take `&mut self` as parameter?
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
pub trait IReaperControlSurface {
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

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut CONTROL_SURFACE_INSTANCE: Option<Box<dyn IReaperControlSurface>> = None;
static INIT_CONTROL_SURFACE_INSTANCE: Once = Once::new();

/// This returns a mutable reference. In general this mutability should not be used, just in case
/// of control surface methods where it's sure that REAPER never reenters them! See
/// [`ControlSurface`](trait.ControlSurface.html) documentation.
pub fn get_control_surface_instance() -> &'static mut Box<dyn IReaperControlSurface> {
    unsafe { CONTROL_SURFACE_INSTANCE.as_mut().unwrap() }
}

/// This function for installing a REAPER control surface is provided because
/// `plugin_register("csurf_inst", my_rust_struct)` isn't going to work. Rust structs can't
/// implement C++ virtual base classes.
///
/// This function sets up the given control surface implemented in Rust but **doesn't yet
/// register it**! Because you are not using the high-level API, the usual REAPER C++ way to
/// register a control surface still applies. See
/// [`get_cpp_control_surface`](fn.get_cpp_control_surface.html). If you register a control surface,
/// you also must take care of unregistering it at the end. This is especially important for VST
/// plug-ins because they live shorter than a REAPER session! **If you don't unregister the control
/// surface before the VST plug-in is destroyed, REAPER will crash** because it will attempt to
/// invoke functions which are not loaded anymore. This kind of responsibility is gone when using
/// the high-level API.     
/// Currently *reaper-rs* supports only one control surface per plug-in. This is not a restriction
/// dictated by Rust, it's just a bit easier to implement and I don't see many use cases where one
/// would want multiple control surfaces.
pub fn install_control_surface(control_surface: impl IReaperControlSurface + 'static) {
    // TODO-low Ensure that only called if there's not a control surface registered already
    // Ideally we would have a generic static but as things are now, we need to box it.
    // However, this is not a big deal because control surfaces are only used in the
    // main thread where these minimal performance differences are not significant.
    unsafe {
        // Save boxed control surface to static variable so that extern "C" functions
        // implemented in Rust have something to delegate to.
        INIT_CONTROL_SURFACE_INSTANCE.call_once(|| {
            CONTROL_SURFACE_INSTANCE = Some(Box::new(control_surface));
        });
    }
}

/// TODO-medium Maybe better to return a NonNull pointer?
/// This returns a reference of a `IReaperControlSurface`-implementing C++ object which will
/// delegate to the Rust [`ControlSurface`](trait.ControlSurface.html) which you installed by
/// invoking [`install_control_surface`](fn.install_control_surface.html). It needs to be
/// passed to [`plugin_register`](struct.Reaper.html#structfield.plugin_register) as a pointer as in
/// `plugin_register("csurf_inst", cs as *mut _ as *mut c_void)` for registering and as in
/// `plugin_register("-csurf_inst", cs as *mut _ as *mut c_void)` for unregistering.
pub fn get_cpp_control_surface() -> &'static mut raw::IReaperControlSurface {
    unsafe { &mut *get_control_surface() }
}

#[no_mangle]
extern "C" fn GetTypeString(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| get_control_surface_instance().GetTypeString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn GetDescString(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| get_control_surface_instance().GetDescString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn GetConfigString(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
) -> *const ::std::os::raw::c_char {
    firewall(|| get_control_surface_instance().GetConfigString()).unwrap_or(null_mut())
}

#[no_mangle]
extern "C" fn CloseNoReset(_callback_target: *mut Box<dyn IReaperControlSurface>) {
    firewall(|| get_control_surface_instance().CloseNoReset());
}

#[no_mangle]
extern "C" fn Run(_callback_target: *mut Box<dyn IReaperControlSurface>) {
    // "Decoding" the thin pointer is not necessary right now because we have a static variable.
    // However, we leave it. Might come in handy one day to support multiple control surfaces
    // (see https://users.rust-lang.org/t/sending-a-boxed-trait-over-ffi/21708/6)
    firewall(|| get_control_surface_instance().Run());
}

#[no_mangle]
extern "C" fn SetTrackListChange(_callback_target: *mut Box<dyn IReaperControlSurface>) {
    firewall(|| get_control_surface_instance().SetTrackListChange());
}

#[no_mangle]
extern "C" fn SetSurfaceVolume(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    volume: f64,
) {
    firewall(|| get_control_surface_instance().SetSurfaceVolume(trackid, volume));
}

#[no_mangle]
extern "C" fn SetSurfacePan(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    pan: f64,
) {
    firewall(|| get_control_surface_instance().SetSurfacePan(trackid, pan));
}

#[no_mangle]
extern "C" fn SetSurfaceMute(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    mute: bool,
) {
    firewall(|| get_control_surface_instance().SetSurfaceMute(trackid, mute));
}

#[no_mangle]
extern "C" fn SetSurfaceSelected(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    selected: bool,
) {
    firewall(|| get_control_surface_instance().SetSurfaceSelected(trackid, selected));
}

#[no_mangle]
extern "C" fn SetSurfaceSolo(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    solo: bool,
) {
    firewall(|| get_control_surface_instance().SetSurfaceSolo(trackid, solo));
}

#[no_mangle]
extern "C" fn SetSurfaceRecArm(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    recarm: bool,
) {
    firewall(|| get_control_surface_instance().SetSurfaceRecArm(trackid, recarm));
}

#[no_mangle]
extern "C" fn SetPlayState(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    play: bool,
    pause: bool,
    rec: bool,
) {
    firewall(|| get_control_surface_instance().SetPlayState(play, pause, rec));
}

#[no_mangle]
extern "C" fn SetRepeatState(_callback_target: *mut Box<dyn IReaperControlSurface>, rep: bool) {
    firewall(|| get_control_surface_instance().SetRepeatState(rep));
}

#[no_mangle]
extern "C" fn SetTrackTitle(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    title: *const ::std::os::raw::c_char,
) {
    firewall(|| get_control_surface_instance().SetTrackTitle(trackid, title));
}

#[no_mangle]
extern "C" fn GetTouchState(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    isPan: ::std::os::raw::c_int,
) -> bool {
    firewall(|| get_control_surface_instance().GetTouchState(trackid, isPan)).unwrap_or(false)
}

#[no_mangle]
extern "C" fn SetAutoMode(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    mode: ::std::os::raw::c_int,
) {
    firewall(|| get_control_surface_instance().SetAutoMode(mode));
}

#[no_mangle]
extern "C" fn ResetCachedVolPanStates(_callback_target: *mut Box<dyn IReaperControlSurface>) {
    firewall(|| get_control_surface_instance().ResetCachedVolPanStates());
}

#[no_mangle]
extern "C" fn OnTrackSelection(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
) {
    firewall(|| get_control_surface_instance().OnTrackSelection(trackid));
}

#[no_mangle]
extern "C" fn IsKeyDown(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    key: ::std::os::raw::c_int,
) -> bool {
    firewall(|| get_control_surface_instance().IsKeyDown(key)).unwrap_or(false)
}

#[no_mangle]
extern "C" fn Extended(
    _callback_target: *mut Box<dyn IReaperControlSurface>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    firewall(|| get_control_surface_instance().Extended(call, parm1, parm2, parm3)).unwrap_or(0)
}
