#![allow(non_snake_case)]
use super::MediaTrack;
use crate::{
    require_non_null_panic, AutomationMode, Bpm, Hidden, InputMonitoringMode, Pan, PanMode,
    PlaybackSpeedFactor, ReaperNormalizedFxParamValue, ReaperPanValue, ReaperStr, ReaperVersion,
    ReaperVolumeValue, TrackFxChainType, TrackFxLocation,
};

use reaper_low::raw;

use std::fmt::Debug;
use std::os::raw::c_void;
use std::ptr::null_mut;

/// Consumers need to implement this trait in order to get notified about various REAPER events.
///
/// All callbacks are invoked in the main thread.
///
/// See [`plugin_register_add_csurf_inst`].
///
/// [`plugin_register_add_csurf_inst`]:
/// struct.ReaperSession.html#method.plugin_register_add_csurf_inst
pub trait ControlSurface: Debug {
    /// Should return the control surface type.
    ///
    /// Must be a simple unique string with only A-Z, 0-9, no spaces or other characters.
    ///
    /// Return `None` if this is a control surface behind the scenes.
    //
    // We can't let this returns something owned because it would be gone as soon as the delegate
    // control surface turns this into a pointer and returns. This method of `IReaperControlSurface`
    // and related ones are just designed to work that way.
    fn get_type_string(&self) -> Option<&ReaperStr> {
        None
    }

    /// Should return the control surface description.
    ///
    /// Should be a human readable description, can include instance-specific information.
    ///
    /// Return `None` if this is a control surface behind the scenes.
    fn get_desc_string(&self) -> Option<&ReaperStr> {
        None
    }

    /// Should return a string of configuration data.
    ///
    /// Return `None` if this is a control surface behind the scenes.
    fn get_config_string(&self) -> Option<&ReaperStr> {
        None
    }

    /// Should close the control surface without sending *reset* messages.
    ///
    /// Prevent *reset* being sent in the destructor.
    fn close_no_reset(&self) {}

    /// Called on each main loop cycle.
    ///
    /// Called about 30 times per second.
    fn run(&mut self) {}

    /// Called when the track list has changed.
    ///
    /// This is called for each track once.
    fn set_track_list_change(&self) {}

    /// Called when the volume of a track has changed.
    fn set_surface_volume(&self, args: SetSurfaceVolumeArgs) {
        let _ = args;
    }

    /// Called when the pan of a track has changed.
    fn set_surface_pan(&self, args: SetSurfacePanArgs) {
        let _ = args;
    }

    /// Called when a track has been muted or unmuted.
    fn set_surface_mute(&self, args: SetSurfaceMuteArgs) {
        let _ = args;
    }

    /// Called when a track has been selected or unselected.
    fn set_surface_selected(&self, args: SetSurfaceSelectedArgs) {
        let _ = args;
    }

    /// Called when a track has been soloed or unsoloed.
    ///
    /// If it's the master track, it means "any solo".
    fn set_surface_solo(&self, args: SetSurfaceSoloArgs) {
        let _ = args;
    }

    /// Called when a track has been armed or unarmed for recording.
    fn set_surface_rec_arm(&self, args: SetSurfaceRecArmArgs) {
        let _ = args;
    }

    /// Called when the transport state has changed (playing, paused, recording).
    fn set_play_state(&self, args: SetPlayStateArgs) {
        let _ = args;
    }

    /// Called when repeat has been enabled or disabled.
    fn set_repeat_state(&self, args: SetRepeatStateArgs) {
        let _ = args;
    }

    /// Called when a track name has changed.
    fn set_track_title(&self, args: SetTrackTitleArgs) {
        let _ = args;
    }

    /// This is regularly queried by REAPER for touch automation mode in order to determine whether
    /// the parameter on the given track should still write automation or not.
    ///
    /// The main use case are touch-sensitive motor faders.
    fn get_touch_state(&self, args: GetTouchStateArgs) -> bool {
        let _ = args;
        false
    }

    /// Called when the automation mode of the current track has changed.
    fn set_auto_mode(&self, args: SetAutoModeArgs) {
        let _ = args;
    }

    /// Should flush the control states.
    fn reset_cached_vol_pan_states(&self) {}

    /// Called when a track has been selected.
    fn on_track_selection(&self, args: OnTrackSelectionArgs) {
        let _ = args;
    }

    /// Should return whether the given modifier key is currently pressed on the surface.
    fn is_key_down(&self, args: IsKeyDownArgs) -> bool {
        let _ = args;
        false
    }

