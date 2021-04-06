use crate::{
    get_media_track_guid, ControlSurfaceEvent, Fx, FxParameter, Guid, Project, Reaper, Track,
    TrackRoute,
};
use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::{
    reaper_str, AutomationMode, Bpm, ExtSetFxParamArgs, GlobalAutomationModeOverride,
    InputMonitoringMode, MediaTrack, Pan, PanMode, PlayState, PlaybackSpeedFactor, ReaProject,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperStr, ReaperVersion, ReaperVolumeValue,
    TrackAttributeKey, TrackFxChainType, TrackLocation, TrackSendCategory, TrackSendDirection,
    VersionDependentFxLocation, VersionDependentTrackFxLocation,
};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ChangeDetectionMiddleware {
    num_track_set_changes_left_to_be_propagated: Cell<u32>,
    last_active_project: Cell<Project>,
    last_global_automation_mode_override: Cell<Option<GlobalAutomationModeOverride>>,
    project_datas: RefCell<ProjectDataMap>,
    fx_has_been_touched_just_a_moment_ago: Cell<bool>,
    // Capabilities depending on REAPER version
    supports_detection_of_input_fx: bool,
    supports_detection_of_input_fx_in_set_fx_change: bool,
}

type ProjectDataMap = HashMap<ReaProject, TrackDataMap>;
type TrackDataMap = HashMap<MediaTrack, TrackData>;

/// Keeps current track values for detecting real value changes.
///
/// When REAPER reads automation, the callbacks are fired like crazy, even if the value
/// has not changed.
#[derive(Debug)]
struct TrackData {
    volume: ReaperVolumeValue,
    pan: Pan,
    selected: bool,
    mute: bool,
    solo: bool,
    recarm: bool,
    number: Option<TrackLocation>,
    recmonitor: InputMonitoringMode,
    automation_mode: AutomationMode,
    recinput: i32,
    guid: Guid,
    receive_count: u32,
    track_send_count: u32,
    output_send_count: u32,
    send_volumes: HashMap<u32, ReaperVolumeValue>,
    send_pans: HashMap<u32, ReaperPanValue>,
    receive_volumes: HashMap<u32, ReaperVolumeValue>,
    receive_pans: HashMap<u32, ReaperPanValue>,
    fx_param_values: HashMap<TrackFxKey, ReaperNormalizedFxParamValue>,
    fx_chain_pair: FxChainPair,
}

impl TrackData {
    /// Returns true if it has changed along with the old value.
    fn update_send_volume(
        &mut self,
        index: u32,
        v: ReaperVolumeValue,
    ) -> (bool, Option<ReaperVolumeValue>) {
        match self.send_volumes.insert(index, v) {
            None => (true, None),
            Some(prev) => (v != prev, Some(prev)),
        }
    }

    /// Returns true if it has changed along with the old value.
    fn update_send_pan(&mut self, index: u32, v: ReaperPanValue) -> (bool, Option<ReaperPanValue>) {
        match self.send_pans.insert(index, v) {
            None => (true, None),
            Some(prev) => (v != prev, Some(prev)),
        }
    }

    /// Returns true if it has changed along with the old value.
    fn update_receive_volume(
        &mut self,
        index: u32,
        v: ReaperVolumeValue,
    ) -> (bool, Option<ReaperVolumeValue>) {
        match self.receive_volumes.insert(index, v) {
            None => (true, None),
            Some(prev) => (v != prev, Some(prev)),
        }
    }

    /// Returns true if it has changed along with the old value.
    fn update_receive_pan(
        &mut self,
        index: u32,
        v: ReaperPanValue,
    ) -> (bool, Option<ReaperPanValue>) {
        match self.receive_pans.insert(index, v) {
            None => (true, None),
            Some(prev) => (v != prev, Some(prev)),
        }
    }

    /// Returns true if it has changed.
    fn update_fx_param_value(
        &mut self,
        is_input_fx: bool,
        fx_index: u32,
        param_index: u32,
        v: ReaperNormalizedFxParamValue,
    ) -> bool {
        let key = TrackFxKey {
            is_input_fx,
            fx_index,
            param_index,
        };
        match self.fx_param_values.insert(key, v) {
            None => true,
            Some(prev) => v != prev,
        }
    }
}

/// For detection of added or removed FX.
#[derive(Debug, Default)]
struct FxChainPair {
    input_fx_guids: HashSet<Guid>,
    output_fx_guids: HashSet<Guid>,
}

#[derive(Eq, PartialEq, Hash, Debug)]
struct TrackFxKey {
    is_input_fx: bool,
    fx_index: u32,
    param_index: u32,
}

#[derive(PartialEq)]
enum State {
    Normal,
    PropagatingTrackSetChanges,
}

impl Default for ChangeDetectionMiddleware {
    fn default() -> Self {
        let version = Reaper::get().version();
        let last_active_project = Reaper::get().current_project();
        let reaper_version_5_95 = ReaperVersion::new("5.95");
        ChangeDetectionMiddleware {
            num_track_set_changes_left_to_be_propagated: Default::default(),
            last_active_project: Cell::new(last_active_project),
            last_global_automation_mode_override: Cell::new(
                Reaper::get().global_automation_override(),
            ),
            project_datas: Default::default(),
            fx_has_been_touched_just_a_moment_ago: Default::default(),
            // since pre1,
            supports_detection_of_input_fx: version >= reaper_version_5_95,
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: version >= reaper_version_5_95,
        }
    }
}

