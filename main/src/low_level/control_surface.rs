use std::ptr::null_mut;

use super::MediaTrack;

pub trait IReaperControlSurface {
    fn GetTypeString(&self) -> *const ::std::os::raw::c_char;

    fn GetDescString(&self) -> *const ::std::os::raw::c_char;

    fn GetConfigString(&self) -> *const ::std::os::raw::c_char;

    fn CloseNoReset(&self);

    fn Run(&self);

    fn SetTrackListChange(&self);

    fn SetSurfaceVolume(&self, trackid: *mut MediaTrack, volume: f64);

    fn SetSurfacePan(&self, trackid: *mut MediaTrack, pan: f64);

    fn SetSurfaceMute(&self, trackid: *mut MediaTrack, mute: bool);

    fn SetSurfaceSelected(&self, trackid: *mut MediaTrack, selected: bool);

    fn SetSurfaceSolo(&self, trackid: *mut MediaTrack, solo: bool);

    fn SetSurfaceRecArm(&self, trackid: *mut MediaTrack, recarm: bool);

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool);

    fn SetRepeatState(&self, rep: bool);

    fn SetTrackTitle(
        &self,
        trackid: *mut MediaTrack,
        title: *const ::std::os::raw::c_char,
    );

    fn GetTouchState(
        &self,
        trackid: *mut MediaTrack,
        isPan: ::std::os::raw::c_int,
    ) -> bool;

    fn SetAutoMode(&self, mode: ::std::os::raw::c_int);

    fn ResetCachedVolPanStates(&self);

    fn OnTrackSelection(&self, trackid: *mut MediaTrack);

    fn IsKeyDown(&self, key: ::std::os::raw::c_int) -> bool;

    fn Extended(
        &self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}

#[no_mangle]
extern "C" fn GetTypeString(callback_target: *mut Box<dyn IReaperControlSurface>) -> *const ::std::os::raw::c_char {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.GetTypeString()
}

#[no_mangle]
extern "C" fn GetDescString(callback_target: *mut Box<dyn IReaperControlSurface>) -> *const ::std::os::raw::c_char {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.GetDescString()
}

#[no_mangle]
extern "C" fn GetConfigString(callback_target: *mut Box<dyn IReaperControlSurface>) -> *const ::std::os::raw::c_char {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.GetConfigString()
}

#[no_mangle]
extern "C" fn CloseNoReset(callback_target: *mut Box<dyn IReaperControlSurface>) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.CloseNoReset()
}

#[no_mangle]
extern "C" fn Run(callback_target: *mut Box<dyn IReaperControlSurface>) {
    // "Decode" thin pointer
    // (see https://users.rust-lang.org/t/sending-a-boxed-trait-over-ffi/21708/6)
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.Run()
}

#[no_mangle]
extern "C" fn SetTrackListChange(callback_target: *mut Box<dyn IReaperControlSurface>) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetTrackListChange()
}

#[no_mangle]
extern "C" fn SetSurfaceVolume(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, volume: f64) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfaceVolume(trackid, volume)
}

#[no_mangle]
extern "C" fn SetSurfacePan(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, pan: f64) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfacePan(trackid, pan)
}

#[no_mangle]
extern "C" fn SetSurfaceMute(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, mute: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfaceMute(trackid, mute)
}

#[no_mangle]
extern "C" fn SetSurfaceSelected(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, selected: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfaceSelected(trackid, selected)
}

#[no_mangle]
extern "C" fn SetSurfaceSolo(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, solo: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfaceSolo(trackid, solo)
}

#[no_mangle]
extern "C" fn SetSurfaceRecArm(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack, recarm: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetSurfaceRecArm(trackid, recarm)
}

#[no_mangle]
extern "C" fn SetPlayState(callback_target: *mut Box<dyn IReaperControlSurface>, play: bool, pause: bool, rec: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetPlayState(play, pause, rec)
}

#[no_mangle]
extern "C" fn SetRepeatState(callback_target: *mut Box<dyn IReaperControlSurface>, rep: bool) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetRepeatState(rep)
}

#[no_mangle]
extern "C" fn SetTrackTitle(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    title: *const ::std::os::raw::c_char,
) {

    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetTrackTitle(trackid, title)
}

#[no_mangle]
extern "C" fn GetTouchState(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    trackid: *mut MediaTrack,
    isPan: ::std::os::raw::c_int,
) -> bool {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.GetTouchState(trackid, isPan)
}

#[no_mangle]
extern "C" fn SetAutoMode(callback_target: *mut Box<dyn IReaperControlSurface>, mode: ::std::os::raw::c_int) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.SetAutoMode(mode)
}

#[no_mangle]
extern "C" fn ResetCachedVolPanStates(callback_target: *mut Box<dyn IReaperControlSurface>) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.ResetCachedVolPanStates()
}

#[no_mangle]
extern "C" fn OnTrackSelection(callback_target: *mut Box<dyn IReaperControlSurface>, trackid: *mut MediaTrack) {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.OnTrackSelection(trackid)
}

#[no_mangle]
extern "C" fn IsKeyDown(callback_target: *mut Box<dyn IReaperControlSurface>, key: ::std::os::raw::c_int) -> bool {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.IsKeyDown(key)
}

#[no_mangle]
extern "C" fn Extended(
    callback_target: *mut Box<dyn IReaperControlSurface>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let control_surface = unsafe { Box::from_raw(callback_target) };
    control_surface.Extended(call, parm1, parm2, parm3)
}