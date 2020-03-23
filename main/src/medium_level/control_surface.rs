use crate::low_level;
use crate::low_level::MediaTrack;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;

pub trait ControlSurface {
    fn get_type_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn get_desc_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn get_config_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn close_no_reset(&self) {}

    fn run(&mut self) {}

    fn set_track_list_change(&self) {}

    fn set_surface_volume(&self, _trackid: *mut MediaTrack, _volume: f64) {}

    fn set_surface_pan(&self, _trackid: *mut MediaTrack, _pan: f64) {}

    fn set_surface_mute(&self, _trackid: *mut MediaTrack, _mute: bool) {}

    fn set_surface_selected(&self, _trackid: *mut MediaTrack, _selected: bool) {}

    fn set_surface_solo(&self, _trackid: *mut MediaTrack, _solo: bool) {}

    fn set_surface_rec_arm(&self, _trackid: *mut MediaTrack, _recarm: bool) {}

    fn set_play_state(&self, _play: bool, _pause: bool, _rec: bool) {}

    fn set_repeat_state(&self, _rep: bool) {}

    fn set_track_title(&self, _trackid: *mut MediaTrack, _title: &CStr) {}

    fn get_touch_state(&self, _trackid: *mut MediaTrack, _is_pan: i32) -> bool {
        false
    }

    fn set_auto_mode(&self, _mode: i32) {}

    fn reset_cached_vol_pan_states(&self) {}

    fn on_track_selection(&self, _trackid: *mut MediaTrack) {}

    fn is_key_down(&self, _key: i32) -> bool {
        false
    }

    fn extended(
        &self,
        _call: i32,
        _parm1: *mut c_void,
        _parm2: *mut c_void,
        _parm3: *mut c_void,
    ) -> i32 {
        0
    }
}

pub struct DelegatingControlSurface<T: ControlSurface> {
    delegate: T,
}

impl<T: ControlSurface> DelegatingControlSurface<T> {
    pub fn new(delegate: T) -> DelegatingControlSurface<T> {
        DelegatingControlSurface { delegate }
    }
}

#[allow(non_snake_case)]
impl<T: ControlSurface> low_level::ControlSurface for DelegatingControlSurface<T> {
    fn GetTypeString(&self) -> *const i8 {
        self.delegate
            .get_type_string()
            .map(|o| o.as_ptr())
            .unwrap_or(null_mut())
    }

    fn GetDescString(&self) -> *const i8 {
        self.delegate
            .get_desc_string()
            .map(|o| o.as_ptr())
            .unwrap_or(null_mut())
    }

    fn GetConfigString(&self) -> *const i8 {
        self.delegate
            .get_config_string()
            .map(|o| o.as_ptr())
            .unwrap_or(null_mut())
    }

    fn CloseNoReset(&self) {
        self.delegate.close_no_reset()
    }

    fn Run(&mut self) {
        self.delegate.run()
    }

    fn SetTrackListChange(&self) {
        self.delegate.set_track_list_change()
    }

    fn SetSurfaceVolume(&self, trackid: *mut MediaTrack, volume: f64) {
        self.delegate.set_surface_volume(trackid, volume)
    }

    fn SetSurfacePan(&self, trackid: *mut MediaTrack, pan: f64) {
        self.delegate.set_surface_pan(trackid, pan)
    }

    fn SetSurfaceMute(&self, trackid: *mut MediaTrack, mute: bool) {
        self.delegate.set_surface_mute(trackid, mute)
    }

    fn SetSurfaceSelected(&self, trackid: *mut MediaTrack, selected: bool) {
        self.delegate.set_surface_selected(trackid, selected)
    }

    fn SetSurfaceSolo(&self, trackid: *mut MediaTrack, solo: bool) {
        self.delegate.set_surface_solo(trackid, solo)
    }

    fn SetSurfaceRecArm(&self, trackid: *mut MediaTrack, recarm: bool) {
        self.delegate.set_surface_rec_arm(trackid, recarm)
    }

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {
        self.delegate.set_play_state(play, pause, rec)
    }

    fn SetRepeatState(&self, rep: bool) {
        self.delegate.set_repeat_state(rep)
    }

    fn SetTrackTitle(&self, trackid: *mut MediaTrack, title: *const i8) {
        self.delegate
            .set_track_title(trackid, unsafe { CStr::from_ptr(title) })
    }

    fn GetTouchState(&self, trackid: *mut MediaTrack, isPan: i32) -> bool {
        self.delegate.get_touch_state(trackid, isPan)
    }

    fn SetAutoMode(&self, mode: i32) {
        self.delegate.set_auto_mode(mode)
    }

    fn ResetCachedVolPanStates(&self) {
        self.delegate.reset_cached_vol_pan_states()
    }

    fn OnTrackSelection(&self, trackid: *mut MediaTrack) {
        self.delegate.on_track_selection(trackid)
    }

    fn IsKeyDown(&self, key: i32) -> bool {
        self.delegate.is_key_down(key)
    }

    fn Extended(
        &self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        self.delegate.extended(call, parm1, parm2, parm3)
    }
}
