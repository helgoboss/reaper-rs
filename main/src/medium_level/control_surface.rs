use crate::low_level::MediaTrack;
use crate::low_level;
use std::ffi::CStr;
use std::borrow::Cow;
use std::os::raw::c_void;

pub trait ControlSurface {
    fn get_type_string(&self) -> Cow<'static, str>;

    fn get_desc_string(&self) -> Cow<'static, str>;

    fn get_config_string(&self) -> Cow<'static, str>;

    fn close_no_reset(&self);

    fn run(&self);

    fn set_track_list_change(&self);

    fn set_surface_volume(&self, trackid: *mut MediaTrack, volume: f64);

    fn set_surface_pan(&self, trackid: *mut MediaTrack, pan: f64);

    fn set_surface_mut(&self, trackid: *mut MediaTrack, mute: bool);

    fn set_surface_selected(&self, trackid: *mut MediaTrack, selected: bool);

    fn set_surface_solo(&self, trackid: *mut MediaTrack, solo: bool);

    fn set_surface_rec_arm(&self, trackid: *mut MediaTrack, recarm: bool);

    fn set_play_state(&self, play: bool, pause: bool, rec: bool);

    fn set_repeat_state(&self, rep: bool);

    fn set_track_title(&self, trackid: *mut MediaTrack, title: &CStr);

    fn get_touch_state(&self, trackid: *mut MediaTrack, is_pan: i32) -> bool;

    fn set_auto_mode(&self, mode: i32);

    fn reset_cached_vol_pan_states(&self);

    fn on_track_selection(&self, trackid: *mut MediaTrack);

    fn is_key_down(&self, key: i32) -> bool;

    fn extended(
        &self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32;
}

pub struct DelegatingIReaperControlSurface {
    delegate: Box<dyn ControlSurface>
}

//impl DelegatingIReaperControlSurface {
//    fn new(delegate: impl ControlSurface) -> DelegatingIReaperControlSurface {
//        DelegatingIReaperControlSurface {
//            delegate: Box::new(delegate)
//        }
//    }
//}

impl low_level::ControlSurface for DelegatingIReaperControlSurface {
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