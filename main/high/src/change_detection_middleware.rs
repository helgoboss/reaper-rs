use crate::{
    get_media_track_guid, ControlSurfaceEvent, Fx, FxParameter, Guid, Project, Reaper, Track,
    TrackSend,
};
use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::TrackAttributeKey::RecInput;
use reaper_medium::{
    reaper_str, AutomationMode, ExtSetFxParamArgs, InputMonitoringMode, MediaTrack, ReaProject,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperStr, ReaperVersion, ReaperVolumeValue,
    TrackAttributeKey, TrackFxChainType, TrackLocation, VersionDependentFxLocation,
    VersionDependentTrackFxLocation,
};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ChangeDetectionMiddleware {
    num_track_set_changes_left_to_be_propagated: Cell<u32>,
    last_active_project: Cell<Project>,
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
    pan: ReaperPanValue,
    selected: bool,
    mute: bool,
    solo: bool,
    recarm: bool,
    number: Option<TrackLocation>,
    recmonitor: InputMonitoringMode,
    recinput: i32,
    guid: Guid,
    send_volumes: HashMap<u32, ReaperVolumeValue>,
    send_pans: HashMap<u32, ReaperPanValue>,
    fx_param_values: HashMap<TrackFxKey, ReaperNormalizedFxParamValue>,
    fx_chain_pair: FxChainPair,
}

impl TrackData {
    /// Returns true if it has changed.
    fn update_send_volume(&mut self, index: u32, v: ReaperVolumeValue) -> bool {
        match self.send_volumes.insert(index, v) {
            None => true,
            Some(prev) => v != prev,
        }
    }

