use crate::medium_level::ControlSurface;
use std::os::raw::c_void;
use crate::low_level::MediaTrack;

pub struct HelperControlSurface {
}

impl ControlSurface for HelperControlSurface {
    fn GetTypeString(&self) -> *const i8 {
        unimplemented!()
    }

    fn GetDescString(&self) -> *const i8 {
        unimplemented!()
    }

    fn GetConfigString(&self) -> *const i8 {
        unimplemented!()
    }

    fn CloseNoReset(&self) {
        unimplemented!()
    }

    fn Run(&self) {
        unimplemented!()
    }

    fn SetTrackListChange(&self) {
        unimplemented!()
    }

    fn SetSurfaceVolume(&self, trackid: *mut MediaTrack, volume: f64) {
        unimplemented!()
    }

    fn SetSurfacePan(&self, trackid: *mut MediaTrack, pan: f64) {
        unimplemented!()
    }

    fn SetSurfaceMute(&self, trackid: *mut MediaTrack, mute: bool) {
        unimplemented!()
    }

    fn SetSurfaceSelected(&self, trackid: *mut MediaTrack, selected: bool) {
        unimplemented!()
    }

    fn SetSurfaceSolo(&self, trackid: *mut MediaTrack, solo: bool) {
        unimplemented!()
    }

    fn SetSurfaceRecArm(&self, trackid: *mut MediaTrack, recarm: bool) {
        unimplemented!()
    }

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {
        unimplemented!()
    }

    fn SetRepeatState(&self, rep: bool) {
        unimplemented!()
    }

    fn SetTrackTitle(&self, trackid: *mut MediaTrack, title: *const i8) {
        unimplemented!()
    }

    fn GetTouchState(&self, trackid: *mut MediaTrack, isPan: i32) -> bool {
        unimplemented!()
    }

    fn SetAutoMode(&self, mode: i32) {
        unimplemented!()
    }

    fn ResetCachedVolPanStates(&self) {
        unimplemented!()
    }

    fn OnTrackSelection(&self, trackid: *mut MediaTrack) {
        unimplemented!()
    }

    fn IsKeyDown(&self, key: i32) -> bool {
        unimplemented!()
    }

    fn Extended(&self, call: i32, parm1: *mut c_void, parm2: *mut c_void, parm3: *mut c_void) -> i32 {
        unimplemented!()
    }
}