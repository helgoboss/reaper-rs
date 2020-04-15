use super::MediaTrack;
use crate::low_level;
use crate::low_level::raw;
use crate::medium_level::{
    AutomationMode, InputMonitoringMode, ReaperVersion, TrackFxRef, VersionDependentFxRef,
};
use c_str_macro::c_str;
use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;

// TODO-high Have a critical look at all signatures of this trait (also return values)
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

    // TODO Prevent params from getting underscore
    fn set_surface_volume(&self, _trackid: MediaTrack, _volume: f64) {}

    fn set_surface_pan(&self, _trackid: MediaTrack, _pan: f64) {}

    fn set_surface_mute(&self, _trackid: MediaTrack, _mute: bool) {}

    fn set_surface_selected(&self, _trackid: MediaTrack, _selected: bool) {}

    fn set_surface_solo(&self, _trackid: MediaTrack, _solo: bool) {}

    fn set_surface_rec_arm(&self, _trackid: MediaTrack, _recarm: bool) {}

    fn set_play_state(&self, _play: bool, _pause: bool, _rec: bool) {}

    fn set_repeat_state(&self, _rep: bool) {}

    fn set_track_title(&self, _trackid: MediaTrack, _title: &CStr) {}

    // TODO is_pan param
    fn get_touch_state(&self, _trackid: MediaTrack, _is_pan: i32) -> bool {
        false
    }

    fn set_auto_mode(&self, _mode: AutomationMode) {}

    fn reset_cached_vol_pan_states(&self) {}

    fn on_track_selection(&self, _trackid: MediaTrack) {}

    // TODO Maybe enum keys
    fn is_key_down(&self, _key: i32) -> bool {
        false
    }

    unsafe fn extended(
        &self,
        _call: i32,
        _parm1: *mut c_void,
        _parm2: *mut c_void,
        _parm3: *mut c_void,
    ) -> i32 {
        0
    }

    fn ext_setinputmonitor(&self, track: MediaTrack, recmonitor: InputMonitoringMode) -> i32 {
        0
    }

    fn ext_setfxparam(
        &self,
        track: MediaTrack,
        // TODO Check if this is called also for input FX in REAPER < 5.95 - or not at all
        fx_index: u32,
        param_index: u32,
        normalized_value: f64,
    ) -> i32 {
        0
    }

    fn ext_setfxparam_recfx(
        &self,
        track: MediaTrack,
        fx_index: u32,
        param_index: u32,
        normalized_value: f64,
    ) -> i32 {
        0
    }

    fn ext_setfxenabled(
        &self,
        track: MediaTrack,
        fxidx: VersionDependentFxRef,
        _enabled: bool,
    ) -> i32 {
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
        fxidx: VersionDependentFxRef,
    ) -> i32 {
        0
    }

    fn ext_setfxopen(&self, track: MediaTrack, fxidx: VersionDependentFxRef, ui_open: bool) -> i32 {
        0
    }

    fn ext_setfxchange(&self, track: MediaTrack, flags: i32) -> i32 {
        0
    }

    fn ext_setlasttouchedfx(
        &self,
        _track: Option<MediaTrack>,
        _mediaitemidx: *mut i32,
        _fxidx: VersionDependentFxRef,
    ) -> i32 {
        0
    }

    fn ext_setbpmandplayrate(&self, bpm: *mut f64, playrate: *mut f64) -> i32 {
        0
    }
}

pub struct DelegatingControlSurface<T: ControlSurface> {
    delegate: T,
    // Capabilities depending on REAPER version
    supports_detection_of_input_fx: bool,
    supports_detection_of_input_fx_in_set_fx_change: bool,
}

impl<T: ControlSurface> DelegatingControlSurface<T> {
    pub fn new(delegate: T, reaper_version: &ReaperVersion) -> DelegatingControlSurface<T> {
        let reaper_version_5_95: ReaperVersion = c_str!("5.95").into();
        DelegatingControlSurface {
            delegate,
            // since pre1,
            supports_detection_of_input_fx: reaper_version >= &reaper_version_5_95,
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: reaper_version >= &reaper_version_5_95,
        }
    }

    fn get_as_version_dependent_fx_ref(&self, ptr: *mut c_void) -> VersionDependentFxRef {
        let ptr = ptr as *mut i32;
        let index = unsafe { *ptr } as u32;
        if self.supports_detection_of_input_fx {
            VersionDependentFxRef::New(index.into())
        } else {
            VersionDependentFxRef::Old(index)
        }
    }
}

#[allow(non_snake_case)]
impl<T: ControlSurface> low_level::IReaperControlSurface for DelegatingControlSurface<T> {
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
        self.delegate
            .set_auto_mode(mode.try_into().expect("Unknown automation mode"))
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
        // TODO-high Make sure that all known CSURF_EXT_ constants are delegated
        match call as u32 {
            raw::CSURF_EXT_SETINPUTMONITOR => {
                let parm2 = parm2 as *mut i32;
                let recmon = unsafe { *parm2 };
                self.delegate.ext_setinputmonitor(
                    MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                    recmon.try_into().expect("Unknown input monitoring mode"),
                )
            }
            raw::CSURF_EXT_SETFXPARAM | raw::CSURF_EXT_SETFXPARAM_RECFX => {
                let parm2 = parm2 as *mut i32;
                let parm3 = parm2 as *mut f64;
                let fxidx_and_paramidx = unsafe { *parm2 };
                let normalized_value = unsafe { *parm3 };
                let fx_index = (fxidx_and_paramidx >> 16) & 0xffff;
                let param_index = fxidx_and_paramidx & 0xffff;
                match call as u32 {
                    raw::CSURF_EXT_SETFXPARAM => self.delegate.ext_setfxparam(
                        MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                        fx_index as u32,
                        param_index as u32,
                        normalized_value,
                    ),
                    raw::CSURF_EXT_SETFXPARAM_RECFX => self.delegate.ext_setfxparam_recfx(
                        MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                        fx_index as u32,
                        param_index as u32,
                        normalized_value,
                    ),
                    _ => unreachable!(),
                }
            }
            raw::CSURF_EXT_SETFOCUSEDFX => self.delegate.ext_setfocusedfx(
                MediaTrack::optional(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                self.get_as_version_dependent_fx_ref(parm3),
            ),
            raw::CSURF_EXT_SETFXOPEN => self.delegate.ext_setfxopen(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                self.get_as_version_dependent_fx_ref(parm2),
                parm3 as usize != 0,
            ),
            raw::CSURF_EXT_SETFXENABLED => self.delegate.ext_setfxenabled(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                self.get_as_version_dependent_fx_ref(parm2),
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
            raw::CSURF_EXT_SETFXCHANGE => self.delegate.ext_setfxchange(
                MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                parm2 as usize as i32,
            ),
            raw::CSURF_EXT_SETLASTTOUCHEDFX => self.delegate.ext_setlasttouchedfx(
                MediaTrack::optional(parm1 as *mut raw::MediaTrack),
                parm2 as *mut i32,
                self.get_as_version_dependent_fx_ref(parm3),
            ),
            raw::CSURF_EXT_SETBPMANDPLAYRATE => self
                .delegate
                .ext_setbpmandplayrate(parm1 as *mut f64, parm2 as *mut f64),
            _ => unsafe { self.delegate.extended(call, parm1, parm2, parm3) },
        }
    }
}
