use crate::fx::Fx;
use crate::guid::Guid;
use crate::{
    get_media_track_guid, MainThreadTask, Project, Reaper, Track, MAIN_THREAD_TASK_BULK_SIZE,
};


use reaper_medium::TrackAttributeKey::{Mute, Pan, RecArm, RecInput, Selected, Solo, Vol};
use reaper_medium::{
    reaper_str, AutomationMode, ControlSurface, ExtSetBpmAndPlayRateArgs, ExtSetFocusedFxArgs,
    ExtSetFxChangeArgs, ExtSetFxEnabledArgs, ExtSetFxOpenArgs, ExtSetFxParamArgs,
    ExtSetInputMonitorArgs, ExtSetLastTouchedFxArgs, ExtSetSendPanArgs, ExtSetSendVolumeArgs,
    ExtTrackFxPresetChangedArgs, InputMonitoringMode, MediaTrack, ReaProject,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperStr, ReaperVersion, ReaperVolumeValue,
    SetSurfaceMuteArgs, SetSurfacePanArgs, SetSurfaceRecArmArgs, SetSurfaceSelectedArgs,
    SetSurfaceSoloArgs, SetSurfaceVolumeArgs, SetTrackTitleArgs, TrackFxChainType, TrackLocation,
    VersionDependentFxLocation, VersionDependentTrackFxLocation,
};
use rxrust::prelude::*;

use std::cell::{Cell, RefCell, RefMut};

use reaper_medium::ProjectContext::{CurrentProject, Proj};
use std::collections::{HashMap, HashSet};


use crate::run_loop_executor::RunLoopExecutor;
use crate::run_loop_scheduler::RxTask;
use crossbeam_channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::iter::once;

#[derive(Debug)]
pub(crate) struct HelperControlSurface {
    // These two are for very simple scheduling. Most light-weight.
    main_thread_task_sender: Sender<MainThreadTask>,
    main_thread_task_receiver: Receiver<MainThreadTask>,
    // This is for executing futures.
    main_thread_executor: RunLoopExecutor,
    // This is for scheduling rxRust observables.
    main_thread_rx_task_receiver: Receiver<RxTask>,
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
}

#[derive(Debug, Default)]
struct FxChainPair {
    input_fx_guids: HashSet<Guid>,
    output_fx_guids: HashSet<Guid>,
}

type ProjectDataMap = HashMap<ReaProject, TrackDataMap>;
type TrackDataMap = HashMap<MediaTrack, TrackData>;

impl HelperControlSurface {
    pub fn new(
        version: ReaperVersion<'static>,
        last_active_project: Project,
        main_thread_task_sender: Sender<MainThreadTask>,
        main_thread_task_receiver: Receiver<MainThreadTask>,
        main_thread_rx_task_receiver: Receiver<RxTask>,
        executor: RunLoopExecutor,
    ) -> HelperControlSurface {
        let reaper_version_5_95 = ReaperVersion::new("5.95");
        HelperControlSurface {
            main_thread_task_sender,
            main_thread_task_receiver,
            main_thread_executor: executor,
            main_thread_rx_task_receiver,
            last_active_project: Cell::new(last_active_project),
            num_track_set_changes_left_to_be_propagated: Default::default(),
            fx_has_been_touched_just_a_moment_ago: Default::default(),
            project_datas: Default::default(),
            fx_chain_pair_by_media_track: Default::default(),
            // since pre1,
            supports_detection_of_input_fx: version >= reaper_version_5_95,
            // since pre2 to be accurate but so what
            supports_detection_of_input_fx_in_set_fx_change: version >= reaper_version_5_95,
        }
    }

    pub fn init(&self) {
        // REAPER doesn't seem to call this automatically when the surface is registered. In our
        // case it's important to call this not at the first change of something (e.g. arm
        // button pressed) but immediately. Because it captures the initial project/track/FX
        // state. If we don't do this immediately, then it happens that change events (e.g.
        // track arm changed) are not reported because the initial state was unknown.
        // TODO-low This executes a bunch of REAPER functions right on start. Maybe do more lazily
        // on activate?  But before activate we can do almost nothing because
        // execute_on_main_thread doesn't work.
        self.set_track_list_change();
    }

    pub fn discard_tasks(&self) {
        self.discard_main_thread_tasks();
        self.discard_main_thread_rx_tasks();
    }

    fn discard_main_thread_tasks(&self) {
        let task_count = self.main_thread_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded main thread tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }

    fn discard_main_thread_rx_tasks(&self) {
        let task_count = self.main_thread_rx_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded main thread rx tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }

