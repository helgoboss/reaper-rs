use super::MediaTrack;
use crate::low_level;
use crate::low_level::raw;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;

// TODO Have a critical look at all signatures of this trait (also return values)
/// This is the medium-level variant of
/// [`low_level::ControlSurface`](../../low_level/trait.ControlSurface.html). An implementation of
/// this trait can be passed to
/// [`medium_level::install_control_surface()`](../fn.install_control_surface.html).
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

    fn set_surface_volume(&self, _trackid: MediaTrack, _volume: f64) {}

    fn set_surface_pan(&self, _trackid: MediaTrack, _pan: f64) {}

    fn set_surface_mute(&self, _trackid: MediaTrack, _mute: bool) {}

    fn set_surface_selected(&self, _trackid: MediaTrack, _selected: bool) {}

    fn set_surface_solo(&self, _trackid: MediaTrack, _solo: bool) {}

    fn set_surface_rec_arm(&self, _trackid: MediaTrack, _recarm: bool) {}

    fn set_play_state(&self, _play: bool, _pause: bool, _rec: bool) {}

    fn set_repeat_state(&self, _rep: bool) {}

    fn set_track_title(&self, _trackid: MediaTrack, _title: &CStr) {}

    fn get_touch_state(&self, _trackid: MediaTrack, _is_pan: i32) -> bool {
        false
    }

    fn set_auto_mode(&self, _mode: i32) {}

    fn reset_cached_vol_pan_states(&self) {}

    fn on_track_selection(&self, _trackid: MediaTrack) {}

    fn is_key_down(&self, _key: i32) -> bool {
        false
    }

    // TODO Should we mark this unsafe to implement?
    fn extended(
        &self,
        _call: i32,
        _parm1: *mut c_void,
        _parm2: *mut c_void,
        _parm3: *mut c_void,
    ) -> i32 {
        0
    }

    fn ext_setinputmonitor(&self, track: MediaTrack, recmonitor: *mut i32) -> i32 {
        0
    }

    fn ext_setfxparam(
        &self,
        track: MediaTrack,
        fxidx_and_paramidx: *mut i32,
        normalized_value: *mut f64,
    ) -> i32 {
        0
    }

    fn ext_setfxparam_recfx(
        &self,
        track: MediaTrack,
        fxidx_and_paramidx: *mut i32,
        normalized_value: *mut f64,
    ) -> i32 {
        0
    }

    fn ext_setfxenabled(&self, track: MediaTrack, fxidx: *mut i32, _enabled: bool) -> i32 {
        0
    }

    fn ext_setsendvolume(&self, track: MediaTrack, sendidx: *mut i32, volume: *mut f64) -> i32 {
        0
    }

    fn ext_setsendpan(&self, track: MediaTrack, sendidx: *mut i32, pan: *mut f64) -> i32 {
        0
    }

    fn ext_setfocusedfx(
        &self,
        track: Option<MediaTrack>,
        mediaitemidx: *mut i32,
        fxidx: *mut i32,
    ) -> i32 {
        0
    }

    fn ext_setfxopen(&self, track: MediaTrack, fxidx: *mut i32, ui_open: bool) -> i32 {
        0
    }

    fn ext_setfxchange(&self, track: MediaTrack, flags: i32) -> i32 {
        0
    }

    fn ext_setlasttouchedfx(
        &self,
        _track: Option<MediaTrack>,
        _mediaitemidx: *mut i32,
        _fxidx: *mut i32,
    ) -> i32 {
        0
    }

    fn ext_setbpmandplayrate(&self, bpm: *mut f64, playrate: *mut f64) -> i32 {
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

    fn SetSurfaceVolume(&self, trackid: *mut raw::MediaTrack, volume: f64) {
        self.delegate
            .set_surface_volume(MediaTrack::required_panic(trackid), volume)
    }

    fn SetSurfacePan(&self, trackid: *mut raw::MediaTrack, pan: f64) {
        self.delegate
            .set_surface_pan(MediaTrack::required_panic(trackid), pan)
    }

    fn SetSurfaceMute(&self, trackid: *mut raw::MediaTrack, mute: bool) {
        self.delegate
            .set_surface_mute(MediaTrack::required_panic(trackid), mute)
    }

    fn SetSurfaceSelected(&self, trackid: *mut raw::MediaTrack, selected: bool) {
        self.delegate
            .set_surface_selected(MediaTrack::required_panic(trackid), selected)
    }

    fn SetSurfaceSolo(&self, trackid: *mut raw::MediaTrack, solo: bool) {
        self.delegate
            .set_surface_solo(MediaTrack::required_panic(trackid), solo)
    }

    fn SetSurfaceRecArm(&self, trackid: *mut raw::MediaTrack, recarm: bool) {
        self.delegate
            .set_surface_rec_arm(MediaTrack::required_panic(trackid), recarm)
    }

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {
        self.delegate.set_play_state(play, pause, rec)
    }

    fn SetRepeatState(&self, rep: bool) {
        self.delegate.set_repeat_state(rep)
    }

    fn SetTrackTitle(&self, trackid: *mut raw::MediaTrack, title: *const i8) {
        self.delegate
            .set_track_title(MediaTrack::required_panic(trackid), unsafe {
                CStr::from_ptr(title)
            })
    }

    fn GetTouchState(&self, trackid: *mut raw::MediaTrack, isPan: i32) -> bool {
        self.delegate
            .get_touch_state(MediaTrack::required_panic(trackid), isPan)
    }

    fn SetAutoMode(&self, mode: i32) {
        self.delegate.set_auto_mode(mode)
    }

    fn ResetCachedVolPanStates(&self) {
        self.delegate.reset_cached_vol_pan_states()
    }

    fn OnTrackSelection(&self, trackid: *mut raw::MediaTrack) {
        self.delegate
            .on_track_selection(MediaTrack::required_panic(trackid))
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
        // TODO Make sure that all known CSURF_EXT_ constants are delegated
        // TODO Some delegate methods still have a bit cryptic parameters
        match call as u32 {
            raw::CSURF_EXT_SETINPUTMONITOR => self.delegate.ext_setinputmonitor(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
            ),
            raw::CSURF_EXT_SETFXPARAM => self.delegate.ext_setfxparam(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut f64,
            ),
            raw::CSURF_EXT_SETFXPARAM_RECFX => self.delegate.ext_setfxparam_recfx(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut f64,
            ),
            raw::CSURF_EXT_SETFXENABLED => self.delegate.ext_setfxenabled(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as usize != 0,
            ),
            raw::CSURF_EXT_SETSENDVOLUME => self.delegate.ext_setsendvolume(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut f64,
            ),
            raw::CSURF_EXT_SETSENDPAN => self.delegate.ext_setsendpan(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut f64,
            ),
            raw::CSURF_EXT_SETFOCUSEDFX => self.delegate.ext_setfocusedfx(
                MediaTrack::optional(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut i32,
            ),
            raw::CSURF_EXT_SETFXOPEN => self.delegate.ext_setfxopen(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as usize != 0,
            ),
            raw::CSURF_EXT_SETFXCHANGE => self.delegate.ext_setfxchange(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as usize as i32,
            ),
            raw::CSURF_EXT_SETLASTTOUCHEDFX => self.delegate.ext_setlasttouchedfx(
                MediaTrack::optional(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                parm3 as *mut i32,
            ),
            raw::CSURF_EXT_SETBPMANDPLAYRATE => self
                .delegate
                .ext_setbpmandplayrate(parm1 as *mut f64, parm2 as *mut f64),
            _ => self.delegate.extended(call, parm1, parm2, parm3),
        }
    }
}
