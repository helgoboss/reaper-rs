use std::os::raw::c_void;
use crate::low_level::MediaTrack;
use crate::medium_level::ControlSurface;
use std::ffi::CStr;
use std::borrow::Cow;

pub struct HelperControlSurface {
}

impl ControlSurface for HelperControlSurface {
    fn get_type_string(&self) -> Cow<'static, str> {
        unimplemented!()
    }

    fn get_desc_string(&self) -> Cow<'static, str> {
        unimplemented!()
    }

    fn get_config_string(&self) -> Cow<'static, str> {
        unimplemented!()
    }

    fn close_no_reset(&self) {
        unimplemented!()
    }

    fn run(&self) {
        unimplemented!()
    }

    fn set_track_list_change(&self) {
        unimplemented!()
    }

    fn set_surface_volume(&self, trackid: *mut MediaTrack, volume: f64) {
        unimplemented!()
    }

    fn set_surface_pan(&self, trackid: *mut MediaTrack, pan: f64) {
        unimplemented!()
    }

    fn set_surface_mut(&self, trackid: *mut MediaTrack, mute: bool) {
        unimplemented!()
    }

    fn set_surface_selected(&self, trackid: *mut MediaTrack, selected: bool) {
        unimplemented!()
    }

    fn set_surface_solo(&self, trackid: *mut MediaTrack, solo: bool) {
        unimplemented!()
    }

    fn set_surface_rec_arm(&self, trackid: *mut MediaTrack, recarm: bool) {
        unimplemented!()
    }

    fn set_play_state(&self, play: bool, pause: bool, rec: bool) {
        unimplemented!()
    }

    fn set_repeat_state(&self, rep: bool) {
        unimplemented!()
    }

    fn set_track_title(&self, trackid: *mut MediaTrack, title: &CStr) {
        unimplemented!()
    }

    fn get_touch_state(&self, trackid: *mut MediaTrack, is_pan: i32) -> bool {
        unimplemented!()
    }

    fn set_auto_mode(&self, mode: i32) {
        unimplemented!()
    }

    fn reset_cached_vol_pan_states(&self) {
        unimplemented!()
    }

    fn on_track_selection(&self, trackid: *mut MediaTrack) {
        unimplemented!()
    }

    fn is_key_down(&self, key: i32) -> bool {
        unimplemented!()
    }

    fn extended(&self, call: i32, parm1: *mut c_void, parm2: *mut c_void, parm3: *mut c_void) -> i32 {
        unimplemented!()
    }
}