    fn state(&self) -> State {
        if self.num_track_set_changes_left_to_be_propagated.get() == 0 {
            State::Normal
        } else {
            State::PropagatingTrackSetChanges
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
        match track.effective_automation_mode() {
            // Is not automated
            None | Some(TrimRead) | Some(Write) => false,
            // Is automated
            _ => true,
        }
    }

    fn remove_invalid_rea_projects(&self) {
        self.project_datas.borrow_mut().retain(|rea_project, _| {
            if Reaper::get()
                .medium_reaper()
                .validate_ptr_2(CurrentProject, *rea_project)
            {
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
        let project = Reaper::get().current_project();
        let mut project_datas = self.project_datas.borrow_mut();
        let track_datas = project_datas.entry(project.raw()).or_default();
        let old_track_count = track_datas.len() as u32;
        // +1 for master track
        let new_track_count = project.track_count() + 1;
        use Ordering::*;
        match new_track_count.cmp(&old_track_count) {
            Less => self.remove_invalid_media_tracks(project, track_datas),
            Equal => self.update_media_track_positions(project, track_datas),
            Greater => self.add_missing_media_tracks(project, track_datas),
        }
    }

    fn add_missing_media_tracks(&self, project: Project, track_datas: &mut TrackDataMap) {
        for t in once(project.master_track()).chain(project.tracks()) {
            let media_track = t.raw();
            track_datas.entry(media_track).or_insert_with(|| {
                let func = Reaper::get().medium_reaper();
                let td = unsafe {
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
                    }
                };
                // TODO-low Use try_borrow_mut(). Then this just doesn't do anything if this event
                //  is currently thrown already. Right now it would panic, which is unreasonable.
                Reaper::get()
                    .subjects
                    .track_added
                    .borrow_mut()
                    .next(t.clone());
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
        let fx_chain_pair = fx_chain_pairs.entry(track.raw()).or_default();
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
            Reaper::get().subjects.fx_reordered.borrow_mut().next(track);
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
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        let new_fx_count = fx_chain.fx_count();
        use Ordering::*;
        match new_fx_count.cmp(&old_fx_count) {
            Less => {
                self.remove_invalid_fx(
                    track,
                    old_fx_guids,
                    is_input_fx,
                    notify_listeners_about_changes,
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
                    Reaper::get()
                        .subjects
                        .fx_removed
                        .borrow_mut()
                        .next(removed_fx);
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
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        for fx in fx_chain.fxs() {
            let was_inserted = fx_guids.insert(fx.guid().expect("No FX GUID set"));
            if was_inserted && notify_listeners_about_changes {
                Reaper::get().subjects.fx_added.borrow_mut().next(fx);
            }
        }
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

    fn remove_invalid_media_tracks(&self, project: Project, track_datas: &mut TrackDataMap) {
        track_datas.retain(|media_track, data| {
            if Reaper::get()
                .medium_reaper()
                .validate_ptr_2(Proj(project.raw()), *media_track)
            {
                true
            } else {
                self.fx_chain_pair_by_media_track
                    .borrow_mut()
                    .remove(media_track);
                let track = project.track_by_guid(&data.guid);
                Reaper::get()
                    .subjects
                    .track_removed
                    .borrow_mut()
                    .next(track);
                false
            }
        });
    }

    fn update_media_track_positions(&self, project: Project, track_datas: &mut TrackDataMap) {
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
            Reaper::get()
                .subjects
                .tracks_reordered
                .borrow_mut()
                .next(project);
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
                Some(args.param_value),
            )
        };
        let fx_chain = if is_input_fx {
            track.input_fx_chain()
        } else {
            track.normal_fx_chain()
        };
        if let Some(fx) = fx_chain.fx_by_index(args.fx_index as u32) {
            let fx_param = fx.parameter_by_index(args.param_index as u32);
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
        normalized_value: Option<ReaperNormalizedFxParamValue>,
    ) -> bool {
        let pairs = self.fx_chain_pair_by_media_track.borrow();
        let pair = match pairs.get(&track.raw()) {
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
            match track.normal_fx_chain().fx_by_index(fx_index) {
                None => true,
                Some(output_fx) => {
                    let output_fx_param = output_fx.parameter_by_index(param_index);
                    let is_probably_output_fx =
                        Some(output_fx_param.reaper_value()) == normalized_value;
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
            .next(());
        // Process plain main thread tasks in queue
        for task in self
            .main_thread_task_receiver
            .try_iter()
            .take(MAIN_THREAD_TASK_BULK_SIZE)
        {
            match task.desired_execution_time {
                None => (task.op)(),
                Some(t) => {
                    if std::time::SystemTime::now() < t {
                        self.main_thread_task_sender
                            .send(task)
                            .expect("couldn't reschedule main thread task");
                    } else {
                        (task.op)()
                    }
                }
            }
        }
        // Execute futures
        self.main_thread_executor.run();
        // Execute observables
        for task in self
            .main_thread_rx_task_receiver
            .try_iter()
            .take(MAIN_THREAD_TASK_BULK_SIZE)
        {
            task();
        }
    }

    fn set_track_list_change(&self) {
        // TODO-low Not multi-project compatible!
        let new_active_project = Reaper::get().current_project();
        if new_active_project != self.last_active_project.get() {
            self.last_active_project.replace(new_active_project);
            Reaper::get()
                .subjects
                .project_switched
                .borrow_mut()
                .next(new_active_project);
        }
        self.num_track_set_changes_left_to_be_propagated
            .replace(new_active_project.track_count() + 1);
        self.remove_invalid_rea_projects();
        self.detect_track_set_changes();
    }

    fn set_surface_pan(&self, args: SetSurfacePanArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.pan == args.pan {
            return;
        }
        td.pan = args.pan;
        let track = Track::new(args.track, None);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_pan_changed
            .borrow_mut()
            .next(track.clone());
        if !self.track_parameter_is_automated(&track, reaper_str!("Pan")) {
            reaper.subjects.track_pan_touched.borrow_mut().next(track);
        }
    }

    fn set_surface_volume(&self, args: SetSurfaceVolumeArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.volume == args.volume {
            return;
        }
        td.volume = args.volume;
        let track = Track::new(args.track, None);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_volume_changed
            .borrow_mut()
            .next(track.clone());
        if !self.track_parameter_is_automated(&track, reaper_str!("Volume")) {
            reaper
                .subjects
                .track_volume_touched
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_mute(&self, args: SetSurfaceMuteArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.mute != args.is_mute {
            td.mute = args.is_mute;
            let track = Track::new(args.track, None);
            let reaper = Reaper::get();
            reaper
                .subjects
                .track_mute_changed
                .borrow_mut()
                .next(track.clone());
            if !self.track_parameter_is_automated(&track, reaper_str!("Mute")) {
                reaper.subjects.track_mute_touched.borrow_mut().next(track);
            }
        }
    }

    fn set_surface_selected(&self, args: SetSurfaceSelectedArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.selected != args.is_selected {
            td.selected = args.is_selected;
            let track = Track::new(args.track, None);
            Reaper::get()
                .subjects
                .track_selected_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_solo(&self, args: SetSurfaceSoloArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.solo != args.is_solo {
            td.solo = args.is_solo;
            let track = Track::new(args.track, None);
            Reaper::get()
                .subjects
                .track_solo_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_rec_arm(&self, args: SetSurfaceRecArmArgs) {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return,
            Some(td) => td,
        };
        if td.recarm != args.is_armed {
            td.recarm = args.is_armed;
            let track = Track::new(args.track, None);
            Reaper::get()
                .subjects
                .track_arm_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_track_title(&self, args: SetTrackTitleArgs) {
        if self.state() == State::PropagatingTrackSetChanges {
            self.decrease_num_track_set_changes_left_to_be_propagated();
            return;
        }
        let track = Track::new(args.track, None);
        Reaper::get()
            .subjects
            .track_name_changed
            .borrow_mut()
            .next(track);
    }

    fn ext_set_input_monitor(&self, args: ExtSetInputMonitorArgs) -> i32 {
        let mut td = match self.find_track_data_in_normal_state(args.track) {
            None => return 1,
            Some(td) => td,
        };
        let reaper = Reaper::get();
        if td.recmonitor != args.mode {
            td.recmonitor = args.mode;
            reaper
                .subjects
                .track_input_monitoring_changed
                .borrow_mut()
                .next(Track::new(args.track, None));
        }
        let recinput = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(args.track, RecInput) as i32
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

    fn ext_set_fx_param(&self, args: ExtSetFxParamArgs) -> i32 {
        self.fx_param_set(args, false);
        1
    }

    fn ext_set_fx_param_rec_fx(&self, args: ExtSetFxParamArgs) -> i32 {
        self.fx_param_set(args, true);
        1
    }

    fn ext_set_fx_enabled(&self, args: ExtSetFxEnabledArgs) -> i32 {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        if let Some(fx) = self.fx_from_parm_fx_index(&track, args.fx_location, None, None) {
            Reaper::get()
                .subjects
                .fx_enabled_changed
                .borrow_mut()
                .next(fx);
        }
        1
    }

    fn ext_set_send_volume(&self, args: ExtSetSendVolumeArgs) -> i32 {
        let track = Track::new(args.track, None);
        let track_send = track.index_based_send_by_index(args.send_index);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_send_volume_changed
            .borrow_mut()
            .next(track_send.clone());
        // Send volume touch event only if not automated
        if !self.track_parameter_is_automated(&track, reaper_str!("Send Volume")) {
            reaper
                .subjects
                .track_send_volume_touched
                .borrow_mut()
                .next(track_send);
        }
        1
    }

    fn ext_set_send_pan(&self, args: ExtSetSendPanArgs) -> i32 {
        let track = Track::new(args.track, None);
        let track_send = track.index_based_send_by_index(args.send_index);
        let reaper = Reaper::get();
        reaper
            .subjects
            .track_send_pan_changed
            .borrow_mut()
            .next(track_send.clone());
        // Send volume touch event only if not automated
        if !self.track_parameter_is_automated(&track, reaper_str!("Send Pan")) {
            reaper
                .subjects
                .track_send_pan_touched
                .borrow_mut()
                .next(track_send);
        }
        1
    }

    fn ext_set_focused_fx(&self, args: ExtSetFocusedFxArgs) -> i32 {
        let reaper = Reaper::get();
        let fx_ref = match args.fx_location {
            None => {
                // Clear focused FX
                reaper.subjects.fx_focused.borrow_mut().next(None);
                return 0;
            }
            Some(r) => r,
        };
        use VersionDependentFxLocation::*;
        match fx_ref.fx_location {
            TakeFx { .. } => {
                // TODO Not handled right now
                0
            }
            TrackFx(track_fx_ref) => {
                // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
                let track = Track::new(fx_ref.track, None);
                if let Some(fx) = self.fx_from_parm_fx_index(&track, track_fx_ref, None, None) {
                    // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2
                    // and on chunk manipulations
                    self.detect_fx_changes_on_track(
                        track,
                        true,
                        !fx.is_input_fx(),
                        fx.is_input_fx(),
                    );
                    reaper.subjects.fx_focused.borrow_mut().next(Some(fx));
                }
                1
            }
        }
    }

    fn ext_set_fx_open(&self, args: ExtSetFxOpenArgs) -> i32 {
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(args.track, None);
        let fx_location = match args.fx_location {
            None => return 1,
            Some(l) => l,
        };
        if let Some(fx) = self.fx_from_parm_fx_index(&track, fx_location, None, None) {
            // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2 and on
            // chunk manipulations
            self.detect_fx_changes_on_track(track, true, !fx.is_input_fx(), fx.is_input_fx());
            let reaper = Reaper::get();
            let subject = if args.is_open {
                &reaper.subjects.fx_opened
            } else {
                &reaper.subjects.fx_closed
            };
            subject.borrow_mut().next(fx);
        }
        1
    }

    fn ext_set_fx_change(&self, args: ExtSetFxChangeArgs) -> i32 {
        let track = Track::new(args.track, None);
        match args.fx_chain_type {
            Some(t) => {
                let is_input_fx = t == TrackFxChainType::InputFxChain;
                self.detect_fx_changes_on_track(track, true, !is_input_fx, is_input_fx);
            }
            None => {
                self.detect_fx_changes_on_track(track, true, true, true);
            }
        }
        1
    }

    fn ext_set_last_touched_fx(&self, _: ExtSetLastTouchedFxArgs) -> i32 {
        self.fx_has_been_touched_just_a_moment_ago.replace(true);
        1
    }

    fn ext_set_bpm_and_play_rate(&self, args: ExtSetBpmAndPlayRateArgs) -> i32 {
        let reaper = Reaper::get();
        if args.tempo.is_some() {
            reaper.subjects.master_tempo_changed.borrow_mut().next(());
            // If there's a tempo envelope, there are just tempo notifications when the tempo is
            // actually changed. So that's okay for "touched".
            // TODO-low What about gradual tempo changes?
            reaper.subjects.master_tempo_touched.borrow_mut().next(());
        }
        if args.play_rate.is_some() {
            reaper
                .subjects
                .master_playrate_changed
                .borrow_mut()
                .next(());
            // FIXME What about playrate automation?
            reaper
                .subjects
                .master_playrate_touched
                .borrow_mut()
                .next(());
        }
        1
    }

    fn ext_track_fx_preset_changed(&self, args: ExtTrackFxPresetChangedArgs) -> i32 {
        let track = Track::new(args.track, None);
        let fx = track
            .fx_by_query_index(args.fx_location.to_raw())
            .expect("preset changed but FX not found");
        Reaper::get()
            .subjects
            .fx_preset_changed
            .borrow_mut()
            .next(fx);
        1
    }
}
