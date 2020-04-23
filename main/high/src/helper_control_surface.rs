use crate::fx::Fx;
use crate::guid::Guid;
use crate::{get_media_track_guid, Payload, Project, Reaper, ScheduledTask, Task, Track};
use c_str_macro::c_str;
use reaper_rs_low::raw;
use reaper_rs_medium::TrackInfoKey::{
    B_MUTE, D_PAN, D_VOL, IP_TRACKNUMBER, I_RECARM, I_RECINPUT, I_RECMON, I_SELECTED, I_SOLO,
};
use reaper_rs_medium::{
    AutomationMode, ControlSurface, ExtSetBpmAndPlayRateArgs, ExtSetFocusedFxArgs,
    ExtSetFxChangeArgs, ExtSetFxEnabledArgs, ExtSetFxOpenArgs, ExtSetFxParamArgs,
    ExtSetInputMonitorArgs, ExtSetLastTouchedFxArgs, ExtSetSendPanArgs, ExtSetSendVolumeArgs,
    FxChainType, InputMonitoringMode, MediaTrack, QualifiedFxRef, ReaProject, ReaperPointer,
    SetSurfaceMuteArgs, SetSurfacePanArgs, SetSurfaceRecArmArgs, SetSurfaceSelectedArgs,
    SetSurfaceSoloArgs, SetSurfaceVolumeArgs, SetTrackTitleArgs, TrackRef, VersionDependentFxRef,
    VersionDependentTrackFxRef,
};
use rxrust::prelude::*;

use std::cell::{Cell, RefCell, RefMut};

use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::mpsc::{Receiver, Sender};

const BULK_TASK_EXECUTION_COUNT: usize = 100;

pub(super) struct HelperControlSurface {
    task_sender: Sender<ScheduledTask>,
    task_receiver: Receiver<ScheduledTask>,
    last_active_project: Cell<Project>,
    num_track_set_changes_left_to_be_propagated: Cell<u32>,
    fx_has_been_touched_just_a_moment_ago: Cell<bool>,
    project_datas: RefCell<ProjectDataMap>,
    fx_chain_pair_by_media_track: RefCell<HashMap<MediaTrack, FxChainPair>>,

    // Capabilities depending on REAPER version
    supports_detection_of_input_fx: bool,
    supports_detection_of_input_fx_in_set_fx_change: bool,
}

#[derive(PartialEq)]
enum State {
    Normal,
    PropagatingTrackSetChanges,
}

struct TrackData {
    volume: f64,
    pan: f64,
    selected: bool,
    mute: bool,
    solo: bool,
    recarm: bool,
    number: Option<TrackRef>,
    recmonitor: InputMonitoringMode,
    recinput: i32,
    guid: Guid,
}

#[derive(Default)]
struct FxChainPair {
    input_fx_guids: HashSet<Guid>,
    output_fx_guids: HashSet<Guid>,
}

type ProjectDataMap = HashMap<ReaProject, TrackDataMap>;
type TrackDataMap = HashMap<MediaTrack, TrackData>;

impl HelperControlSurface {
    pub(super) fn new(
        task_sender: Sender<ScheduledTask>,
        task_receiver: Receiver<ScheduledTask>,
    ) -> HelperControlSurface {
        let reaper = Reaper::get();
        let version = reaper.get_version();
        let surface = HelperControlSurface {
            task_sender,
            task_receiver,
            last_active_project: Cell::new(reaper.get_current_project()),
            num_track_set_changes_left_to_be_propagated: Default::default(),
            fx_has_been_touched_just_a_moment_ago: Default::default(),
            project_datas: Default::default(),
            fx_chain_pair_by_media_track: Default::default(),
            // since pre1,
            supports_detection_of_input_fx: version >= c_str!("5.95").into(),
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: version >= c_str!("5.95").into(),
        };
        // REAPER doesn't seem to call this automatically when the surface is registered. In our
        // case it's important to call this not at the first change of something (e.g. arm
        // button pressed) but immediately. Because it captures the initial project/track/FX
        // state. If we don't do this immediately, then it happens that change events (e.g.
        // track arm changed) are not reported because the initial state was unknown.
        // TODO-low This executes a bunch of REAPER functions right on start. Maybe do more lazily
        // on activate?  But before activate we can do almost nothing because
        // execute_on_main_thread doesn't work.
        surface.set_track_list_change();
        surface
    }

