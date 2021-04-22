use reaper_medium::{
    ControlSurface, ExtResetArgs, ExtSetBpmAndPlayRateArgs, ExtSetFocusedFxArgs,
    ExtSetFxChangeArgs, ExtSetFxEnabledArgs, ExtSetFxOpenArgs, ExtSetFxParamArgs,
    ExtSetInputMonitorArgs, ExtSetLastTouchedFxArgs, ExtSetPanExArgs,
    ExtSetProjectMarkerChangeArgs, ExtSetRecvPanArgs, ExtSetRecvVolumeArgs, ExtSetSendPanArgs,
    ExtSetSendVolumeArgs, ExtSupportsExtendedTouchArgs, ExtTrackFxPresetChangedArgs,
    GetTouchStateArgs, IsKeyDownArgs, OnTrackSelectionArgs, ReaperStr, SetAutoModeArgs,
    SetPlayStateArgs, SetRepeatStateArgs, SetSurfaceMuteArgs, SetSurfacePanArgs,
    SetSurfaceRecArmArgs, SetSurfaceSelectedArgs, SetSurfaceSoloArgs, SetSurfaceVolumeArgs,
    SetTrackTitleArgs,
};

use std::fmt::Debug;

/// This control surface "redirects" each callback method with event character into an enum value,
/// thereby enabling middleware-style composition of different control surface logic.
#[derive(Debug)]
pub struct MiddlewareControlSurface<M: ControlSurfaceMiddleware + Debug> {
    middleware: M,
}

pub trait ControlSurfaceMiddleware {
    fn run(&mut self) {}

    /// Should return `true` if this event is handled.
    fn handle_event(&self, event: ControlSurfaceEvent) -> bool {
        let _ = event;
        false
    }

    fn get_type_string(&self) -> Option<&ReaperStr> {
        None
    }

    fn get_desc_string(&self) -> Option<&ReaperStr> {
        None
    }

    fn get_config_string(&self) -> Option<&ReaperStr> {
        None
    }

    fn get_touch_state(&self, args: GetTouchStateArgs) -> bool {
        let _ = args;
        false
    }

    fn is_key_down(&self, args: IsKeyDownArgs) -> bool {
        let _ = args;
        false
    }

    fn ext_supports_extended_touch(&self, _: ExtSupportsExtendedTouchArgs) -> i32 {
        0
    }
}

impl<H: ControlSurfaceMiddleware + Debug> MiddlewareControlSurface<H> {
    pub fn new(middleware: H) -> MiddlewareControlSurface<H> {
        MiddlewareControlSurface { middleware }
    }

    pub fn middleware(&self) -> &H {
        &self.middleware
    }

    pub fn middleware_mut(&mut self) -> &mut H {
        &mut self.middleware
    }
}

impl<H: ControlSurfaceMiddleware + Debug> ControlSurface for MiddlewareControlSurface<H> {
    fn run(&mut self) {
        self.middleware.run();
    }

    fn close_no_reset(&self) {
        self.middleware
            .handle_event(ControlSurfaceEvent::CloseNoReset);
    }

