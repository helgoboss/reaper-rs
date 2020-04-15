use super::MediaTrack;
use crate::high_level::FxChain;
use crate::low_level;
use crate::low_level::raw;
use crate::medium_level::{AutomationMode, InputMonitoringMode, ReaperVersion, TrackFxRef};
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

    // TODO is_pan param, maybe introduce struct to be immune against future meaning extensions of
    //  the i32
    fn get_touch_state(&self, _trackid: MediaTrack, _is_pan: i32) -> bool {
        false
    }

    // TODO Automation for what? The global one? Can it be None? Bypass?
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

    fn ext_setinputmonitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        0
    }

    // TODO Check if this is called also for input FX in REAPER < 5.95 - or not at all
    fn ext_setfxparam(&self, args: ExtSetFxParamArgs) -> i32 {
        0
    }

    fn ext_setfxparam_recfx(&self, args: ExtSetFxParamArgs) -> i32 {
        0
    }

    fn ext_setfxenabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        0
    }

    fn ext_setsendvolume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        0
    }

    fn ext_setsendpan(&self, args: ExtSetSendPanArgs) -> i32 {
        0
    }

    fn ext_setfocusedfx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        0
    }

    fn ext_setlasttouchedfx(&self, args: ExtSetLastTouchedFxArgs) -> i32 {
        0
    }

    fn ext_setfxopen(&self, args: ExtSetFxOpenArgs) -> i32 {
        0
    }

    fn ext_setfxchange(&self, args: ExtSetFxChangeArgs) -> i32 {
        0
    }

    fn ext_setbpmandplayrate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetInputMonitorArgs {
    pub track: MediaTrack,
    pub recmonitor: InputMonitoringMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtSetFxParamArgs {
    pub track: MediaTrack,
    pub fx_index: u32,
    pub param_index: u32,
    pub normalized_value: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetFxEnabledArgs {
    pub track: MediaTrack,
    pub fxidx: VersionDependentTrackFxRef,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtSetSendVolumeArgs {
    pub track: MediaTrack,
    pub sendidx: u32,
    pub volume: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtSetSendPanArgs {
    pub track: MediaTrack,
    pub sendidx: u32,
    pub pan: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetFocusedFxArgs {
    pub fx_ref: Option<QualifiedFxRef>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetLastTouchedFxArgs {
    pub fx_ref: Option<QualifiedFxRef>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetFxOpenArgs {
    pub track: MediaTrack,
    pub fxidx: VersionDependentTrackFxRef,
    pub ui_open: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtSetFxChangeArgs {
    pub track: MediaTrack,
    // In REAPER < 5.95 we don't know if the change happened on input or normal FX chain
    pub fx_chain_type: Option<FxChainType>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtSetBpmAndPlayRateArgs {
    pub bpm: Option<f64>,
    pub playrate: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QualifiedFxRef {
    pub track: MediaTrack,
    pub fx_ref: VersionDependentFxRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionDependentFxRef {
    ItemFx { item_index: u32, fx_index: u32 },
    TrackFx(VersionDependentTrackFxRef),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FxChainType {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionDependentTrackFxRef {
    /// In old REAPER versions (< 5.95) the index can represent either input or output FX - we
    /// don't know.
    Old(u32),
    /// In newer REAPER versions, it's possible to distinguish between input and output FX
    New(TrackFxRef),
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

    unsafe fn get_as_qualified_fx_ref(
        &self,
        media_track_ptr: *mut c_void,
        media_item_ptr: *mut c_void,
        fx_index_ptr: *mut c_void,
    ) -> Option<QualifiedFxRef> {
        if media_track_ptr.is_null() {
            return None;
        }
        Some(QualifiedFxRef {
            track: MediaTrack::required_panic(media_track_ptr as *mut raw::MediaTrack),
            fx_ref: if media_item_ptr.is_null() {
                VersionDependentFxRef::TrackFx(
                    self.get_as_version_dependent_track_fx_ref(fx_index_ptr),
                )
            } else {
                VersionDependentFxRef::ItemFx {
                    item_index: unref_into::<i32>(media_item_ptr).unwrap() as u32,
                    fx_index: unref_into::<i32>(fx_index_ptr).unwrap() as u32,
                }
            },
        })
    }

    unsafe fn get_as_version_dependent_track_fx_ref(
        &self,
        ptr: *mut c_void,
    ) -> VersionDependentTrackFxRef {
        let index = unref_into::<i32>(ptr).unwrap() as u32;
        if self.supports_detection_of_input_fx {
            VersionDependentTrackFxRef::New(index.into())
        } else {
            VersionDependentTrackFxRef::Old(index)
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
        unsafe {
            // TODO-high Make sure that all known CSURF_EXT_ constants are delegated
            match call as u32 {
                raw::CSURF_EXT_SETINPUTMONITOR => {
                    let recmon: i32 = unref_into(parm2).unwrap();
                    self.delegate.ext_setinputmonitor(ExtSetInputMonitorArgs {
                        track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                        recmonitor: recmon.try_into().expect("Unknown input monitoring mode"),
                    })
                }
                raw::CSURF_EXT_SETFXPARAM | raw::CSURF_EXT_SETFXPARAM_RECFX => {
                    let fxidx_and_paramidx: i32 = unref_into(parm2).unwrap();
                    let normalized_value: f64 = unref_into(parm3).unwrap();
                    let fx_index = (fxidx_and_paramidx >> 16) & 0xffff;
                    let param_index = fxidx_and_paramidx & 0xffff;
                    let args = ExtSetFxParamArgs {
                        track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                        fx_index: fx_index as u32,
                        param_index: param_index as u32,
                        normalized_value,
                    };
                    match call as u32 {
                        raw::CSURF_EXT_SETFXPARAM => self.delegate.ext_setfxparam(args),
                        raw::CSURF_EXT_SETFXPARAM_RECFX => self.delegate.ext_setfxparam_recfx(args),
                        _ => unreachable!(),
                    }
                }
                raw::CSURF_EXT_SETFOCUSEDFX => {
                    self.delegate.ext_setfocusedfx(ExtSetFocusedFxArgs {
                        fx_ref: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                    })
                }
                raw::CSURF_EXT_SETLASTTOUCHEDFX => {
                    self.delegate.ext_setlasttouchedfx(ExtSetLastTouchedFxArgs {
                        fx_ref: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                    })
                }
                raw::CSURF_EXT_SETFXOPEN => self.delegate.ext_setfxopen(ExtSetFxOpenArgs {
                    track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                    fxidx: self.get_as_version_dependent_track_fx_ref(parm2),
                    ui_open: interpret_as_bool(parm3),
                }),
                raw::CSURF_EXT_SETFXENABLED => {
                    if parm1.is_null() {
                        // Don't know how to handle that case. Maybe a bug in REAPER.
                        self.delegate.extended(call, parm1, parm2, parm3)
                    } else {
                        self.delegate.ext_setfxenabled(ExtSetFxEnabledArgs {
                            track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                            fxidx: self.get_as_version_dependent_track_fx_ref(parm2),
                            enabled: interpret_as_bool(parm3),
                        })
                    }
                }
                raw::CSURF_EXT_SETSENDVOLUME => {
                    self.delegate.ext_setsendvolume(ExtSetSendVolumeArgs {
                        track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                        sendidx: unref_into::<i32>(parm2).unwrap() as u32,
                        volume: unref_into(parm3).unwrap(),
                    })
                }
                raw::CSURF_EXT_SETSENDPAN => self.delegate.ext_setsendpan(ExtSetSendPanArgs {
                    track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                    sendidx: unref_into::<i32>(parm2).unwrap() as u32,
                    pan: unref_into(parm3).unwrap(),
                }),
                raw::CSURF_EXT_SETFXCHANGE => self.delegate.ext_setfxchange(ExtSetFxChangeArgs {
                    track: MediaTrack::required_panic(parm1 as *mut raw::MediaTrack),
                    fx_chain_type: {
                        if self.supports_detection_of_input_fx_in_set_fx_change {
                            let flags = parm2 as usize as u32;
                            let fx_chain_type = if (flags & 1) == 1 {
                                FxChainType::Input
                            } else {
                                FxChainType::Output
                            };
                            Some(fx_chain_type)
                        } else {
                            None
                        }
                    },
                }),
                raw::CSURF_EXT_SETBPMANDPLAYRATE => {
                    self.delegate
                        .ext_setbpmandplayrate(ExtSetBpmAndPlayRateArgs {
                            bpm: unref_into(parm1),
                            playrate: unref_into(parm2),
                        })
                }
                _ => self.delegate.extended(call, parm1, parm2, parm3),
            }
        }
    }
}

unsafe fn unref_into<T: Copy>(ptr: *mut c_void) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    let ptr = ptr as *mut T;
    Some(*ptr)
}

unsafe fn interpret_as_bool(ptr: *mut c_void) -> bool {
    !ptr.is_null()
}
