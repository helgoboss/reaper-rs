use super::MediaTrack;
use crate::{
    require_non_null_panic, AutomationMode, InputMonitoringMode, ReaperControlSurface,
    ReaperNormalizedValue, ReaperPanValue, ReaperVersion, ReaperVolumeValue, TrackFxChainType,
    TrackFxRef,
};
use c_str_macro::c_str;
use enumflags2::_internal::core::convert::TryFrom;
use reaper_rs_low;
use reaper_rs_low::{raw, IReaperControlSurface};
use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::panic::RefUnwindSafe;
use std::ptr::{null_mut, NonNull};

/// Consumers need to implement this trait in order to be notified about various REAPER events.
///
/// See [`plugin_register_add_csurf_inst`].
///
/// [`plugin_register_add_csurf_inst`]: struct.Reaper.html#method.plugin_register_add_csurf_inst
pub trait MediumReaperControlSurface: RefUnwindSafe {
    /// simple unique string with only A-Z, 0-9, no spaces or other chars
    fn get_type_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    /// human readable description (can include instance specific info)
    fn get_desc_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    /// string of configuration data
    fn get_config_string(&self) -> Option<Cow<'static, CStr>> {
        None
    }

    /// close without sending "reset" messages, prevent "reset" being sent on destructor
    fn close_no_reset(&self) {}

    /// called 30x/sec or so.
    fn run(&mut self) {}

    fn set_track_list_change(&self) {}

    fn set_surface_volume(&self, args: SetSurfaceVolumeArgs) {}

    fn set_surface_pan(&self, args: SetSurfacePanArgs) {}

    fn set_surface_mute(&self, args: SetSurfaceMuteArgs) {}

    fn set_surface_selected(&self, args: SetSurfaceSelectedArgs) {}

    /// trackid==master means "any solo"
    fn set_surface_solo(&self, args: SetSurfaceSoloArgs) {}

    fn set_surface_rec_arm(&self, args: SetSurfaceRecArmArgs) {}

    fn set_play_state(&self, args: SetPlayStateArgs) {}

    fn set_repeat_state(&self, args: SetRepeatStateArgs) {}

    fn set_track_title(&self, args: SetTrackTitleArgs) {}

    fn get_touch_state(&self, args: GetTouchStateArgs) -> bool {
        false
    }

    /// automation mode for current track
    fn set_auto_mode(&self, args: SetAutoModeArgs) {}

    /// good to flush your control states here
    fn reset_cached_vol_pan_states(&self) {}

    /// track was selected
    fn on_track_selection(&self, args: OnTrackSelectionArgs) {}

    /// Control, Menu, Shift, etc, whatever makes sense for your surface
    fn is_key_down(&self, args: IsKeyDownArgs) -> bool {
        false
    }

    /// return 0 if unsupported
    unsafe fn extended(
        &self,
        _call: i32,
        _parm1: *mut c_void,
        _parm2: *mut c_void,
        _parm3: *mut c_void,
    ) -> i32 {
        0
    }

    fn ext_set_input_monitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        0
    }

    /// For REAPER < 5.95 this is called for FX in the input FX chain as well. In this case we just
    /// don't know if the given FX index refers to the normal or input FX chain.
    fn ext_set_fx_param(&self, args: ExtSetFxParamArgs) -> i32 {
        0
    }

    fn ext_set_fx_param_rec_fx(&self, args: ExtSetFxParamArgs) -> i32 {
        0
    }

    fn ext_set_fx_enabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        0
    }

    fn ext_set_send_volume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        0
    }

    fn ext_set_send_pan(&self, args: ExtSetSendPanArgs) -> i32 {
        0
    }

    fn ext_set_focused_fx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        0
    }

    fn ext_set_last_touched_fx(&self, args: ExtSetLastTouchedFxArgs) -> i32 {
        0
    }

    fn ext_set_fx_open(&self, args: ExtSetFxOpenArgs) -> i32 {
        0
    }

    fn ext_set_fx_change(&self, args: ExtSetFxChangeArgs) -> i32 {
        0
    }

    fn ext_set_bpm_and_play_rate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SetSurfaceVolumeArgs {
    pub track: MediaTrack,
    pub volume: ReaperVolumeValue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SetSurfacePanArgs {
    pub track: MediaTrack,
    pub pan: ReaperPanValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetSurfaceMuteArgs {
    pub track: MediaTrack,
    pub mute: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetSurfaceSelectedArgs {
    pub track: MediaTrack,
    pub selected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetSurfaceSoloArgs {
    pub track: MediaTrack,
    pub solo: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetSurfaceRecArmArgs {
    pub track: MediaTrack,
    pub recarm: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetPlayStateArgs {
    pub play: bool,
    pub pause: bool,
    pub rec: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetRepeatStateArgs {
    pub rep: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetTrackTitleArgs<'a> {
    pub trackid: MediaTrack,
    pub title: &'a CStr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetTouchStateArgs {
    pub trackid: MediaTrack,
    pub is_pan: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetAutoModeArgs {
    pub mode: AutomationMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OnTrackSelectionArgs {
    pub trackid: MediaTrack,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ModKey {
    Shift,
    Control,
    Menu,
    Custom(u32),
}

impl TryFrom<i32> for ModKey {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(());
        };
        let value = value as u32;
        use ModKey::*;
        let key = match value {
            raw::VK_SHIFT => Shift,
            raw::VK_CONTROL => Control,
            raw::VK_MENU => Menu,
            _ => Custom(value),
        };
        Ok(key)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IsKeyDownArgs {
    pub key: ModKey,
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
    pub normalized_value: ReaperNormalizedValue,
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
    pub fx_chain_type: Option<TrackFxChainType>,
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
pub enum VersionDependentTrackFxRef {
    /// In old REAPER versions (< 5.95) the index can represent either input or output FX - we
    /// don't know.
    Old(u32),
    /// In newer REAPER versions, it's possible to distinguish between input and output FX
    New(TrackFxRef),
}

pub struct DelegatingControlSurface {
    delegate: Box<dyn MediumReaperControlSurface>,
    // Capabilities depending on REAPER version
    supports_detection_of_input_fx: bool,
    supports_detection_of_input_fx_in_set_fx_change: bool,
}

impl DelegatingControlSurface {
    pub fn new(
        delegate: impl MediumReaperControlSurface + 'static,
        reaper_version: &ReaperVersion,
    ) -> DelegatingControlSurface {
        let reaper_version_5_95: ReaperVersion = ReaperVersion::from("5.95");
        DelegatingControlSurface {
            delegate: Box::new(delegate),
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
            track: require_non_null_panic(media_track_ptr as *mut raw::MediaTrack),
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
impl reaper_rs_low::IReaperControlSurface for DelegatingControlSurface {
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
        self.delegate.set_surface_volume(SetSurfaceVolumeArgs {
            track: require_non_null_panic(trackid),
            volume,
        })
    }

    fn SetSurfacePan(&self, trackid: *mut raw::MediaTrack, pan: f64) {
        self.delegate.set_surface_pan(SetSurfacePanArgs {
            track: require_non_null_panic(trackid),
            pan,
        })
    }

    fn SetSurfaceMute(&self, trackid: *mut raw::MediaTrack, mute: bool) {
        self.delegate.set_surface_mute(SetSurfaceMuteArgs {
            track: require_non_null_panic(trackid),
            mute,
        })
    }

    fn SetSurfaceSelected(&self, trackid: *mut raw::MediaTrack, selected: bool) {
        self.delegate.set_surface_selected(SetSurfaceSelectedArgs {
            track: require_non_null_panic(trackid),
            selected,
        })
    }

    fn SetSurfaceSolo(&self, trackid: *mut raw::MediaTrack, solo: bool) {
        self.delegate.set_surface_solo(SetSurfaceSoloArgs {
            track: require_non_null_panic(trackid),
            solo,
        })
    }

    fn SetSurfaceRecArm(&self, trackid: *mut raw::MediaTrack, recarm: bool) {
        self.delegate.set_surface_rec_arm(SetSurfaceRecArmArgs {
            track: require_non_null_panic(trackid),
            recarm,
        })
    }

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {
        self.delegate
            .set_play_state(SetPlayStateArgs { play, pause, rec })
    }

    fn SetRepeatState(&self, rep: bool) {
        self.delegate.set_repeat_state(SetRepeatStateArgs { rep })
    }

    fn SetTrackTitle(&self, trackid: *mut raw::MediaTrack, title: *const i8) {
        self.delegate.set_track_title(SetTrackTitleArgs {
            trackid: require_non_null_panic(trackid),
            title: unsafe { CStr::from_ptr(title) },
        })
    }

    fn GetTouchState(&self, trackid: *mut raw::MediaTrack, isPan: i32) -> bool {
        self.delegate.get_touch_state(GetTouchStateArgs {
            trackid: require_non_null_panic(trackid),
            is_pan: isPan != 0,
        })
    }

    fn SetAutoMode(&self, mode: i32) {
        self.delegate.set_auto_mode(SetAutoModeArgs {
            mode: mode.try_into().expect("Unknown automation mode"),
        })
    }

    fn ResetCachedVolPanStates(&self) {
        self.delegate.reset_cached_vol_pan_states()
    }

    fn OnTrackSelection(&self, trackid: *mut raw::MediaTrack) {
        self.delegate.on_track_selection(OnTrackSelectionArgs {
            trackid: require_non_null_panic(trackid),
        })
    }

    fn IsKeyDown(&self, key: i32) -> bool {
        self.delegate.is_key_down(IsKeyDownArgs {
            key: key.try_into().expect("Got negative key code"),
        })
    }

    fn Extended(
        &self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        unsafe {
            // TODO-low Delegate all known CSURF_EXT_ constants
            match call as u32 {
                raw::CSURF_EXT_SETINPUTMONITOR => {
                    let recmon: i32 = unref_into(parm2).unwrap();
                    self.delegate.ext_set_input_monitor(ExtSetInputMonitorArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        recmonitor: recmon.try_into().expect("Unknown input monitoring mode"),
                    })
                }
                raw::CSURF_EXT_SETFXPARAM | raw::CSURF_EXT_SETFXPARAM_RECFX => {
                    let fxidx_and_paramidx: i32 = unref_into(parm2).unwrap();
                    let normalized_value: f64 = unref_into(parm3).unwrap();
                    let fx_index = (fxidx_and_paramidx >> 16) & 0xffff;
                    let param_index = fxidx_and_paramidx & 0xffff;
                    let args = ExtSetFxParamArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        fx_index: fx_index as u32,
                        param_index: param_index as u32,
                        normalized_value: ReaperNormalizedValue::new(normalized_value),
                    };
                    match call as u32 {
                        raw::CSURF_EXT_SETFXPARAM => self.delegate.ext_set_fx_param(args),
                        raw::CSURF_EXT_SETFXPARAM_RECFX => {
                            self.delegate.ext_set_fx_param_rec_fx(args)
                        }
                        _ => unreachable!(),
                    }
                }
                raw::CSURF_EXT_SETFOCUSEDFX => {
                    self.delegate.ext_set_focused_fx(ExtSetFocusedFxArgs {
                        fx_ref: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                    })
                }
                raw::CSURF_EXT_SETLASTTOUCHEDFX => {
                    self.delegate
                        .ext_set_last_touched_fx(ExtSetLastTouchedFxArgs {
                            fx_ref: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                        })
                }
                raw::CSURF_EXT_SETFXOPEN => self.delegate.ext_set_fx_open(ExtSetFxOpenArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    fxidx: self.get_as_version_dependent_track_fx_ref(parm2),
                    ui_open: interpret_as_bool(parm3),
                }),
                raw::CSURF_EXT_SETFXENABLED => {
                    if parm1.is_null() {
                        // Don't know how to handle that case. Maybe a bug in REAPER.
                        self.delegate.extended(call, parm1, parm2, parm3)
                    } else {
                        self.delegate.ext_set_fx_enabled(ExtSetFxEnabledArgs {
                            track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                            fxidx: self.get_as_version_dependent_track_fx_ref(parm2),
                            enabled: interpret_as_bool(parm3),
                        })
                    }
                }
                raw::CSURF_EXT_SETSENDVOLUME => {
                    self.delegate.ext_set_send_volume(ExtSetSendVolumeArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        sendidx: unref_into::<i32>(parm2).unwrap() as u32,
                        volume: unref_into(parm3).unwrap(),
                    })
                }
                raw::CSURF_EXT_SETSENDPAN => self.delegate.ext_set_send_pan(ExtSetSendPanArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    sendidx: unref_into::<i32>(parm2).unwrap() as u32,
                    pan: unref_into(parm3).unwrap(),
                }),
                raw::CSURF_EXT_SETFXCHANGE => self.delegate.ext_set_fx_change(ExtSetFxChangeArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    fx_chain_type: {
                        if self.supports_detection_of_input_fx_in_set_fx_change {
                            let flags = parm2 as usize as u32;
                            let fx_chain_type = if (flags & 1) == 1 {
                                TrackFxChainType::InputFxChain
                            } else {
                                TrackFxChainType::NormalFxChain
                            };
                            Some(fx_chain_type)
                        } else {
                            None
                        }
                    },
                }),
                raw::CSURF_EXT_SETBPMANDPLAYRATE => {
                    self.delegate
                        .ext_set_bpm_and_play_rate(ExtSetBpmAndPlayRateArgs {
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
