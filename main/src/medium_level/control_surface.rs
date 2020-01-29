use crate::low_level::MediaTrack;
use std::ptr::null_mut;

// TODO This should probably be moved to low_level API!
pub trait ControlSurface {
    fn run(&self);

}

#[no_mangle]
extern "C" fn GetTypeString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    null_mut()
}

#[no_mangle]
extern "C" fn GetDescString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    null_mut()
}

#[no_mangle]
extern "C" fn GetConfigString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    null_mut()
}

#[no_mangle]
extern "C" fn CloseNoReset(callback_target: *mut Box<dyn ControlSurface>) {
}

#[no_mangle]
extern "C" fn Run(callback_target: *mut Box<dyn ControlSurface>) {
    // "Decode" thin pointer
    // (see https://users.rust-lang.org/t/sending-a-boxed-trait-over-ffi/21708/6)
    let control_surface = unsafe { Box::from_raw(callback_target)};
    control_surface.run();
}

#[no_mangle]
extern "C" fn SetTrackListChange(callback_target: *mut Box<dyn ControlSurface>) {
}

#[no_mangle]
extern "C" fn SetSurfaceVolume(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, volume: f64) {
}

#[no_mangle]
extern "C" fn SetSurfacePan(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, pan: f64) {
}

#[no_mangle]
extern "C" fn SetSurfaceMute(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, mute: bool) {
}

#[no_mangle]
extern "C" fn SetSurfaceSelected(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, selected: bool) {
}

#[no_mangle]
extern "C" fn SetSurfaceSolo(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, solo: bool) {
}

#[no_mangle]
extern "C" fn SetSurfaceRecArm(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, recarm: bool) {
}

#[no_mangle]
extern "C" fn SetPlayState(callback_target: *mut Box<dyn ControlSurface>, play: bool, pause: bool, rec: bool) {
}

#[no_mangle]
extern "C" fn SetRepeatState(callback_target: *mut Box<dyn ControlSurface>, rep: bool) {
}

#[no_mangle]
extern "C" fn SetTrackTitle(
    callback_target: *mut Box<dyn ControlSurface>,
    trackid: *mut MediaTrack,
    title: *const ::std::os::raw::c_char,
) {
}

#[no_mangle]
extern "C" fn GetTouchState(
    callback_target: *mut Box<dyn ControlSurface>,
    trackid: *mut MediaTrack,
    isPan: ::std::os::raw::c_int,
) -> bool {
    false
}

#[no_mangle]
extern "C" fn SetAutoMode(callback_target: *mut Box<dyn ControlSurface>, mode: ::std::os::raw::c_int) {
}

#[no_mangle]
extern "C" fn ResetCachedVolPanStates(callback_target: *mut Box<dyn ControlSurface>) {
}

#[no_mangle]
extern "C" fn OnTrackSelection(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack) {
}

#[no_mangle]
extern "C" fn IsKeyDown(callback_target: *mut Box<dyn ControlSurface>, key: ::std::os::raw::c_int) -> bool {
    false
}

#[no_mangle]
extern "C" fn Extended(
    callback_target: *mut Box<dyn ControlSurface>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    0
}