    /// Returns true if it has changed.
    fn update_send_pan(&mut self, index: u32, v: ReaperPanValue) -> bool {
        match self.send_pans.insert(index, v) {
            None => true,
            Some(prev) => v != prev,
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

impl ChangeDetectionMiddleware {
    pub fn new(
        version: ReaperVersion<'static>,
        last_active_project: Project,
    ) -> ChangeDetectionMiddleware {
        let reaper_version_5_95 = ReaperVersion::new("5.95");
        ChangeDetectionMiddleware {
            num_track_set_changes_left_to_be_propagated: Default::default(),
            last_active_project: Cell::new(last_active_project),
            project_datas: Default::default(),
            fx_has_been_touched_just_a_moment_ago: Default::default(),
            // since pre1,
            supports_detection_of_input_fx: version >= reaper_version_5_95,
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: version >= reaper_version_5_95,
        }
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
    ) {
        use ControlSurfaceEvent::*;
        match event {
            SetTrackListChange => self.set_track_list_change(handle_change),
            SetSurfacePan(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.pan == args.pan {
                    return;
                }
                td.pan = args.pan;
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackPanChanged(track.clone()));
                if !self.track_parameter_is_automated(&track, reaper_str!("Pan")) {
                    handle_change(ChangeEvent::TrackPanTouched(track));
                }
            }
            SetSurfaceVolume(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.volume == args.volume {
                    return;
                }
                td.volume = args.volume;
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackVolumeChanged(track.clone()));
                if !self.track_parameter_is_automated(&track, reaper_str!("Volume")) {
                    handle_change(ChangeEvent::TrackVolumeTouched(track));
                }
            }
            SetSurfaceMute(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.mute != args.is_mute {
                    td.mute = args.is_mute;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackMuteChanged(track.clone()));
                    if !self.track_parameter_is_automated(&track, reaper_str!("Mute")) {
                        handle_change(ChangeEvent::TrackMuteTouched(track));
                    }
                }
            }
            SetSurfaceSelected(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.selected != args.is_selected {
                    td.selected = args.is_selected;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackSelectedChanged(track));
                }
            }
            SetSurfaceSolo(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.solo != args.is_solo {
                    td.solo = args.is_solo;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackSoloChanged(track));
                }
            }
            SetSurfaceRecArm(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.recarm != args.is_armed {
                    td.recarm = args.is_armed;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackArmChanged(track));
                }
            }
            SetTrackTitle(args) => {
                if self.state() == State::PropagatingTrackSetChanges {
                    self.decrease_num_track_set_changes_left_to_be_propagated();
                    return;
                }
                let track = Track::new(args.track, None);
                handle_change(ChangeEvent::TrackNameChanged(track));
            }
            ExtSetInputMonitor(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if td.recmonitor != args.mode {
                    td.recmonitor = args.mode;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackInputMonitoringChanged(track));
                }
                let recinput = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .get_media_track_info_value(args.track, RecInput) as i32
                };
                if td.recinput != recinput {
                    td.recinput = recinput;
                    let track = Track::new(args.track, None);
                    handle_change(ChangeEvent::TrackInputChanged(track));
                }
            }
            ExtSetFxParam(args) => self.fx_param_set(args, false, handle_change),
            ExtSetFxParamRecFx(args) => self.fx_param_set(args, true, handle_change),
            ExtSetFxEnabled(args) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(args.track, None);
                if let Some(fx) = self.fx_from_parm_fx_index(&track, args.fx_location, None, None) {
                    handle_change(ChangeEvent::FxEnabledChanged(fx));
                }
            }
            ExtSetSendVolume(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if !td.update_send_volume(args.send_index, args.volume) {
                    return;
                }
                let track = Track::new(args.track, None);
                let track_send = track.index_based_send_by_index(args.send_index);
                handle_change(ChangeEvent::TrackSendVolumeChanged(track_send.clone()));
                // Send volume touch event only if not automated
                if !self.track_parameter_is_automated(&track, reaper_str!("Send Volume")) {
                    handle_change(ChangeEvent::TrackSendVolumeTouched(track_send));
                }
            }
            ExtSetSendPan(args) => {
                let mut td = match self.find_track_data_in_normal_state(args.track) {
                    None => return,
                    Some(td) => td,
                };
                if !td.update_send_pan(args.send_index, args.pan) {
                    return;
                }
                let track = Track::new(args.track, None);
                let track_send = track.index_based_send_by_index(args.send_index);
                handle_change(ChangeEvent::TrackSendPanChanged(track_send.clone()));
                // Send volume touch event only if not automated
                if !self.track_parameter_is_automated(&track, reaper_str!("Send Pan")) {
                    handle_change(ChangeEvent::TrackSendPanTouched(track_send));
                }
            }
            ExtSetFocusedFx(args) => {
                let fx_ref = match args.fx_location {
                    None => {
                        // Clear focused FX
                        handle_change(ChangeEvent::FxFocused(None));
                        return;
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
                                handle_change(ChangeEvent::FxFocused(Some(fx)));
                            }
                        }
                    }
                }
            }
            ExtSetFxOpen(args) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(args.track, None);
                let fx_location = match args.fx_location {
                    None => return,
                    Some(l) => l,
                };
                if let Some(fx) = self.fx_from_parm_fx_index(&track, fx_location, None, None) {
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
                            ChangeEvent::FxOpened(fx)
                        } else {
                            ChangeEvent::FxClosed(fx)
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
                if args.tempo.is_some() {
                    handle_change(ChangeEvent::MasterTempoChanged);
                    // If there's a tempo envelope, there are just tempo notifications when the
                    // tempo is actually changed. So that's okay for "touched".
                    // TODO-low What about gradual tempo changes?
                    handle_change(ChangeEvent::MasterTempoTouched);
                }
                if args.play_rate.is_some() {
                    handle_change(ChangeEvent::MasterPlayrateChanged);
                    // FIXME What about playrate automation?
                    handle_change(ChangeEvent::MasterPlayrateTouched);
                }
            }
            ExtTrackFxPresetChanged(args) => {
                let track = Track::new(args.track, None);
                let fx = track
                    .fx_by_query_index(args.fx_location.to_raw())
                    .expect("preset changed but FX not found");
                handle_change(ChangeEvent::FxPresetChanged(fx));
            }
            SetPlayState(_args) => {
                handle_change(ChangeEvent::PlayStateChanged);
            }
            SetRepeatState(_args) => {
                handle_change(ChangeEvent::RepeatStateChanged);
            }
            _ => {}
        }
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

    fn find_track_data_map(&self) -> Option<RefMut<TrackDataMap>> {
        let rea_project = Reaper::get().current_project().raw();
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
        let track_data_map = self.find_track_data_map()?;
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
                    let is_probably_output_fx = output_fx_param
                        .reaper_normalized_value()
                        .contains(&normalized_value);
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
            let fx_param = fx.parameter_by_index(args.param_index as u32);
            handle_change(ChangeEvent::FxParameterValueChanged(fx_param.clone()));
            if self.fx_has_been_touched_just_a_moment_ago.get() {
                self.fx_has_been_touched_just_a_moment_ago.replace(false);
                handle_change(ChangeEvent::FxParameterTouched(fx_param));
            }
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
            self.last_active_project.replace(new_active_project);
            handle_change(ChangeEvent::ProjectSwitched(new_active_project));
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
                handle_change(ChangeEvent::ProjectClosed(Project::new(*rea_project)));
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
            Equal => self.update_media_track_positions(project, track_datas, handle_change),
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
                handle_change(ChangeEvent::TrackRemoved(track));
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
            let media_track = t.raw();
            track_datas.entry(media_track).or_insert_with(|| {
                let func = Reaper::get().medium_reaper();
                let mut td = unsafe {
                    use TrackAttributeKey::*;
                    TrackData {
                        volume: ReaperVolumeValue::new(
                            func.get_media_track_info_value(media_track, Vol),
                        ),
                        pan: ReaperPanValue::new(func.get_media_track_info_value(media_track, Pan)),
                        selected: func.get_media_track_info_value(media_track, Selected) != 0.0,
                        mute: func.get_media_track_info_value(media_track, Mute) != 0.0,
                        solo: func.get_media_track_info_value(media_track, Solo) != 0.0,
                        recarm: func.get_media_track_info_value(media_track, RecArm) != 0.0,
                        number: func.get_set_media_track_info_get_track_number(media_track),
                        recmonitor: func.get_set_media_track_info_get_rec_mon(media_track),
                        recinput: func.get_media_track_info_value(media_track, RecInput) as i32,
                        guid: get_media_track_guid(media_track),
                        send_volumes: Default::default(),
                        send_pans: Default::default(),
                        fx_param_values: Default::default(),
                        fx_chain_pair: Default::default(),
                    }
                };
                // TODO-low Use try_borrow_mut(). Then this just doesn't do anything if this event
                //  is currently thrown already. Right now it would panic, which is unreasonable.
                handle_change(ChangeEvent::TrackAdded(t.clone()));
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
            handle_change(ChangeEvent::FxReordered(track));
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
                    handle_change(ChangeEvent::FxRemoved(removed_fx));
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
                handle_change(ChangeEvent::FxAdded(fx));
            }
        }
    }

    fn update_media_track_positions(
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
            let new_number =
                unsafe { reaper.get_set_media_track_info_get_track_number(*media_track) };
            if new_number != track_data.number {
                tracks_have_been_reordered = true;
                track_data.number = new_number;
            }
        }
        if tracks_have_been_reordered {
            handle_change(ChangeEvent::TracksReordered(project));
        }
    }
}

