use std::ptr::{null_mut, null};

use super::MediaTrack;
use crate::low_level::get_control_surface_instance;

pub trait ControlSurface {
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

    fn Run(&self) {}

    fn SetTrackListChange(&self) {}

    fn SetSurfaceVolume(&self, trackid: *mut MediaTrack, volume: f64) {}

    fn SetSurfacePan(&self, trackid: *mut MediaTrack, pan: f64) {}

    fn SetSurfaceMute(&self, trackid: *mut MediaTrack, mute: bool) {}

    fn SetSurfaceSelected(&self, trackid: *mut MediaTrack, selected: bool) {}

    fn SetSurfaceSolo(&self, trackid: *mut MediaTrack, solo: bool) {}

    fn SetSurfaceRecArm(&self, trackid: *mut MediaTrack, recarm: bool) {}

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {}

    fn SetRepeatState(&self, rep: bool) {}

    fn SetTrackTitle(
        &self,
        trackid: *mut MediaTrack,
        title: *const ::std::os::raw::c_char,
    ) {}

    fn GetTouchState(
        &self,
        trackid: *mut MediaTrack,
        isPan: ::std::os::raw::c_int,
    ) -> bool {
        false
    }

    fn SetAutoMode(&self, mode: ::std::os::raw::c_int) {}

    fn ResetCachedVolPanStates(&self) {}

    fn OnTrackSelection(&self, trackid: *mut MediaTrack) {}

    fn IsKeyDown(&self, key: ::std::os::raw::c_int) -> bool {
        false
    }

    fn Extended(
        &self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        0
    }
}

#[no_mangle]
extern "C" fn GetTypeString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    get_control_surface_instance().GetTypeString()
}

#[no_mangle]
extern "C" fn GetDescString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    get_control_surface_instance().GetDescString()
}

#[no_mangle]
extern "C" fn GetConfigString(callback_target: *mut Box<dyn ControlSurface>) -> *const ::std::os::raw::c_char {
    get_control_surface_instance().GetConfigString()
}

#[no_mangle]
extern "C" fn CloseNoReset(callback_target: *mut Box<dyn ControlSurface>) {
    get_control_surface_instance().CloseNoReset()
}

#[no_mangle]
extern "C" fn Run(callback_target: *mut Box<dyn ControlSurface>) {
    // "Decoding" the thin pointer is not necessary right now because we have a static variable.
    // However, we leave it. Might come in handy one day to support multiple control surfaces
    // (see https://users.rust-lang.org/t/sending-a-boxed-trait-over-ffi/21708/6)
    get_control_surface_instance().Run()
}

#[no_mangle]
extern "C" fn SetTrackListChange(callback_target: *mut Box<dyn ControlSurface>) {
    get_control_surface_instance().SetTrackListChange()
}

#[no_mangle]
extern "C" fn SetSurfaceVolume(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, volume: f64) {
    get_control_surface_instance().SetSurfaceVolume(trackid, volume)
}

#[no_mangle]
extern "C" fn SetSurfacePan(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, pan: f64) {
    get_control_surface_instance().SetSurfacePan(trackid, pan)
}

#[no_mangle]
extern "C" fn SetSurfaceMute(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, mute: bool) {
    get_control_surface_instance().SetSurfaceMute(trackid, mute)
}

#[no_mangle]
extern "C" fn SetSurfaceSelected(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, selected: bool) {
    get_control_surface_instance().SetSurfaceSelected(trackid, selected)
}

#[no_mangle]
extern "C" fn SetSurfaceSolo(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, solo: bool) {
    get_control_surface_instance().SetSurfaceSolo(trackid, solo)
}

#[no_mangle]
extern "C" fn SetSurfaceRecArm(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack, recarm: bool) {
    get_control_surface_instance().SetSurfaceRecArm(trackid, recarm)
}

#[no_mangle]
extern "C" fn SetPlayState(callback_target: *mut Box<dyn ControlSurface>, play: bool, pause: bool, rec: bool) {
    get_control_surface_instance().SetPlayState(play, pause, rec)
}

#[no_mangle]
extern "C" fn SetRepeatState(callback_target: *mut Box<dyn ControlSurface>, rep: bool) {
    get_control_surface_instance().SetRepeatState(rep)
}

#[no_mangle]
extern "C" fn SetTrackTitle(
    callback_target: *mut Box<dyn ControlSurface>,
    trackid: *mut MediaTrack,
    title: *const ::std::os::raw::c_char,
) {
    get_control_surface_instance().SetTrackTitle(trackid, title)
}

#[no_mangle]
extern "C" fn GetTouchState(
    callback_target: *mut Box<dyn ControlSurface>,
    trackid: *mut MediaTrack,
    isPan: ::std::os::raw::c_int,
) -> bool {
    get_control_surface_instance().GetTouchState(trackid, isPan)
}

#[no_mangle]
extern "C" fn SetAutoMode(callback_target: *mut Box<dyn ControlSurface>, mode: ::std::os::raw::c_int) {
    get_control_surface_instance().SetAutoMode(mode)
}

#[no_mangle]
extern "C" fn ResetCachedVolPanStates(callback_target: *mut Box<dyn ControlSurface>) {
    get_control_surface_instance().ResetCachedVolPanStates()
}

#[no_mangle]
extern "C" fn OnTrackSelection(callback_target: *mut Box<dyn ControlSurface>, trackid: *mut MediaTrack) {
    get_control_surface_instance().OnTrackSelection(trackid)
}

#[no_mangle]
extern "C" fn IsKeyDown(callback_target: *mut Box<dyn ControlSurface>, key: ::std::os::raw::c_int) -> bool {
    get_control_surface_instance().IsKeyDown(key)
}

#[no_mangle]
extern "C" fn Extended(
    callback_target: *mut Box<dyn ControlSurface>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    get_control_surface_instance().Extended(call, parm1, parm2, parm3)
}