    fn set_track_list_change(&self) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetTrackListChange);
    }

    fn set_surface_volume(&self, args: SetSurfaceVolumeArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfaceVolume(args));
    }

    fn set_surface_pan(&self, args: SetSurfacePanArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfacePan(args));
    }

    fn set_surface_mute(&self, args: SetSurfaceMuteArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfaceMute(args));
    }

    fn set_surface_selected(&self, args: SetSurfaceSelectedArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfaceSelected(args));
    }

    fn set_surface_solo(&self, args: SetSurfaceSoloArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfaceSolo(args));
    }

    fn set_surface_rec_arm(&self, args: SetSurfaceRecArmArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetSurfaceRecArm(args));
    }

    fn set_play_state(&self, args: SetPlayStateArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetPlayState(args));
    }

    fn set_repeat_state(&self, args: SetRepeatStateArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetRepeatState(args));
    }

    fn set_track_title(&self, args: SetTrackTitleArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetTrackTitle(args));
    }

    fn set_auto_mode(&self, args: SetAutoModeArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::SetAutoMode(args));
    }

    fn reset_cached_vol_pan_states(&self) {
        self.middleware
            .handle_event(ControlSurfaceEvent::ResetCachedVolPanStates);
    }

    fn on_track_selection(&self, args: OnTrackSelectionArgs) {
        self.middleware
            .handle_event(ControlSurfaceEvent::OnTrackSelection(args));
    }

    fn ext_set_input_monitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetInputMonitor(args)),
        )
    }

    fn ext_set_fx_param(&self, args: ExtSetFxParamArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFxParam(args)),
        )
    }

    fn ext_set_fx_param_rec_fx(&self, args: ExtSetFxParamArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFxParamRecFx(args)),
        )
    }

    fn ext_set_fx_enabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFxEnabled(args)),
        )
    }

    fn ext_set_send_volume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetSendVolume(args)),
        )
    }

    fn ext_set_send_pan(&self, args: ExtSetSendPanArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetSendPan(args)),
        )
    }

    fn ext_set_recv_volume(&self, args: ExtSetRecvVolumeArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetRecvVolume(args)),
        )
    }

    fn ext_set_recv_pan(&self, args: ExtSetRecvPanArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetRecvPan(args)),
        )
    }

    fn ext_set_pan_ex(&self, args: ExtSetPanExArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetPanExt(args)),
        )
    }

    fn ext_set_focused_fx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFocusedFx(args)),
        )
    }

    fn ext_set_last_touched_fx(&self, args: ExtSetLastTouchedFxArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetLastTouchedFx(args)),
        )
    }

    fn ext_set_fx_open(&self, args: ExtSetFxOpenArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFxOpen(args)),
        )
    }

    fn ext_set_fx_change(&self, args: ExtSetFxChangeArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetFxChange(args)),
        )
    }

    fn ext_set_bpm_and_play_rate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetBpmAndPlayRate(args)),
        )
    }

    fn ext_track_fx_preset_changed(&self, args: ExtTrackFxPresetChangedArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtTrackFxPresetChanged(args)),
        )
    }

    fn ext_reset(&self, args: ExtResetArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtReset(args)),
        )
    }

    fn ext_set_project_marker_change(&self, args: ExtSetProjectMarkerChangeArgs) -> i32 {
        to_int(
            self.middleware
                .handle_event(ControlSurfaceEvent::ExtSetProjectMarkerChange(args)),
        )
    }

    fn get_type_string(&self) -> Option<&ReaperStr> {
        self.middleware.get_type_string()
    }

    fn get_desc_string(&self) -> Option<&ReaperStr> {
        self.middleware.get_desc_string()
    }

    fn get_config_string(&self) -> Option<&ReaperStr> {
        self.middleware.get_config_string()
    }

    fn get_touch_state(&self, args: GetTouchStateArgs) -> bool {
        self.middleware.get_touch_state(args)
    }

    fn is_key_down(&self, args: IsKeyDownArgs) -> bool {
        self.middleware.is_key_down(args)
    }

    fn ext_supports_extended_touch(&self, args: ExtSupportsExtendedTouchArgs) -> i32 {
        self.middleware.ext_supports_extended_touch(args)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ControlSurfaceEvent<'a> {
    CloseNoReset,
    SetTrackListChange,
    SetSurfaceVolume(SetSurfaceVolumeArgs),
    SetSurfacePan(SetSurfacePanArgs),
    SetSurfaceMute(SetSurfaceMuteArgs),
    SetSurfaceSelected(SetSurfaceSelectedArgs),
    SetSurfaceSolo(SetSurfaceSoloArgs),
    SetSurfaceRecArm(SetSurfaceRecArmArgs),
    SetPlayState(SetPlayStateArgs),
    SetRepeatState(SetRepeatStateArgs),
    SetTrackTitle(SetTrackTitleArgs<'a>),
    SetAutoMode(SetAutoModeArgs),
    ResetCachedVolPanStates,
    OnTrackSelection(OnTrackSelectionArgs),
    ExtSetInputMonitor(ExtSetInputMonitorArgs),
    ExtSetFxParam(ExtSetFxParamArgs),
    ExtSetFxParamRecFx(ExtSetFxParamArgs),
    ExtSetFxEnabled(ExtSetFxEnabledArgs),
    ExtSetSendVolume(ExtSetSendVolumeArgs),
    ExtSetSendPan(ExtSetSendPanArgs),
    ExtSetRecvVolume(ExtSetRecvVolumeArgs),
    ExtSetRecvPan(ExtSetRecvPanArgs),
    ExtSetPanExt(ExtSetPanExArgs),
    ExtSetFocusedFx(ExtSetFocusedFxArgs),
    ExtSetLastTouchedFx(ExtSetLastTouchedFxArgs),
    ExtSetFxOpen(ExtSetFxOpenArgs),
    ExtSetFxChange(ExtSetFxChangeArgs),
    ExtSetBpmAndPlayRate(ExtSetBpmAndPlayRateArgs),
    ExtTrackFxPresetChanged(ExtTrackFxPresetChangedArgs),
    ExtReset(ExtResetArgs),
    ExtSetProjectMarkerChange(ExtSetProjectMarkerChangeArgs),
}

fn to_int(value: bool) -> i32 {
    if value { 1 } else { 0 }
}