impl ChangeDetectionMiddleware {
    pub fn new() -> ChangeDetectionMiddleware {
        Default::default()
    }

    pub fn reset(&self, handle_change: impl FnMut(ChangeEvent) + Copy) {
        // REAPER doesn't seem to call this automatically when the surface is registered. In our
        // case it's important to call this not at the first change of something (e.g. arm
        // button pressed) but immediately. Because it captures the initial project/track/FX
        // state. If we don't do this immediately, then it happens that change events (e.g.
        // track arm changed) are not reported because the initial state was unknown.
        // TODO-low This executes a bunch of REAPER functions right on start. Maybe do more lazily
        // on activate?  But before activate we can do almost nothing because
        // execute_on_main_thread doesn't work.
        self.react_to_track_list_change(Reaper::get().current_project(), handle_change);
    }

    pub fn process(
        &self,
        event: ControlSurfaceEvent,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) -> bool {
        use ControlSurfaceEvent::*;
        match event {
            SetTrackListChange => self.set_track_list_change(handle_change),
            SetSurfacePan(args) => {
                let td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                // This is mostly handled by ExtSetPanExt already but there's a situation when
                // ExtSetPanExt is not triggered: When Programmatically changing pan via
                // `CSurf_SetSurfacePan`, e.g. when users changes pan via ReaLearn, not via REAPER
                // UI.
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackPanChanged(TrackPanChangedEvent {
                    touched: false,
                    track,
                    old_value: td.pan,
                    new_value: AvailablePanValue::Incomplete(args.pan),
                }));
            }
            SetSurfaceVolume(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return false,
                    Some(td) => td,
                };
                let track = Track::new(args.track, None);
                if td.volume != args.volume {
                    let old = td.volume;
                    td.volume = args.volume;
                    handle_change(ChangeEvent::TrackVolumeChanged(TrackVolumeChangedEvent {
                        touched: !self.track_parameter_is_automated(&track, reaper_str!("Volume")),
                        track: track.clone(),
                        old_value: old,
                        new_value: args.volume,
                    }));
                }
                let new_automation_mode = unsafe {
                    Reaper::get().medium_reaper.get_track_automation_mode(args.track)
                };
                if td.automation_mode != new_automation_mode {
                    let old = td.automation_mode;
                    td.automation_mode = new_automation_mode;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackAutomationModeChanged(TrackAutomationModeChangedEvent {
                        track,
                        old_value: old,
                        new_value: new_automation_mode,
                    }));
                }
                let new_automation_override = Reaper::get().global_automation_override();
                let old_automation_override = self.last_global_automation_mode_override.get();
                if old_automation_override != new_automation_override {
                    self.last_global_automation_mode_override.set(new_automation_override);
                    handle_change(ChangeEvent::GlobalAutomationOverrideChanged(GlobalAutomationOverrideChangedEvent {
                        project: track.project(),
                        old_value: old_automation_override,
                        new_value: new_automation_override,
                    }));
                }
            }
            SetSurfaceMute(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let old = td.mute;
                if td.mute != args.is_mute {
                    td.mute = args.is_mute;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackMuteChanged(TrackMuteChangedEvent {
                        touched: !self.track_parameter_is_automated(&track, reaper_str!("Mute")),
                        track,
                        old_value: old,
                        new_value: args.is_mute,
                    }));
                }
            }
            SetSurfaceSelected(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let old = td.selected;
                if td.selected != args.is_selected {
                    td.selected = args.is_selected;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackSelectedChanged(
                        TrackSelectedChangedEvent {
                            track,
                            old_value: old,
                            new_value: args.is_selected,
                        },
                    ));
                }
            }
            SetSurfaceSolo(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let old = td.solo;
                if td.solo != args.is_solo {
                    td.solo = args.is_solo;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackSoloChanged(TrackSoloChangedEvent {
                        track,
                        old_value: old,
                        new_value: args.is_solo,
                    }));
                }
            }
            SetSurfaceRecArm(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let old = td.recarm;
                if td.recarm != args.is_armed {
                    td.recarm = args.is_armed;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackArmChanged(TrackArmChangedEvent {
                        track,
                        old_value: old,
                        new_value: args.is_armed,
                    }));
                }
            }
            SetTrackTitle(args) => {
                if self.state() == State::PropagatingTrackSetChanges {
                    self.decrease_num_track_set_changes_left_to_be_propagated();
                    return true;
                }
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackNameChanged(TrackNameChangedEvent {
                    track,
                }));
            }
            ExtSetInputMonitor(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let old = td.recmonitor;
                if td.recmonitor != args.mode {
                    td.recmonitor = args.mode;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackInputMonitoringChanged(
                        TrackInputMonitoringChangedEvent {
                            track,
                            old_value: old,
                            new_value: args.mode,
                        },
                    ));
                }
                let recinput = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .get_media_track_info_value(args.track, TrackAttributeKey::RecInput)
                        as i32
                };
                if td.recinput != recinput {
                    td.recinput = recinput;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackInputChanged(TrackInputChangedEvent {
                        track,
                    }));
                }
            }
            ExtSetFxParam(args) => self.fx_param_set(args, false, handle_change),
            ExtSetFxParamRecFx(args) => self.fx_param_set(args, true, handle_change),
            ExtSetFxEnabled(args) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(args.track, None);
                if let Some(fx) = self.fx_from_parm_fx_index(&track, args.fx_location, None, None) {
                    handle_change(ChangeEvent::FxEnabledChanged(FxEnabledChangedEvent {
                        fx,
                        new_value: args.is_enabled,
                    }));
                }
            }
            ExtSetSendVolume(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let (changed, old) = td.update_send_volume(args.send_index, args.volume);
                if !changed {
                    return true;
                }
                let track = Track::new(args.track, None);
                let is_automated =
                    self.track_parameter_is_automated(&track, reaper_str!("Send Volume"));
                let route = TrackRoute::new(track, TrackSendDirection::Send, args.send_index);
                handle_change(ChangeEvent::TrackRouteVolumeChanged(
                    TrackRouteVolumeChangedEvent {
                        touched: !is_automated,
                        route,
                        old_value: old,
                        new_value: args.volume,
                    },
                ));
            }
            ExtSetSendPan(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let (changed, old) = td.update_send_pan(args.send_index, args.pan);
                if !changed {
                    return true;
                }
                let track = Track::new(args.track, None);
                let is_automated =
                    self.track_parameter_is_automated(&track, reaper_str!("Send Pan"));
                let route = TrackRoute::new(track, TrackSendDirection::Send, args.send_index);
                handle_change(ChangeEvent::TrackRoutePanChanged(TrackRoutePanChangedEvent {
                    touched: !is_automated,
                    route,
                    old_value: old,
                    new_value: args.pan,
                }));
            }
            ExtSetRecvVolume(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let (changed, old) = td.update_receive_volume(args.receive_index, args.volume);
                if !changed {
                    return true;
                }
                let track = Track::new(args.track, None);
                let is_automated =
                    self.track_parameter_is_automated(&track, reaper_str!("Send Volume"));
                let route = TrackRoute::new(track, TrackSendDirection::Receive, args.receive_index);
                handle_change(ChangeEvent::TrackRouteVolumeChanged(
                    TrackRouteVolumeChangedEvent {
                        touched: !is_automated,
                        route,
                        old_value: old,
                        new_value: args.volume,
                    },
                ));
            }
            ExtSetRecvPan(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                let (changed, old) = td.update_receive_pan(args.receive_index, args.pan);
                if !changed {
                    return true;
                }
                let track = Track::new(args.track, None);
                let is_automated =
                    self.track_parameter_is_automated(&track, reaper_str!("Send Pan"));
                let route = TrackRoute::new(track, TrackSendDirection::Receive, args.receive_index);
                handle_change(ChangeEvent::TrackRoutePanChanged(TrackRoutePanChangedEvent {
                    touched: !is_automated,
                    route,
                    old_value: old,
                    new_value: args.pan,
                }));
            }
            ExtSetPanExt(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return true,
                    Some(td) => td,
                };
                if td.pan == args.pan {
                    return true;
                }
                let old = td.pan;
                td.pan = args.pan;
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackPanChanged(TrackPanChangedEvent {
                    touched: !self.track_parameter_is_automated(&track, reaper_str!("Pan")),
                    track: track.clone(),
                    old_value: old,
                    new_value: AvailablePanValue::Complete(args.pan),
                }));
            }
            ExtSetFocusedFx(args) => {
                let fx_ref = match args.fx_location {
                    None => {
                        // Clear focused FX
                        handle_change(ChangeEvent::FxFocused(FxFocusedEvent { fx: None }));
                        return true;
                    }
                    Some(r) => r,
                };
                use VersionDependentFxLocation::*;
                match fx_ref.fx_location {
                    TakeFx { .. } => {
                        // TODO Not handled right now
                    }
                    TrackFx(track_fx_ref) => {
                        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a
                        // nullptr.
                        let track = Track::new(fx_ref.track, None);
                        if let Some(fx) =
                            self.fx_from_parm_fx_index(&track, track_fx_ref, None, None)
                        {
                            // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER <
                            // 5.95-pre2 and on chunk manipulations
                            if let Some(mut td) = self.find_track_data(track.raw()) {
                                self.detect_fx_changes_on_track(
                                    &mut td.fx_chain_pair,
                                    track,
                                    true,
                                    !fx.is_input_fx(),
                                    fx.is_input_fx(),
                                    handle_change,
                                );
                                handle_change(ChangeEvent::FxFocused(FxFocusedEvent {
                                    fx: Some(fx),
                                }));
                            }
                        }
                    }
                }
            }
            ExtSetFxOpen(args) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(args.track, None);
                if let Some(fx) = self.fx_from_parm_fx_index(&track, args.fx_location, None, None) {
                    // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2
                    // and on chunk manipulations
                    if let Some(mut td) = self.find_track_data(track.raw()) {
                        self.detect_fx_changes_on_track(
                            &mut td.fx_chain_pair,
                            track,
                            true,
                            !fx.is_input_fx(),
                            fx.is_input_fx(),
                            handle_change,
                        );
                        let change_event = if args.is_open {
                            ChangeEvent::FxOpened(FxOpenedEvent { fx })
                        } else {
                            ChangeEvent::FxClosed(FxClosedEvent { fx })
                        };
                        handle_change(change_event);
                    }
                }
            }
            ExtSetFxChange(args) => {
                let track = Track::new(args.track, None);
                if let Some(mut td) = self.find_track_data(track.raw()) {
                    match args.fx_chain_type {
                        Some(t) => {
                            let is_input_fx = t == TrackFxChainType::InputFxChain;
                            self.detect_fx_changes_on_track(
                                &mut td.fx_chain_pair,
                                track,
                                true,
                                !is_input_fx,
                                is_input_fx,
                                handle_change,
                            );
                        }
                        None => {
                            self.detect_fx_changes_on_track(
                                &mut td.fx_chain_pair,
                                track,
                                true,
                                true,
                                true,
                                handle_change,
                            );
                        }
                    }
                }
            }
            ExtSetLastTouchedFx(_) => {
                self.fx_has_been_touched_just_a_moment_ago.replace(true);
            }
            ExtSetBpmAndPlayRate(args) => {
                if let Some(tempo) = args.tempo {
                    handle_change(ChangeEvent::MasterTempoChanged(MasterTempoChangedEvent {
                        project: Reaper::get().current_project(),
                        // At the moment we can't support touched for tempo because we always have
                        // a tempo map envelope and sometimes there are seemingly random invocations
                        // of this callback.
                        touched: false,
                        new_value: tempo,
                    }));
                }
                if let Some(play_rate) = args.play_rate {
                    handle_change(ChangeEvent::MasterPlayrateChanged(
                        MasterPlayrateChangedEvent {
                            project: Reaper::get().current_project(),
                            // The playrate affected by automation is something else, so we can
                            // always consider this as touched.
                            touched: true,
                            new_value: play_rate,
                        },
                    ));
                }
            }
            ExtTrackFxPresetChanged(args) => {
                let track = Track::new(args.track, None);
                let fx = track
                    .fx_by_query_index(args.fx_location.to_raw())
                    .expect("preset changed but FX not found");
                handle_change(ChangeEvent::FxPresetChanged(FxPresetChangedEvent { fx }));
            }
            SetPlayState(args) => {
                handle_change(ChangeEvent::PlayStateChanged(PlayStateChangedEvent {
                    project: Reaper::get().current_project(),
                    new_value: PlayState {
                        is_playing: args.is_playing,
                        is_paused: args.is_paused,
                        is_recording: args.is_recording,
                    },
                }));
            }
            SetRepeatState(args) => {
                handle_change(ChangeEvent::RepeatStateChanged(RepeatStateChangedEvent {
                    project: Reaper::get().current_project(),
                    new_value: args.is_enabled,
                }));
            }
            ExtSetProjectMarkerChange(_) => {
                handle_change(ChangeEvent::BookmarksChanged(BookmarksChangedEvent {
                    project: Reaper::get().current_project()
                }));
            }
            CloseNoReset |
            SetAutoMode(_) |
            ResetCachedVolPanStates |
            // TODO-low What's the difference to SetSurfaceSelected?
            OnTrackSelection(_) |
            ExtReset(_) => return false
        };
        true
    }

    // From REAPER > 5.95, parmFxIndex should be interpreted as query index. For earlier versions
    // it's a normal index
    // - which unfortunately doesn't contain information if the FX is on the normal FX chain or the
    //   input FX chain.
    // In this case a heuristic is applied to determine which chain it is. It gets more accurate
    // when paramIndex and paramValue are supplied.
    fn fx_from_parm_fx_index(
        &self,
        track: &Track,
        parm_fx_index: VersionDependentTrackFxLocation,
        param_index: Option<u32>,
        param_value: Option<ReaperNormalizedFxParamValue>,
    ) -> Option<Fx> {
        use VersionDependentTrackFxLocation::*;
        match parm_fx_index {
            Old(index) => {
                let is_input_fx = self.is_probably_input_fx(track, index, param_index, param_value);
                let fx_chain = if is_input_fx {
                    track.input_fx_chain()
                } else {
                    track.normal_fx_chain()
                };
                fx_chain.fx_by_index(index)
            }
            New(fx_ref) => track.fx_by_query_index(fx_ref.to_raw()),
        }
    }

    fn find_track_data_in_normal_state(&self, track: MediaTrack) -> Option<RefMut<TrackData>> {
        if self.state() == State::PropagatingTrackSetChanges {
            return None;
        }
        self.find_track_data(track)
    }

    fn find_track_data_map(&self, rea_project: ReaProject) -> Option<RefMut<TrackDataMap>> {
        if !self.project_datas.borrow().contains_key(&rea_project) {
            return None;
        }
        Some(RefMut::map(self.project_datas.borrow_mut(), |tds| {
            tds.get_mut(&rea_project).unwrap()
        }))
    }

    fn decrease_num_track_set_changes_left_to_be_propagated(&self) {
        let previous_value = self.num_track_set_changes_left_to_be_propagated.get();
        self.num_track_set_changes_left_to_be_propagated
            .replace(previous_value - 1);
    }

    fn find_track_data(&self, track: MediaTrack) -> Option<RefMut<TrackData>> {
        let project = unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_track_info_get_project(track)
                .unwrap_or_else(|| Reaper::get().current_project().raw())
        };
        let track_data_map = self.find_track_data_map(project)?;
        if !track_data_map.contains_key(&track) {
            return None;
        }
        Some(RefMut::map(track_data_map, |tdm| {
            tdm.get_mut(&track).unwrap()
        }))
    }

    fn track_parameter_is_automated(&self, track: &Track, parameter_name: &ReaperStr) -> bool {
        if !track.is_available() {
            return false;
        }
        let env = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_envelope_by_name(track.raw(), parameter_name)
        };
        if env.is_none() {
            return false;
        }
        use AutomationMode::*;
        !matches!(
            track.effective_automation_mode(),
            None | Some(TrimRead) | Some(Write)
        )
    }

    fn state(&self) -> State {
        if self.num_track_set_changes_left_to_be_propagated.get() == 0 {
            State::Normal
        } else {
            State::PropagatingTrackSetChanges
        }
    }

    fn is_probably_input_fx(
        &self,
        track: &Track,
        fx_index: u32,
        param_index: Option<u32>,
        normalized_value: Option<ReaperNormalizedFxParamValue>,
    ) -> bool {
        let td = match self.find_track_data(track.raw()) {
            None => {
                // Should not happen. In this case, an FX yet unknown to Realearn has sent a
                // parameter change
                return false;
            }
            Some(d) => d,
        };
        let could_be_input_fx = (fx_index as usize) < td.fx_chain_pair.input_fx_guids.len();
        let could_be_output_fx = (fx_index as usize) < td.fx_chain_pair.output_fx_guids.len();
        if !could_be_input_fx && !could_be_output_fx {
            false
        } else if could_be_input_fx && !could_be_output_fx {
            true
        } else {
            // Could be both
            let param_index = match param_index {
                None => {
                    // We don't have a parameter number at our disposal so we need to guess - we
                    // guess normal FX TODO-low
                    return false;
                }
                Some(i) => i,
            };
            // Compare parameter values (a heuristic but so what, it's just for MIDI learn)
            let normalized_value = match normalized_value {
                None => return true,
                Some(v) => v,
            };
            match track.normal_fx_chain().fx_by_index(fx_index) {
                None => true,
                Some(output_fx) => {
                    let output_fx_param = output_fx.parameter_by_index(param_index);
                    let is_probably_output_fx =
                        output_fx_param.reaper_normalized_value() == normalized_value;
                    !is_probably_output_fx
                }
            }
        }
    }

    fn fx_param_set(
        &self,
        args: ExtSetFxParamArgs,
        is_input_fx_if_supported: bool,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        let is_input_fx = if self.supports_detection_of_input_fx {
            is_input_fx_if_supported
        } else {
            self.is_probably_input_fx(
                &track,
                args.fx_index,
                Some(args.param_index),
                Some(args.param_value),
            )
        };
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if !td.update_fx_param_value(
            is_input_fx,
            args.fx_index,
            args.param_index,
            args.param_value,
        ) {
            return;
        }
        let fx_chain = if is_input_fx {
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        if let Some(fx) = fx_chain.fx_by_index(args.fx_index as u32) {
            let parameter = fx.parameter_by_index(args.param_index as u32);
            handle_change(ChangeEvent::FxParameterValueChanged(
                FxParameterValueChangedEvent {
                    touched: self.fx_has_been_touched_just_a_moment_ago.replace(false),
                    parameter,
                    new_value: args.param_value,
                },
            ));
        }
    }

    fn set_track_list_change(&self, handle_change: impl FnMut(ChangeEvent) + Copy) {
        // TODO-low Not multi-project compatible!
        let new_active_project = Reaper::get().current_project();
        self.num_track_set_changes_left_to_be_propagated
            .replace(new_active_project.track_count() + 1);
        self.react_to_track_list_change(new_active_project, handle_change);
    }

    fn react_to_track_list_change(
        &self,
        new_active_project: Project,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        if new_active_project != self.last_active_project.get() {
            let old = self.last_active_project.replace(new_active_project);
            handle_change(ChangeEvent::ProjectSwitched(ProjectSwitchedEvent {
                old_project: old,
                new_project: new_active_project,
            }));
        }
        self.remove_invalid_rea_projects(handle_change);
        self.detect_track_set_changes(handle_change);
    }

    fn remove_invalid_rea_projects(&self, mut handle_change: impl FnMut(ChangeEvent) + Copy) {
        self.project_datas.borrow_mut().retain(|rea_project, _| {
            if Reaper::get()
                .medium_reaper()
                .validate_ptr_2(CurrentProject, *rea_project)
            {
                true
            } else {
                handle_change(ChangeEvent::ProjectClosed(ProjectClosedEvent {
                    project: Project::new(*rea_project),
                }));
                false
            }
        });
    }

    fn detect_track_set_changes(&self, handle_change: impl FnMut(ChangeEvent) + Copy) {
        let project = Reaper::get().current_project();
        let mut project_datas = self.project_datas.borrow_mut();
        let track_datas = project_datas.entry(project.raw()).or_default();
        let old_track_count = track_datas.len() as u32;
        // +1 for master track
        let new_track_count = project.track_count() + 1;
        use std::cmp::Ordering::*;
        match new_track_count.cmp(&old_track_count) {
            Less => self.remove_invalid_media_tracks(project, track_datas, handle_change),
            Equal => self.update_media_track_positions_and_route_counts(
                project,
                track_datas,
                handle_change,
            ),
            Greater => self.add_missing_media_tracks(project, track_datas, handle_change),
        }
    }

    fn remove_invalid_media_tracks(
        &self,
        project: Project,
        track_datas: &mut TrackDataMap,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        track_datas.retain(|media_track, data| {
            if Reaper::get()
                .medium_reaper()
                .validate_ptr_2(Proj(project.raw()), *media_track)
            {
                true
            } else {
                let track = project.track_by_guid(&data.guid);
                handle_change(ChangeEvent::TrackRemoved(TrackRemovedEvent { track }));
                false
            }
        });
    }

    fn add_missing_media_tracks(
        &self,
        project: Project,
        track_datas: &mut TrackDataMap,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        for t in std::iter::once(project.master_track()).chain(project.tracks()) {
            let mt = t.raw();
            track_datas.entry(mt).or_insert_with(|| {
                let func = Reaper::get().medium_reaper();
                let mut td = unsafe {
                    TrackData {
                        volume: ReaperVolumeValue::new(
                            func.get_media_track_info_value(mt, TrackAttributeKey::Vol),
                        ),
                        pan: {
                            let project_pan_mode = {
                                let proj_conf_result = func
                                    .project_config_var_get_offs("panmode")
                                    .expect("couldn't find panmode project config");
                                let var = func.project_config_var_addr(
                                    Proj(project.raw()),
                                    proj_conf_result.offset,
                                ) as *mut i32;
                                let ipanmode = *var;
                                PanMode::from_raw(ipanmode)
                            };
                            use PanMode::*;
                            let track_pan_mode = func
                                .get_set_media_track_info_get_pan_mode(mt)
                                .unwrap_or(project_pan_mode);
                            match track_pan_mode {
                                BalanceV1 => {
                                    Pan::BalanceV1(func.get_set_media_track_info_get_pan(mt))
                                }
                                BalanceV4 => {
                                    Pan::BalanceV4(func.get_set_media_track_info_get_pan(mt))
                                }
                                StereoPan => Pan::StereoPan {
                                    pan: func.get_set_media_track_info_get_pan(mt),
                                    width: func.get_set_media_track_info_get_width(mt),
                                },
                                DualPan => Pan::DualPan {
                                    left: func.get_set_media_track_info_get_dual_pan_l(mt),
                                    right: func.get_set_media_track_info_get_dual_pan_r(mt),
                                },
                                Unknown(x) => Pan::Unknown(x),
                            }
                        },
                        selected: func.get_media_track_info_value(mt, TrackAttributeKey::Selected)
                            != 0.0,
                        mute: func.get_media_track_info_value(mt, TrackAttributeKey::Mute) != 0.0,
                        solo: func.get_media_track_info_value(mt, TrackAttributeKey::Solo) != 0.0,
                        recarm: func.get_media_track_info_value(mt, TrackAttributeKey::RecArm)
                            != 0.0,
                        number: func.get_set_media_track_info_get_track_number(mt),
                        recmonitor: func.get_set_media_track_info_get_rec_mon(mt),
                        recinput: func.get_media_track_info_value(mt, TrackAttributeKey::RecInput)
                            as i32,
                        automation_mode: func.get_track_automation_mode(mt),
                        guid: get_media_track_guid(mt),
                        receive_count: func.get_track_num_sends(mt, TrackSendCategory::Receive),
                        track_send_count: func.get_track_num_sends(mt, TrackSendCategory::Send),
                        output_send_count: func
                            .get_track_num_sends(mt, TrackSendCategory::HardwareOutput),
                        send_volumes: Default::default(),
                        send_pans: Default::default(),
                        receive_volumes: Default::default(),
                        receive_pans: Default::default(),
                        fx_param_values: Default::default(),
                        fx_chain_pair: Default::default(),
                    }
                };
                // TODO-low Use try_borrow_mut(). Then this just doesn't do anything if this event
                //  is currently thrown already. Right now it would panic, which is unreasonable.
                handle_change(ChangeEvent::TrackAdded(TrackAddedEvent {
                    track: t.clone(),
                }));
                self.detect_fx_changes_on_track(
                    &mut td.fx_chain_pair,
                    t,
                    false,
                    true,
                    true,
                    handle_change,
                );
                td
            });
        }
    }

    fn detect_fx_changes_on_track(
        &self,
        fx_chain_pair: &mut FxChainPair,
        track: Track,
        notify_listeners_about_changes: bool,
        check_normal_fx_chain: bool,
        check_input_fx_chain: bool,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        if !track.is_available() {
            return;
        }
        let added_or_removed_output_fx = if check_normal_fx_chain {
            self.detect_fx_changes_on_track_internal(
                &track,
                &mut fx_chain_pair.output_fx_guids,
                false,
                notify_listeners_about_changes,
                handle_change,
            )
        } else {
            false
        };
        let added_or_removed_input_fx = if check_input_fx_chain {
            self.detect_fx_changes_on_track_internal(
                &track,
                &mut fx_chain_pair.input_fx_guids,
                true,
                notify_listeners_about_changes,
                handle_change,
            )
        } else {
            false
        };
        if notify_listeners_about_changes
            && !added_or_removed_input_fx
            && !added_or_removed_output_fx
        {
            handle_change(ChangeEvent::FxReordered(FxReorderedEvent { track }));
        }
    }

    // Returns true if FX was added or removed
    fn detect_fx_changes_on_track_internal(
        &self,
        track: &Track,
        old_fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
        handle_change: impl FnMut(ChangeEvent) + Copy,
    ) -> bool {
        let old_fx_count = old_fx_guids.len() as u32;
        let fx_chain = if is_input_fx {
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        let new_fx_count = fx_chain.fx_count();
        use std::cmp::Ordering::*;
        match new_fx_count.cmp(&old_fx_count) {
            Less => {
                self.remove_invalid_fx(
                    track,
                    old_fx_guids,
                    is_input_fx,
                    notify_listeners_about_changes,
                    handle_change,
                );
                true
            }
            Equal => {
                // Reordering (or nothing)
                false
            }
            Greater => {
                self.add_missing_fx(
                    track,
                    old_fx_guids,
                    is_input_fx,
                    notify_listeners_about_changes,
                    handle_change,
                );
                true
            }
        }
    }

    fn remove_invalid_fx(
        &self,
        track: &Track,
        old_fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        let new_fx_guids = self.fx_guids_on_track(track, is_input_fx);
        old_fx_guids.retain(|old_fx_guid| {
            if new_fx_guids.contains(old_fx_guid) {
                true
            } else {
                if notify_listeners_about_changes {
                    let fx_chain = if is_input_fx {
                        track.input_fx_chain()
                    } else {
                        track.normal_fx_chain()
                    };
                    let removed_fx = fx_chain.fx_by_guid(old_fx_guid);
                    handle_change(ChangeEvent::FxRemoved(FxRemovedEvent { fx: removed_fx }));
                }
                false
            }
        });
    }

    fn fx_guids_on_track(&self, track: &Track, is_input_fx: bool) -> HashSet<Guid> {
        let fx_chain = if is_input_fx {
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        fx_chain
            .fxs()
            .map(|fx| fx.guid().expect("No FX GUID set"))
            .collect()
    }

    fn add_missing_fx(
        &self,
        track: &Track,
        fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        let fx_chain = if is_input_fx {
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        for fx in fx_chain.fxs() {
            let was_inserted = fx_guids.insert(fx.guid().expect("No FX GUID set"));
            if was_inserted && notify_listeners_about_changes {
                handle_change(ChangeEvent::FxAdded(FxAddedEvent { fx }));
            }
        }
    }

    fn update_media_track_positions_and_route_counts(
        &self,
        project: Project,
        track_datas: &mut TrackDataMap,
        mut handle_change: impl FnMut(ChangeEvent) + Copy,
    ) {
        let mut tracks_have_been_reordered = false;
        for (media_track, track_data) in track_datas.iter_mut() {
            let reaper = Reaper::get().medium_reaper();
            if !reaper.validate_ptr_2(Proj(project.raw()), *media_track) {
                continue;
            }
            // Handle reordering
            let new_number =
                unsafe { reaper.get_set_media_track_info_get_track_number(*media_track) };
            if new_number != track_data.number {
                tracks_have_been_reordered = true;
                track_data.number = new_number;
            }
            // Handle route counts
            let new_output_send_count = unsafe {
                reaper.get_track_num_sends(*media_track, TrackSendCategory::HardwareOutput)
            };
            let new_track_send_count =
                unsafe { reaper.get_track_num_sends(*media_track, TrackSendCategory::Send) };
            let new_receive_count =
                unsafe { reaper.get_track_num_sends(*media_track, TrackSendCategory::Receive) };
            if new_output_send_count != track_data.output_send_count
                || new_track_send_count != track_data.track_send_count
                || new_receive_count != track_data.receive_count
            {
                // TODO-high Use lightweight tracks so that creating them is essentially a no-op!
                let track = Track::new(*media_track, None);
                if new_output_send_count != track_data.output_send_count {
                    handle_change(ChangeEvent::HardwareOutputSendCountChanged(
                        HardwareOutputSendCountChangedEvent {
                            track: track.clone(),
                            old: track_data.output_send_count,
                            new: new_output_send_count,
                        },
                    ));
                    track_data.output_send_count = new_output_send_count;
                }
                if new_track_send_count != track_data.track_send_count {
                    handle_change(ChangeEvent::TrackSendCountChanged(
                        TrackSendCountChangedEvent {
                            track: track.clone(),
                            old: track_data.track_send_count,
                            new: new_track_send_count,
                        },
                    ));
                    track_data.track_send_count = new_track_send_count;
                }
                if new_receive_count != track_data.receive_count {
                    handle_change(ChangeEvent::ReceiveCountChanged(ReceiveCountChangedEvent {
                        track: track.clone(),
                        old: track_data.receive_count,
                        new: new_receive_count,
                    }));
                    track_data.receive_count = new_receive_count;
                }
            }
        }
        if tracks_have_been_reordered {
            handle_change(ChangeEvent::TracksReordered(TracksReorderedEvent {
                project,
            }));
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChangeEvent {
    ProjectSwitched(ProjectSwitchedEvent),
    TrackVolumeChanged(TrackVolumeChangedEvent),
    TrackPanChanged(TrackPanChangedEvent),
    TrackRouteVolumeChanged(TrackRouteVolumeChangedEvent),
    TrackRoutePanChanged(TrackRoutePanChangedEvent),
    TrackAdded(TrackAddedEvent),
    TrackRemoved(TrackRemovedEvent),
    TracksReordered(TracksReorderedEvent),
    ReceiveCountChanged(ReceiveCountChangedEvent),
    HardwareOutputSendCountChanged(HardwareOutputSendCountChangedEvent),
    TrackSendCountChanged(TrackSendCountChangedEvent),
    TrackNameChanged(TrackNameChangedEvent),
    TrackInputChanged(TrackInputChangedEvent),
    TrackInputMonitoringChanged(TrackInputMonitoringChangedEvent),
    TrackArmChanged(TrackArmChangedEvent),
    TrackMuteChanged(TrackMuteChangedEvent),
    TrackSoloChanged(TrackSoloChangedEvent),
    TrackSelectedChanged(TrackSelectedChangedEvent),
    TrackAutomationModeChanged(TrackAutomationModeChangedEvent),
    FxAdded(FxAddedEvent),
    FxRemoved(FxRemovedEvent),
    FxEnabledChanged(FxEnabledChangedEvent),
    FxOpened(FxOpenedEvent),
    FxClosed(FxClosedEvent),
    FxFocused(FxFocusedEvent),
    FxReordered(FxReorderedEvent),
    FxParameterValueChanged(FxParameterValueChangedEvent),
    FxPresetChanged(FxPresetChangedEvent),
    MasterTempoChanged(MasterTempoChangedEvent),
    MasterPlayrateChanged(MasterPlayrateChangedEvent),
    GlobalAutomationOverrideChanged(GlobalAutomationOverrideChangedEvent),
    PlayStateChanged(PlayStateChangedEvent),
    RepeatStateChanged(RepeatStateChangedEvent),
    ProjectClosed(ProjectClosedEvent),
    BookmarksChanged(BookmarksChangedEvent),
}

#[derive(Clone, Debug)]
pub struct ProjectSwitchedEvent {
    pub old_project: Project,
    pub new_project: Project,
}

#[derive(Clone, Debug)]
pub struct TrackVolumeChangedEvent {
    pub touched: bool,
    pub track: Track,
    pub old_value: ReaperVolumeValue,
    pub new_value: ReaperVolumeValue,
}

#[derive(Clone, Debug)]
pub struct TrackPanChangedEvent {
    pub touched: bool,
    pub track: Track,
    pub old_value: Pan,
    pub new_value: AvailablePanValue,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AvailablePanValue {
    Complete(Pan),
    Incomplete(ReaperPanValue),
}

#[derive(Clone, Debug)]
pub struct TrackRouteVolumeChangedEvent {
    pub touched: bool,
    pub route: TrackRoute,
    pub old_value: Option<ReaperVolumeValue>,
    pub new_value: ReaperVolumeValue,
}

#[derive(Clone, Debug)]
pub struct TrackRoutePanChangedEvent {
    pub touched: bool,
    pub route: TrackRoute,
    pub old_value: Option<ReaperPanValue>,
    pub new_value: ReaperPanValue,
}

#[derive(Clone, Debug)]
pub struct TrackAddedEvent {
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct TrackRemovedEvent {
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct HardwareOutputSendCountChangedEvent {
    pub track: Track,
    pub old: u32,
    pub new: u32,
}

#[derive(Clone, Debug)]
pub struct TrackSendCountChangedEvent {
    pub track: Track,
    pub old: u32,
    pub new: u32,
}

#[derive(Clone, Debug)]
pub struct ReceiveCountChangedEvent {
    pub track: Track,
    pub old: u32,
    pub new: u32,
}

#[derive(Clone, Debug)]
pub struct TracksReorderedEvent {
    pub project: Project,
}

#[derive(Clone, Debug)]
pub struct TrackNameChangedEvent {
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct TrackInputChangedEvent {
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct TrackInputMonitoringChangedEvent {
    pub track: Track,
    pub old_value: InputMonitoringMode,
    pub new_value: InputMonitoringMode,
}

#[derive(Clone, Debug)]
pub struct TrackArmChangedEvent {
    pub track: Track,
    pub old_value: bool,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct TrackMuteChangedEvent {
    pub touched: bool,
    pub track: Track,
    pub old_value: bool,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct TrackSoloChangedEvent {
    pub track: Track,
    pub old_value: bool,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct TrackSelectedChangedEvent {
    pub track: Track,
    pub old_value: bool,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct TrackAutomationModeChangedEvent {
    pub track: Track,
    pub old_value: AutomationMode,
    pub new_value: AutomationMode,
}

#[derive(Clone, Debug)]
pub struct FxAddedEvent {
    pub fx: Fx,
}

#[derive(Clone, Debug)]
pub struct FxRemovedEvent {
    pub fx: Fx,
}

#[derive(Clone, Debug)]
pub struct FxEnabledChangedEvent {
    pub fx: Fx,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct FxOpenedEvent {
    pub fx: Fx,
}

#[derive(Clone, Debug)]
pub struct FxClosedEvent {
    pub fx: Fx,
}

#[derive(Clone, Debug)]
pub struct FxFocusedEvent {
    pub fx: Option<Fx>,
}

#[derive(Clone, Debug)]
pub struct FxReorderedEvent {
    pub track: Track,
}

#[derive(Clone, Debug)]
pub struct FxParameterValueChangedEvent {
    pub touched: bool,
    pub parameter: FxParameter,
    pub new_value: ReaperNormalizedFxParamValue,
}

#[derive(Clone, Debug)]
pub struct FxPresetChangedEvent {
    pub fx: Fx,
}

#[derive(Clone, Debug)]
pub struct MasterTempoChangedEvent {
    pub project: Project,
    pub touched: bool,
    pub new_value: Bpm,
}

#[derive(Clone, Debug)]
pub struct MasterPlayrateChangedEvent {
    pub project: Project,
    pub touched: bool,
    pub new_value: PlaybackSpeedFactor,
}

#[derive(Clone, Debug)]
pub struct GlobalAutomationOverrideChangedEvent {
    pub project: Project,
    pub old_value: Option<GlobalAutomationModeOverride>,
    pub new_value: Option<GlobalAutomationModeOverride>,
}

#[derive(Clone, Debug)]
pub struct PlayStateChangedEvent {
    pub project: Project,
    pub new_value: PlayState,
}

#[derive(Clone, Debug)]
pub struct RepeatStateChangedEvent {
    pub project: Project,
    pub new_value: bool,
}

#[derive(Clone, Debug)]
pub struct ProjectClosedEvent {
    pub project: Project,
}

#[derive(Clone, Debug)]
pub struct BookmarksChangedEvent {
    pub project: Project,
}