    /// Generic method which is called for many kinds of events. Prefer implementing the type-safe
    /// `ext_` methods instead!
    ///
    /// *reaper-rs* calls this method only if you didn't process the event already in one of the
    /// `ext_` methods. The meaning of the return value depends on the particular event type
    /// ([`args.call`]). In any case, returning 0 means that the event has not been handled.
    ///
    /// # Safety
    ///
    /// Implementing this is unsafe because you need to deal with raw pointers.
    ///
    /// [`args.call`]: struct.ExtendedArgs.html#structfield.call
    unsafe fn extended(&self, args: ExtendedArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the input monitoring mode of a track has has changed.
    fn ext_set_input_monitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a parameter of an FX in the normal FX chain has changed its value.
    ///
    /// For REAPER < 5.95 this is also called for an FX in the input FX chain. In this case there's
    /// no way to know whether the given FX index refers to the normal or input FX chain.
    fn ext_set_fx_param(&self, args: ExtSetFxParamArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a parameter of an FX in the input FX chain has changed its value.
    ///
    /// Only called for REAPER >= 5.95.
    fn ext_set_fx_param_rec_fx(&self, args: ExtSetFxParamArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a an FX has been enabled or disabled.
    fn ext_set_fx_enabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the volume of a track send has changed.
    fn ext_set_send_volume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the pan of a track send has changed.
    fn ext_set_send_pan(&self, args: ExtSetSendPanArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the volume of a track receive has changed.
    fn ext_set_recv_volume(&self, args: ExtSetRecvVolumeArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the pan of a track receive has changed.
    fn ext_set_recv_pan(&self, args: ExtSetRecvPanArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the pan of a track has changed.
    ///
    /// If a control surface supports this, it should ignore [`set_surface_pan`].
    ///
    /// [`set_surface_pan`]: #method.set_surface_pan
    fn ext_set_pan_ex(&self, args: ExtSetPanExArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a certain FX has gained focus.
    fn ext_set_focused_fx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a certain FX has been touched.
    fn ext_set_last_touched_fx(&self, args: ExtSetLastTouchedFxArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the user interface of a certain FX has been opened.
    fn ext_set_fx_open(&self, args: ExtSetFxOpenArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when an FX has been added, removed or when it changed its position in the chain.
    fn ext_set_fx_change(&self, args: ExtSetFxChangeArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when the master tempo or play rate has changed.
    fn ext_set_bpm_and_play_rate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        let _ = args;
        0
    }

    /// Called when a preset of a track FX has been selected.
    ///
    /// Since REAPER v6.12+dev0617
    fn ext_track_fx_preset_changed(&self, args: ExtTrackFxPresetChangedArgs) -> i32 {
        let _ = args;
        0
    }

    /// Should return `1` if [`get_touch_state()`] wants to deal with parameters other than
    /// volume and pan (at the moment this is width only).
    ///
    /// [`get_touch_state()`]: #method.get_touch_state
    fn ext_supports_extended_touch(&self, _: ExtSupportsExtendedTouchArgs) -> i32 {
        0
    }

    /// Clear all surface state and reset (harder reset than [`set_track_list_change`]).
    ///
    /// [`set_track_list_change`]: #method.set_track_list_change
    fn ext_reset(&self, _: ExtResetArgs) -> i32 {
        0
    }

    /// Called whenever project markers are changed.
    fn ext_set_project_marker_change(&self, _: ExtSetProjectMarkerChangeArgs) -> i32 {
        0
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetSurfaceVolumeArgs {
    pub track: MediaTrack,
    pub volume: ReaperVolumeValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetSurfacePanArgs {
    pub track: MediaTrack,
    pub pan: ReaperPanValue,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetSurfaceMuteArgs {
    pub track: MediaTrack,
    pub is_mute: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetSurfaceSelectedArgs {
    pub track: MediaTrack,
    pub is_selected: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetSurfaceSoloArgs {
    pub track: MediaTrack,
    pub is_solo: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetSurfaceRecArmArgs {
    pub track: MediaTrack,
    pub is_armed: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetPlayStateArgs {
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_recording: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetRepeatStateArgs {
    pub is_enabled: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetTrackTitleArgs<'a> {
    pub track: MediaTrack,
    pub name: &'a ReaperStr,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetTouchStateArgs {
    pub track: MediaTrack,
    pub parameter_type: TouchedParameterType,
}

/// Type of a touched parameter.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TouchedParameterType {
    Volume,
    Pan,
    Width,
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl TouchedParameterType {
    /// Converts an integer as returned by the low-level API to a type.
    fn from_raw(value: i32) -> TouchedParameterType {
        use TouchedParameterType::*;
        match value {
            0 => Volume,
            1 => Pan,
            2 => Width,
            x => Unknown(Hidden(x)),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetAutoModeArgs {
    pub mode: AutomationMode,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct OnTrackSelectionArgs {
    pub track: MediaTrack,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IsKeyDownArgs {
    pub key: ModKey,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtendedArgs {
    /// Represents the type of event.
    pub call: i32,
    pub parm_1: *mut c_void,
    pub parm_2: *mut c_void,
    pub parm_3: *mut c_void,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetInputMonitorArgs {
    pub track: MediaTrack,
    pub mode: InputMonitoringMode,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetFxParamArgs {
    pub track: MediaTrack,
    pub fx_index: u32,
    pub param_index: u32,
    pub param_value: ReaperNormalizedFxParamValue,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetFxEnabledArgs {
    pub track: MediaTrack,
    pub fx_location: VersionDependentTrackFxLocation,
    pub is_enabled: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetSendVolumeArgs {
    pub track: MediaTrack,
    /// This starts with the hardware output sends and continues with the track sends.
    pub send_index: u32,
    pub volume: ReaperVolumeValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetSendPanArgs {
    pub track: MediaTrack,
    /// This starts with the hardware output sends and continues with the track sends.
    pub send_index: u32,
    pub pan: ReaperPanValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetRecvVolumeArgs {
    pub track: MediaTrack,
    pub receive_index: u32,
    pub volume: ReaperVolumeValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetRecvPanArgs {
    pub track: MediaTrack,
    pub receive_index: u32,
    pub pan: ReaperPanValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetPanExArgs {
    pub track: MediaTrack,
    pub pan: Pan,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetFocusedFxArgs {
    pub fx_location: Option<QualifiedFxLocation>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetLastTouchedFxArgs {
    pub fx_location: Option<QualifiedFxLocation>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetFxOpenArgs {
    pub track: MediaTrack,
    pub fx_location: VersionDependentTrackFxLocation,
    pub is_open: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetFxChangeArgs {
    pub track: MediaTrack,
    /// In REAPER < 5.95 this is `None` because we can't know if the change happened in the normal
    /// or input FX chain.
    pub fx_chain_type: Option<TrackFxChainType>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtTrackFxPresetChangedArgs {
    pub track: MediaTrack,
    pub fx_location: TrackFxLocation,
}

/// Just a placeholder for upward compatibility reasons.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSupportsExtendedTouchArgs;

/// Just a placeholder for upward compatibility reasons.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtResetArgs;

/// Just a placeholder for upward compatibility reasons.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtSetProjectMarkerChangeArgs;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ExtSetBpmAndPlayRateArgs {
    pub tempo: Option<Bpm>,
    pub play_rate: Option<PlaybackSpeedFactor>,
}

/// Modifier key according to
/// [this list](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
///
/// You can find some frequently used predefined keys in [`mod_keys`](mod_keys/index.html).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModKey(pub(crate) i32);

impl ModKey {
    /// Creates a modifier key.
    pub const fn new(raw: i32) -> Self {
        Self(raw)
    }

    /// Returns the wrapped value.
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Contains predefined modifier keys.
pub mod mod_keys {
    use crate::ModKey;
    use reaper_low::raw;

    pub const SHIFT: ModKey = ModKey(raw::VK_SHIFT as _);
    pub const CONTROL: ModKey = ModKey(raw::VK_CONTROL as _);
    pub const MENU: ModKey = ModKey(raw::VK_MENU as _);
}

/// Location of a track or take FX including the parent track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct QualifiedFxLocation {
    /// Parent track.
    pub track: MediaTrack,
    /// Location of FX on the parent track.
    pub fx_location: VersionDependentFxLocation,
}

/// Location of a track or take FX.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum VersionDependentFxLocation {
    /// It's a take FX.
    ///
    /// The take index is currently not exposed by REAPER.
    TakeFx {
        /// Index of the item on that track.
        item_index: u32,
        /// Index of the FX within the take FX chain.
        fx_index: u32,
    },
    /// It's a track FX.
    TrackFx(VersionDependentTrackFxLocation),
}

/// Location of a track FX.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum VersionDependentTrackFxLocation {
    /// This is REAPER < 5.95.
    ///
    /// The given index can refer either to the input or output FX chain - we don't know.
    Old(u32),
    /// This is REAPER >= 5.95.
    ///
    /// It's possible to distinguish between input and output FX.
    New(TrackFxLocation),
}

#[derive(Debug)]
pub(crate) struct ControlSurfaceAdapter {
    // As you can see, this is a `Box` instead of a generic type parameter! Reasoning:
    //
    // - `ReaperSession` needs to store control surface instances of consumer-defined unknown types
    //   in one list. This requires boxing already.
    // - The way we work with consumer-provided control surfaces is to give ownership to REAPER and
    //   get it back at some point while being able to restore the original.
    //
    // Because control surface calls happen in the main thread, the dynamic dispatch this
    // is absolutely no issue.
    delegate: Box<dyn ControlSurface>,
    // Capabilities depending on REAPER version
    supports_detection_of_input_fx: bool,
    supports_detection_of_input_fx_in_set_fx_change: bool,
}

impl ControlSurfaceAdapter {
    pub fn new(
        delegate: Box<dyn ControlSurface>,
        reaper_version: &ReaperVersion,
    ) -> ControlSurfaceAdapter {
        let reaper_version_5_95: ReaperVersion = ReaperVersion::new("5.95");
        ControlSurfaceAdapter {
            delegate,
            // since pre1,
            supports_detection_of_input_fx: reaper_version >= &reaper_version_5_95,
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: reaper_version >= &reaper_version_5_95,
        }
    }

    pub fn into_delegate(self) -> Box<dyn ControlSurface> {
        self.delegate
    }

    unsafe fn get_as_qualified_fx_ref(
        &self,
        media_track_ptr: *mut c_void,
        media_item_ptr: *mut c_void,
        fx_index_ptr: *mut c_void,
    ) -> Option<QualifiedFxLocation> {
        if media_track_ptr.is_null() {
            return None;
        }
        Some(QualifiedFxLocation {
            track: require_non_null_panic(media_track_ptr as *mut raw::MediaTrack),
            fx_location: if media_item_ptr.is_null() {
                VersionDependentFxLocation::TrackFx(
                    self.get_as_version_dependent_track_fx_ref(fx_index_ptr),
                )
            } else {
                VersionDependentFxLocation::TakeFx {
                    item_index: deref_as::<i32>(media_item_ptr).expect("media item pointer is null")
                        as u32,
                    fx_index: deref_as::<i32>(fx_index_ptr).expect("FX index pointer is null")
                        as u32,
                }
            },
        })
    }

    unsafe fn get_as_version_dependent_track_fx_ref(
        &self,
        ptr: *mut c_void,
    ) -> VersionDependentTrackFxLocation {
        let fx_index = deref_as::<i32>(ptr).expect("FX index is null");
        if self.supports_detection_of_input_fx {
            VersionDependentTrackFxLocation::New(TrackFxLocation::from_raw(fx_index))
        } else {
            VersionDependentTrackFxLocation::Old(fx_index as u32)
        }
    }
}

impl reaper_low::IReaperControlSurface for ControlSurfaceAdapter {
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
        self.delegate.run();
    }

    fn SetTrackListChange(&self) {
        self.delegate.set_track_list_change()
    }

    fn SetSurfaceVolume(&self, trackid: *mut raw::MediaTrack, volume: f64) {
        self.delegate.set_surface_volume(SetSurfaceVolumeArgs {
            track: require_non_null_panic(trackid),
            volume: ReaperVolumeValue(volume),
        })
    }

    fn SetSurfacePan(&self, trackid: *mut raw::MediaTrack, pan: f64) {
        self.delegate.set_surface_pan(SetSurfacePanArgs {
            track: require_non_null_panic(trackid),
            pan: ReaperPanValue(pan),
        })
    }

    fn SetSurfaceMute(&self, trackid: *mut raw::MediaTrack, mute: bool) {
        self.delegate.set_surface_mute(SetSurfaceMuteArgs {
            track: require_non_null_panic(trackid),
            is_mute: mute,
        })
    }

    fn SetSurfaceSelected(&self, trackid: *mut raw::MediaTrack, selected: bool) {
        self.delegate.set_surface_selected(SetSurfaceSelectedArgs {
            track: require_non_null_panic(trackid),
            is_selected: selected,
        })
    }

    fn SetSurfaceSolo(&self, trackid: *mut raw::MediaTrack, solo: bool) {
        self.delegate.set_surface_solo(SetSurfaceSoloArgs {
            track: require_non_null_panic(trackid),
            is_solo: solo,
        })
    }

    fn SetSurfaceRecArm(&self, trackid: *mut raw::MediaTrack, recarm: bool) {
        self.delegate.set_surface_rec_arm(SetSurfaceRecArmArgs {
            track: require_non_null_panic(trackid),
            is_armed: recarm,
        })
    }

    fn SetPlayState(&self, play: bool, pause: bool, rec: bool) {
        self.delegate.set_play_state(SetPlayStateArgs {
            is_playing: play,
            is_paused: pause,
            is_recording: rec,
        })
    }

    fn SetRepeatState(&self, rep: bool) {
        self.delegate
            .set_repeat_state(SetRepeatStateArgs { is_enabled: rep })
    }

    fn SetTrackTitle(&self, trackid: *mut raw::MediaTrack, title: *const i8) {
        self.delegate.set_track_title(SetTrackTitleArgs {
            track: require_non_null_panic(trackid),
            name: unsafe { ReaperStr::from_ptr(title) },
        })
    }

    fn GetTouchState(&self, trackid: *mut raw::MediaTrack, isPan: i32) -> bool {
        self.delegate.get_touch_state(GetTouchStateArgs {
            track: require_non_null_panic(trackid),
            parameter_type: TouchedParameterType::from_raw(isPan),
        })
    }

    fn SetAutoMode(&self, mode: i32) {
        self.delegate.set_auto_mode(SetAutoModeArgs {
            mode: AutomationMode::from_raw(mode),
        })
    }

    fn ResetCachedVolPanStates(&self) {
        self.delegate.reset_cached_vol_pan_states()
    }

    fn OnTrackSelection(&self, trackid: *mut raw::MediaTrack) {
        self.delegate.on_track_selection(OnTrackSelectionArgs {
            track: require_non_null_panic(trackid),
        })
    }

    fn IsKeyDown(&self, key: i32) -> bool {
        self.delegate
            .is_key_down(IsKeyDownArgs { key: ModKey(key) })
    }

    fn Extended(
        &self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        let result = unsafe {
            // TODO-low Delegate all known CSURF_EXT_ constants
            match call {
                raw::CSURF_EXT_SETINPUTMONITOR => {
                    let recmon: i32 = deref_as(parm2).expect("recmon pointer is null");
                    self.delegate.ext_set_input_monitor(ExtSetInputMonitorArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        mode: InputMonitoringMode::from_raw(recmon),
                    })
                }
                raw::CSURF_EXT_SETFXPARAM | raw::CSURF_EXT_SETFXPARAM_RECFX => {
                    let fxidx_and_paramidx: i32 =
                        deref_as(parm2).expect("fx/param index pointer is null");
                    let value: f64 = deref_as(parm3).expect("value pointer is null");
                    let newtype_value = ReaperNormalizedFxParamValue::new(value);
                    let fx_index = (fxidx_and_paramidx >> 16) & 0xffff;
                    let param_index = fxidx_and_paramidx & 0xffff;
                    let args = ExtSetFxParamArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        fx_index: fx_index as u32,
                        param_index: param_index as u32,
                        param_value: newtype_value,
                    };
                    match call {
                        raw::CSURF_EXT_SETFXPARAM => self.delegate.ext_set_fx_param(args),
                        raw::CSURF_EXT_SETFXPARAM_RECFX => {
                            self.delegate.ext_set_fx_param_rec_fx(args)
                        }
                        _ => unreachable!(),
                    }
                }
                raw::CSURF_EXT_SETFOCUSEDFX => {
                    self.delegate.ext_set_focused_fx(ExtSetFocusedFxArgs {
                        fx_location: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                    })
                }
                raw::CSURF_EXT_SETLASTTOUCHEDFX => {
                    self.delegate
                        .ext_set_last_touched_fx(ExtSetLastTouchedFxArgs {
                            fx_location: self.get_as_qualified_fx_ref(parm1, parm2, parm3),
                        })
                }
                raw::CSURF_EXT_SETFXOPEN => self.delegate.ext_set_fx_open(ExtSetFxOpenArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    fx_location: self.get_as_version_dependent_track_fx_ref(parm2),
                    is_open: interpret_as_bool(parm3),
                }),
                raw::CSURF_EXT_SETFXENABLED => {
                    if parm1.is_null() {
                        // Don't know how to handle that case. Maybe a bug in REAPER.
                        0
                    } else {
                        self.delegate.ext_set_fx_enabled(ExtSetFxEnabledArgs {
                            track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                            fx_location: self.get_as_version_dependent_track_fx_ref(parm2),
                            is_enabled: interpret_as_bool(parm3),
                        })
                    }
                }
                raw::CSURF_EXT_SETSENDVOLUME => {
                    self.delegate.ext_set_send_volume(ExtSetSendVolumeArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        send_index: deref_as::<i32>(parm2).expect("send index pointer is null")
                            as u32,
                        volume: deref_as(parm3).expect("volume pointer is null"),
                    })
                }
                raw::CSURF_EXT_SETSENDPAN => self.delegate.ext_set_send_pan(ExtSetSendPanArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    send_index: deref_as::<i32>(parm2).expect("send index pointer is null") as u32,
                    pan: deref_as(parm3).expect("pan pointer is null"),
                }),
                raw::CSURF_EXT_SETRECVVOLUME => {
                    self.delegate.ext_set_recv_volume(ExtSetRecvVolumeArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        receive_index: deref_as::<i32>(parm2)
                            .expect("receive index pointer is null")
                            as u32,
                        volume: deref_as(parm3).expect("volume pointer is null"),
                    })
                }
                raw::CSURF_EXT_SETRECVPAN => self.delegate.ext_set_recv_pan(ExtSetRecvPanArgs {
                    track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                    receive_index: deref_as::<i32>(parm2).expect("receive index pointer is null")
                        as u32,
                    pan: deref_as(parm3).expect("pan pointer is null"),
                }),
                raw::CSURF_EXT_SETPAN_EX => {
                    let mode: i32 = deref_as(parm3).expect("pan mode pointer is null");
                    use PanMode::*;
                    let pan_null_msg = "pan pointer is null";
                    let pan = match PanMode::from_raw(mode) {
                        BalanceV1 => Pan::BalanceV1(deref_as(parm2).expect(pan_null_msg)),
                        BalanceV4 => Pan::BalanceV4(deref_as(parm2).expect(pan_null_msg)),
                        StereoPan => Pan::StereoPan {
                            pan: deref_as(parm2).expect(pan_null_msg),
                            width: {
                                let next = (parm2 as *const f64).offset(1);
                                deref_as(next as _).expect("width is null")
                            },
                        },
                        DualPan => Pan::DualPan {
                            left: deref_as(parm2).expect("left pan is null"),
                            right: {
                                let next = (parm2 as *const f64).offset(1);
                                deref_as(next as _).expect("right pan is null")
                            },
                        },
                        Unknown(x) => Pan::Unknown(x),
                    };
                    self.delegate.ext_set_pan_ex(ExtSetPanExArgs {
                        track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                        pan,
                    })
                }
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
                            tempo: deref_as(parm1),
                            play_rate: deref_as(parm2),
                        })
                }
                raw::CSURF_EXT_TRACKFX_PRESET_CHANGED => {
                    self.delegate
                        .ext_track_fx_preset_changed(ExtTrackFxPresetChangedArgs {
                            track: require_non_null_panic(parm1 as *mut raw::MediaTrack),
                            fx_location: get_as_track_fx_location(parm2),
                        })
                }
                raw::CSURF_EXT_SUPPORTS_EXTENDED_TOUCH => self
                    .delegate
                    .ext_supports_extended_touch(ExtSupportsExtendedTouchArgs),
                raw::CSURF_EXT_RESET => self.delegate.ext_reset(ExtResetArgs),
                raw::CSURF_EXT_SETPROJECTMARKERCHANGE => self
                    .delegate
                    .ext_set_project_marker_change(ExtSetProjectMarkerChangeArgs),
                _ => 0,
            }
        };
        if result != 0 {
            // Call was processed in one of the type-safe methods. No need to call `extended`.
            return result;
        }
        unsafe {
            self.delegate.extended(ExtendedArgs {
                call,
                parm_1: parm1,
                parm_2: parm2,
                parm_3: parm3,
            })
        }
    }
}

unsafe fn deref_as<T: Copy>(ptr: *const c_void) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    let ptr = ptr as *const T;
    Some(*ptr)
}

unsafe fn interpret_as_bool(ptr: *mut c_void) -> bool {
    !ptr.is_null()
}

unsafe fn get_as_track_fx_location(ptr: *mut c_void) -> TrackFxLocation {
    let fx_index = deref_as::<i32>(ptr).expect("FX index is null");
    TrackFxLocation::from_raw(fx_index)
}