    fn get_state(&self) -> State {
        if self.num_track_set_changes_left_to_be_propagated.get() == 0 {
            State::Normal
        } else {
            State::PropagatingTrackSetChanges
        }
    }

    fn find_track_data_in_normal_state<'a>(&self, track: MediaTrack) -> Option<RefMut<TrackData>> {
        if self.get_state() == State::PropagatingTrackSetChanges {
            return None;
        }
        self.find_track_data(track)
    }

    fn find_track_data_map(&self) -> Option<RefMut<TrackDataMap>> {
        let rea_project = Reaper::get().get_current_project().get_raw();
        if !self.project_datas.borrow().contains_key(&rea_project) {
            return None;
        }
        Some(RefMut::map(self.project_datas.borrow_mut(), |tds| {
            tds.get_mut(&rea_project).unwrap()
        }))
    }

    fn find_track_data<'a>(&self, track: MediaTrack) -> Option<RefMut<TrackData>> {
        let track_data_map = self.find_track_data_map()?;
        if !track_data_map.contains_key(&track) {
            return None;
        }
        Some(RefMut::map(track_data_map, |tdm| {
            tdm.get_mut(&track).unwrap()
        }))
    }

    fn track_parameter_is_automated(&self, track: &Track, parameter_name: &CStr) -> bool {
        if !track.is_available() {
            return false;
        }
        let env = unsafe {
            Reaper::get()
                .medium
                .get_track_envelope_by_name(track.get_raw(), parameter_name)
        };
        if env.is_none() {
            return false;
        }
        use AutomationMode::*;
        match track.get_effective_automation_mode() {
            // Is not automated
            None | Some(TrimRead) | Some(Write) => false,
            // Is automated
            _ => true,
        }
    }

    fn remove_invalid_rea_projects(&self) {
        self.project_datas.borrow_mut().retain(|rea_project, _| {
            if Reaper::get().medium.validate_ptr_2(None, *rea_project) {
                true
            } else {
                Reaper::get()
                    .subjects
                    .project_closed
                    .borrow_mut()
                    .next(Project::new(*rea_project));
                false
            }
        });
    }

    fn detect_track_set_changes(&self) {
        let project = Reaper::get().get_current_project();
        let mut project_datas = self.project_datas.borrow_mut();
        let track_datas = project_datas.entry(project.get_raw()).or_default();
        let old_track_count = track_datas.len() as u32;
        let new_track_count = project.get_track_count();
        if new_track_count < old_track_count {
            self.remove_invalid_media_tracks(project, track_datas);
        } else if new_track_count > old_track_count {
            self.add_missing_media_tracks(project, track_datas);
        } else {
            self.update_media_track_positions(project, track_datas);
        }
    }

    fn add_missing_media_tracks(&self, project: Project, track_datas: &mut TrackDataMap) {
        for t in project.get_tracks() {
            let media_track = t.get_raw();
            track_datas.entry(media_track).or_insert_with(|| {
                let reaper = Reaper::get();
                let m = &reaper.medium;
                let td = unsafe {
                    TrackData {
                        volume: m.get_media_track_info_value(media_track, D_VOL),
                        pan: m.get_media_track_info_value(media_track, D_PAN),
                        selected: m.get_media_track_info_value(media_track, I_SELECTED) != 0.0,
                        mute: m.get_media_track_info_value(media_track, B_MUTE) != 0.0,
                        solo: m.get_media_track_info_value(media_track, I_SOLO) != 0.0,
                        recarm: m.get_media_track_info_value(media_track, I_RECARM) != 0.0,
                        number: m.get_media_track_info_tracknumber(media_track),
                        recmonitor: m.get_media_track_info_recmon(media_track),
                        recinput: m.get_media_track_info_value(media_track, I_RECINPUT) as i32,
                        guid: get_media_track_guid(media_track),
                    }
                };
                reaper.subjects.track_added.borrow_mut().next(t.clone());
                self.detect_fx_changes_on_track(t, false, true, true);
                td
            });
        }
    }

    fn detect_fx_changes_on_track(
        &self,
        track: Track,
        notify_listeners_about_changes: bool,
        check_normal_fx_chain: bool,
        check_input_fx_chain: bool,
    ) {
        if !track.is_available() {
            return;
        }
        let mut fx_chain_pairs = self.fx_chain_pair_by_media_track.borrow_mut();
        let fx_chain_pair = fx_chain_pairs.entry(track.get_raw()).or_default();
        let added_or_removed_output_fx = if check_normal_fx_chain {
            self.detect_fx_changes_on_track_internal(
                &track,
                &mut fx_chain_pair.output_fx_guids,
                false,
                notify_listeners_about_changes,
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
            )
        } else {
            false
        };
        if notify_listeners_about_changes
            && !added_or_removed_input_fx
            && !added_or_removed_output_fx
        {
            Reaper::get()
                .subjects
                .fx_reordered
                .borrow_mut()
                .next(track.into());
        }
    }

    // Returns true if FX was added or removed
    fn detect_fx_changes_on_track_internal(
        &self,
        track: &Track,
        old_fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
    ) -> bool {
        let old_fx_count = old_fx_guids.len() as u32;
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        let new_fx_count = fx_chain.get_fx_count();
        if new_fx_count < old_fx_count {
            self.remove_invalid_fx(
                track,
                old_fx_guids,
                is_input_fx,
                notify_listeners_about_changes,
            );
            true
        } else if new_fx_count > old_fx_count {
            self.add_missing_fx(
                track,
                old_fx_guids,
                is_input_fx,
                notify_listeners_about_changes,
            );
            true
        } else {
            // Reordering (or nothing)
            false
        }
    }

    fn remove_invalid_fx(
        &self,
        track: &Track,
        old_fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
    ) {
        let new_fx_guids = self.get_fx_guids_on_track(track, is_input_fx);
        old_fx_guids.retain(|old_fx_guid| {
            if new_fx_guids.contains(old_fx_guid) {
                true
            } else {
                if notify_listeners_about_changes {
                    let fx_chain = if is_input_fx {
                        track.get_input_fx_chain()
                    } else {
                        track.get_normal_fx_chain()
                    };
                    let removed_fx = fx_chain.get_fx_by_guid(old_fx_guid);
                    Reaper::get()
                        .subjects
                        .fx_removed
                        .borrow_mut()
                        .next(removed_fx.into());
                }
                false
            }
        });
    }

    fn add_missing_fx(
        &self,
        track: &Track,
        fx_guids: &mut HashSet<Guid>,
        is_input_fx: bool,
        notify_listeners_about_changes: bool,
    ) {
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        for fx in fx_chain.get_fxs() {
            let was_inserted = fx_guids.insert(fx.get_guid().expect("No FX GUID set"));
            if was_inserted && notify_listeners_about_changes {
                Reaper::get().subjects.fx_added.borrow_mut().next(fx.into());
            }
        }
    }

    fn get_fx_guids_on_track(&self, track: &Track, is_input_fx: bool) -> HashSet<Guid> {
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        fx_chain
            .get_fxs()
            .map(|fx| fx.get_guid().expect("No FX GUID set"))
            .collect()
    }

    fn remove_invalid_media_tracks(&self, project: Project, track_datas: &mut TrackDataMap) {
        track_datas.retain(|media_track, data| {
            let reaper = Reaper::get();
            if reaper
                .medium
                .validate_ptr_2(Some(project.get_raw()), *media_track)
            {
                true
            } else {
                self.fx_chain_pair_by_media_track
                    .borrow_mut()
                    .remove(media_track);
                let track = project.get_track_by_guid(&data.guid);
                reaper
                    .subjects
                    .track_removed
                    .borrow_mut()
                    .next(track.into());
                false
            }
        });
    }

    fn update_media_track_positions(&self, project: Project, track_datas: &mut TrackDataMap) {
        let mut tracks_have_been_reordered = false;
        let reaper = Reaper::get();
        for (media_track, track_data) in track_datas.iter_mut() {
            if !reaper
                .medium
                .validate_ptr_2(Some(project.get_raw()), *media_track)
            {
                continue;
            }
            let new_number =
                unsafe { reaper.medium.get_media_track_info_tracknumber(*media_track) };
            if new_number != track_data.number {
                tracks_have_been_reordered = true;
                track_data.number = new_number;
            }
        }
        if tracks_have_been_reordered {
            reaper.subjects.tracks_reordered.borrow_mut().next(project);
        }
    }

    fn fx_param_set(&self, args: ExtSetFxParamArgs, is_input_fx_if_supported: bool) {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        let is_input_fx = if self.supports_detection_of_input_fx {
            is_input_fx_if_supported
        } else {
            self.is_probably_input_fx(
                &track,
                args.fx_index,
                Some(args.param_index),
                Some(args.normalized_value),
            )
        };
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        if let Some(fx) = fx_chain.get_fx_by_index(args.fx_index as u32) {
            let fx_param = fx.get_parameter_by_index(args.param_index as u32);
            let reaper = Reaper::get();
            reaper
                .subjects
                .fx_parameter_value_changed
                .borrow_mut()
                .next(fx_param.clone());
            if self.fx_has_been_touched_just_a_moment_ago.get() {
                self.fx_has_been_touched_just_a_moment_ago.replace(false);
                reaper
                    .subjects
                    .fx_parameter_touched
                    .borrow_mut()
                    .next(fx_param);
            }
        }
    }

    fn is_probably_input_fx(
        &self,
        track: &Track,
        fx_index: u32,
        param_index: Option<u32>,
        normalized_value: Option<f64>,
    ) -> bool {
        let pairs = self.fx_chain_pair_by_media_track.borrow();
        let pair = match pairs.get(&track.get_raw()) {
            None => {
                // Should not happen. In this case, an FX yet unknown to Realearn has sent a
                // parameter change
                return false;
            }
            Some(pair) => pair,
        };
        let could_be_input_fx = (fx_index as usize) < pair.input_fx_guids.len();
        let could_be_output_fx = (fx_index as usize) < pair.output_fx_guids.len();
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
            match track.get_normal_fx_chain().get_fx_by_index(fx_index) {
                None => true,
                Some(output_fx) => {
                    let output_fx_param = output_fx.get_parameter_by_index(param_index);
                    let is_probably_output_fx =
                        Some(output_fx_param.get_reaper_value()) == normalized_value;
                    !is_probably_output_fx
                }
            }
        }
    }

    // From REAPER > 5.95, parmFxIndex should be interpreted as query index. For earlier versions
    // it's a normal index
    // - which unfortunately doesn't contain information if the FX is on the normal FX chain or the
    //   input FX chain.
    // In this case a heuristic is applied to determine which chain it is. It gets more accurate
    // when paramIndex and paramValue are supplied.
    fn get_fx_from_parm_fx_index(
        &self,
        track: &Track,
        parm_fx_index: VersionDependentTrackFxRef,
        param_index: Option<u32>,
        param_value: Option<f64>,
    ) -> Option<Fx> {
        use VersionDependentTrackFxRef::*;
        match parm_fx_index {
            Old(index) => {
                let is_input_fx = self.is_probably_input_fx(track, index, param_index, param_value);
                let fx_chain = if is_input_fx {
                    track.get_input_fx_chain()
                } else {
                    track.get_normal_fx_chain()
                };
                fx_chain.get_fx_by_index(index)
            }
            New(fx_ref) => track.get_fx_by_query_index(fx_ref.into()),
        }
    }

    fn decrease_num_track_set_changes_left_to_be_propagated(&self) {
        let previous_value = self.num_track_set_changes_left_to_be_propagated.get();
        self.num_track_set_changes_left_to_be_propagated
            .replace(previous_value - 1);
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&mut self) {
        // Invoke custom idle code
        Reaper::get()
            .subjects
            .main_thread_idle
            .borrow_mut()
            .next(true);
        // Process tasks in queue
        for task in self
            .task_receiver
            .try_iter()
            .take(BULK_TASK_EXECUTION_COUNT)
        {
            match task.desired_execution_time {
                None => (task.task)(),
                Some(t) => {
                    if std::time::SystemTime::now() < t {
                        self.task_sender.send(task);
                    } else {
                        (task.task)()
                    }
                }
            }
        }
    }

    fn set_track_list_change(&self) {
        // TODO-low Not multi-project compatible!
        let reaper = Reaper::get();
        let new_active_project = reaper.get_current_project();
        if new_active_project != self.last_active_project.get() {
            self.last_active_project.replace(new_active_project);
            reaper
                .subjects
                .project_switched
                .borrow_mut()
                .next(new_active_project);
        }
        self.num_track_set_changes_left_to_be_propagated
            .replace(new_active_project.get_track_count() + 1);
        self.remove_invalid_rea_projects();
        self.detect_track_set_changes();
    }

    fn set_surface_pan(&self, args: SetSurfacePanArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.pan == args.pan {
            return;
        }
        td.pan = args.pan;
        let track = Track::new(args.trackid, None);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_pan_changed
            .borrow_mut()
            .next(track.clone());
        if !self.track_parameter_is_automated(&track, c_str!("Pan")) {
            reaper.subjects.track_pan_touched.borrow_mut().next(track);
        }
    }

    fn set_surface_volume(&self, args: SetSurfaceVolumeArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.volume == args.volume {
            return;
        }
        td.volume = args.volume;
        let track = Track::new(args.trackid, None);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_volume_changed
            .borrow_mut()
            .next(track.clone());
        if !self.track_parameter_is_automated(&track, c_str!("Volume")) {
            reaper
                .subjects
                .track_volume_touched
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_mute(&self, args: SetSurfaceMuteArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.mute != args.mute {
            td.mute = args.mute;
            let track = Track::new(args.trackid, None);
            let reaper = Reaper::get();
            reaper
                .subjects
                .track_mute_changed
                .borrow_mut()
                .next(track.clone());
            if !self.track_parameter_is_automated(&track, c_str!("Mute")) {
                reaper.subjects.track_mute_touched.borrow_mut().next(track);
            }
        }
    }

    fn set_surface_selected(&self, args: SetSurfaceSelectedArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.selected != args.selected {
            td.selected = args.selected;
            let track = Track::new(args.trackid, None);
            Reaper::get()
                .subjects
                .track_selected_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_solo(&self, args: SetSurfaceSoloArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.solo != args.solo {
            td.solo = args.solo;
            let track = Track::new(args.trackid, None);
            Reaper::get()
                .subjects
                .track_solo_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_rec_arm(&self, args: SetSurfaceRecArmArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.trackid) {
            None => return,
            Some(td) => td,
        };
        if td.recarm != args.recarm {
            td.recarm = args.recarm;
            let track = Track::new(args.trackid, None);
            Reaper::get()
                .subjects
                .track_arm_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_track_title(&self, args: SetTrackTitleArgs) {
        if self.get_state() == State::PropagatingTrackSetChanges {
            self.decrease_num_track_set_changes_left_to_be_propagated();
            return;
        }
        let track = Track::new(args.trackid, None);
        Reaper::get()
            .subjects
            .track_name_changed
            .borrow_mut()
            .next(track);
    }

    fn ext_setinputmonitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return 1,
            Some(td) => td,
        };
        let reaper = Reaper::get();
        if td.recmonitor != args.recmonitor {
            td.recmonitor = args.recmonitor;
            reaper
                .subjects
                .track_input_monitoring_changed
                .borrow_mut()
                .next(Track::new(args.track, None));
        }
        let recinput = unsafe {
            reaper
                .medium
                .get_media_track_info_value(args.track, I_RECINPUT) as i32
        };
        if td.recinput != recinput {
            td.recinput = recinput;
            reaper
                .subjects
                .track_input_changed
                .borrow_mut()
                .next(Track::new(args.track, None));
        }
        1
    }

    fn ext_setfxparam(&self, args: ExtSetFxParamArgs) -> i32 {
        self.fx_param_set(args, false);
        1
    }

    fn ext_setfxparam_recfx(&self, args: ExtSetFxParamArgs) -> i32 {
        self.fx_param_set(args, true);
        1
    }

    fn ext_setfxenabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        if let Some(fx) = self.get_fx_from_parm_fx_index(&track, args.fxidx, None, None) {
            Reaper::get()
                .subjects
                .fx_enabled_changed
                .borrow_mut()
                .next(fx);
        }
        1
    }

    fn ext_setsendvolume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        let track = Track::new(args.track, None);
        let track_send = track.get_index_based_send_by_index(args.sendidx);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_send_volume_changed
            .borrow_mut()
            .next(track_send.clone());
        // Send volume touch event only if not automated
        if !self.track_parameter_is_automated(&track, c_str!("Send Volume")) {
            reaper
                .subjects
                .track_send_volume_touched
                .borrow_mut()
                .next(track_send);
        }
        1
    }

    fn ext_setsendpan(&self, args: ExtSetSendPanArgs) -> i32 {
        let track = Track::new(args.track, None);
        let track_send = track.get_index_based_send_by_index(args.sendidx);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_send_pan_changed
            .borrow_mut()
            .next(track_send.clone());
        // Send volume touch event only if not automated
        if !self.track_parameter_is_automated(&track, c_str!("Send Pan")) {
            reaper
                .subjects
                .track_send_pan_touched
                .borrow_mut()
                .next(track_send);
        }
        1
    }

    fn ext_setfocusedfx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        let reaper = Reaper::get();
        let fx_ref = match args.fx_ref {
            None => {
                // Clear focused FX
                reaper.subjects.fx_focused.borrow_mut().next(Payload(None));
                return 0;
            }
            Some(r) => r,
        };
        use VersionDependentFxRef::*;
        match fx_ref.fx_ref {
            ItemFx { .. } => {
                // TODO Not handled right now
                0
            }
            TrackFx(track_fx_ref) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(fx_ref.track, None);
                if let Some(fx) = self.get_fx_from_parm_fx_index(&track, track_fx_ref, None, None) {
                    // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2
                    // and on chunk manipulations
                    self.detect_fx_changes_on_track(
                        track,
                        true,
                        !fx.is_input_fx(),
                        fx.is_input_fx(),
                    );
                    reaper
                        .subjects
                        .fx_focused
                        .borrow_mut()
                        .next(Payload(Some(fx)));
                }
                1
            }
        }
    }

    fn ext_setfxopen(&self, args: ExtSetFxOpenArgs) -> i32 {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        if let Some(fx) = self.get_fx_from_parm_fx_index(&track, args.fxidx, None, None) {
            // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2 and on
            // chunk manipulations
            self.detect_fx_changes_on_track(track, true, !fx.is_input_fx(), fx.is_input_fx());
            let reaper = Reaper::get();
            let subject = if args.ui_open {
                &reaper.subjects.fx_opened
            } else {
                &reaper.subjects.fx_closed
            };
            subject.borrow_mut().next(fx);
        }
        1
    }

    fn ext_setfxchange(&self, args: ExtSetFxChangeArgs) -> i32 {
        let track = Track::new(args.track, None);
        match args.fx_chain_type {
            Some(t) => {
                let is_input_fx = t == FxChainType::Input;
                self.detect_fx_changes_on_track(track, true, !is_input_fx, is_input_fx);
            }
            None => {
                self.detect_fx_changes_on_track(track, true, true, true);
            }
        }
        1
    }

    fn ext_setlasttouchedfx(&self, _: ExtSetLastTouchedFxArgs) -> i32 {
        self.fx_has_been_touched_just_a_moment_ago.replace(true);
        1
    }

    fn ext_setbpmandplayrate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        let reaper = Reaper::get();
        if args.bpm.is_some() {
            reaper.subjects.master_tempo_changed.borrow_mut().next(());
            // If there's a tempo envelope, there are just tempo notifications when the tempo is
            // actually changed. So that's okay for "touched".
            // TODO-low What about gradual tempo changes?
            reaper.subjects.master_tempo_touched.borrow_mut().next(());
        }
        if args.playrate.is_some() {
            reaper
                .subjects
                .master_playrate_changed
                .borrow_mut()
                .next(true);
            // FIXME What about playrate automation?
            reaper
                .subjects
                .master_playrate_touched
                .borrow_mut()
                .next(true);
        }
        1
    }
}