pub enum ChangeEvent {
    ProjectSwitched(Project),
    TrackVolumeChanged(Track),
    TrackVolumeTouched(Track),
    TrackPanChanged(Track),
    TrackPanTouched(Track),
    TrackSendVolumeChanged(TrackSend),
    TrackSendVolumeTouched(TrackSend),
    TrackSendPanChanged(TrackSend),
    TrackSendPanTouched(TrackSend),
    TrackAdded(Track),
    TrackRemoved(Track),
    TracksReordered(Project),
    TrackNameChanged(Track),
    TrackInputChanged(Track),
    TrackInputMonitoringChanged(Track),
    TrackArmChanged(Track),
    TrackMuteChanged(Track),
    TrackMuteTouched(Track),
    TrackSoloChanged(Track),
    TrackSelectedChanged(Track),
    FxAdded(Fx),
    FxRemoved(Fx),
    FxEnabledChanged(Fx),
    FxOpened(Fx),
    FxClosed(Fx),
    FxFocused(Option<Fx>),
    FxReordered(Track),
    FxParameterValueChanged(FxParameter),
    FxParameterTouched(FxParameter),
    FxPresetChanged(Fx),
    MasterTempoChanged,
    MasterTempoTouched,
    MasterPlayrateChanged,
    MasterPlayrateTouched,
    PlayStateChanged,
    RepeatStateChanged,
    ProjectClosed(Project),
}
