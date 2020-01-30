use crate::low_level::MediaTrack;
use crate::low_level;
use std::ffi::CStr;
use std::borrow::Cow;
use std::os::raw::c_void;
use std::ptr::{null_mut, null};

pub trait ControlSurface {
    fn get_type_string(&mut self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn get_desc_string(&mut self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn get_config_string(&mut self) -> Option<Cow<'static, CStr>> {
        None
    }

    fn close_no_reset(&mut self) {}

    fn run(&mut self) {}

    fn set_track_list_change(&mut self) {}

    fn set_surface_volume(&mut self, trackid: *mut MediaTrack, volume: f64) {}

    fn set_surface_pan(&mut self, trackid: *mut MediaTrack, pan: f64) {}

    fn set_surface_mute(&mut self, trackid: *mut MediaTrack, mute: bool) {}

    fn set_surface_selected(&mut self, trackid: *mut MediaTrack, selected: bool) {}

    fn set_surface_solo(&mut self, trackid: *mut MediaTrack, solo: bool) {}

    fn set_surface_rec_arm(&mut self, trackid: *mut MediaTrack, recarm: bool) {}

    fn set_play_state(&mut self, play: bool, pause: bool, rec: bool) {}

    fn set_repeat_state(&mut self, rep: bool) {}

    fn set_track_title(&mut self, trackid: *mut MediaTrack, title: &CStr) {}

    fn get_touch_state(&mut self, trackid: *mut MediaTrack, is_pan: i32) -> bool {
        false
    }

    fn set_auto_mode(&mut self, mode: i32) {}

    fn reset_cached_vol_pan_states(&mut self) {}

    fn on_track_selection(&mut self, trackid: *mut MediaTrack) {}

    fn is_key_down(&mut self, key: i32) -> bool {
        false
    }

    fn extended(
        &mut self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        0
    }
}

pub struct DelegatingControlSurface<T: ControlSurface> {
    delegate: T
}

impl<T: ControlSurface> DelegatingControlSurface<T> {
    pub fn new(delegate: T) -> DelegatingControlSurface<T> {
        DelegatingControlSurface {
            delegate
        }
    }
}

impl<T: ControlSurface> low_level::ControlSurface for DelegatingControlSurface<T> {
    fn GetTypeString(&mut self) -> *const i8 {
        self.delegate.get_type_string().map(|o| o.as_ptr()).unwrap_or(null_mut())
    }

    fn GetDescString(&mut self) -> *const i8 {
        self.delegate.get_desc_string().map(|o| o.as_ptr()).unwrap_or(null_mut())
    }

    fn GetConfigString(&mut self) -> *const i8 {
        self.delegate.get_config_string().map(|o| o.as_ptr()).unwrap_or(null_mut())
    }

    fn CloseNoReset(&mut self) {
        self.delegate.close_no_reset()
    }

    fn Run(&mut self) {
        self.delegate.run()
    }

    fn SetTrackListChange(&mut self) {
        self.delegate.set_track_list_change()
    }

    fn SetSurfaceVolume(&mut self, trackid: *mut MediaTrack, volume: f64) {
        self.delegate.set_surface_volume(trackid, volume)
    }

    fn SetSurfacePan(&mut self, trackid: *mut MediaTrack, pan: f64) {
        self.delegate.set_surface_pan(trackid, pan)
    }

    fn SetSurfaceMute(&mut self, trackid: *mut MediaTrack, mute: bool) {
        self.delegate.set_surface_mute(trackid, mute)
    }

    fn SetSurfaceSelected(&mut self, trackid: *mut MediaTrack, selected: bool) {
        self.delegate.set_surface_selected(trackid, selected)
    }

    fn SetSurfaceSolo(&mut self, trackid: *mut MediaTrack, solo: bool) {
        self.delegate.set_surface_solo(trackid, solo)
    }

    fn SetSurfaceRecArm(&mut self, trackid: *mut MediaTrack, recarm: bool) {
        self.delegate.set_surface_rec_arm(trackid, recarm)
    }

    fn SetPlayState(&mut self, play: bool, pause: bool, rec: bool) {
        self.delegate.set_play_state(play, pause, rec)
    }

    fn SetRepeatState(&mut self, rep: bool) {
        self.delegate.set_repeat_state(rep)
    }

    fn SetTrackTitle(&mut self, trackid: *mut MediaTrack, title: *const i8) {
        self.delegate.set_track_title(trackid, unsafe { CStr::from_ptr(title) })
    }

    fn GetTouchState(&mut self, trackid: *mut MediaTrack, isPan: i32) -> bool {
        self.delegate.get_touch_state(trackid, isPan)
    }

    fn SetAutoMode(&mut self, mode: i32) {
        self.delegate.set_auto_mode(mode)
    }

    fn ResetCachedVolPanStates(&mut self) {
        self.delegate.reset_cached_vol_pan_states()
    }

    fn OnTrackSelection(&mut self, trackid: *mut MediaTrack) {
        self.delegate.on_track_selection(trackid)
    }

    fn IsKeyDown(&mut self, key: i32) -> bool {
        self.delegate.is_key_down(key)
    }

    fn Extended(&mut self, call: i32, parm1: *mut c_void, parm2: *mut c_void, parm3: *mut c_void) -> i32 {
        self.delegate.extended(call, parm1, parm2, parm3)
    }
}