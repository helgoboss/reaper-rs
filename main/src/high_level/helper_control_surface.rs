use crate::high_level::fx::Fx;
use crate::high_level::guid::Guid;
use crate::high_level::{
    get_media_track_guid, AutomationMode, Payload, Project, Reaper, Task, Track,
};
use crate::low_level::{
    MediaTrack, ReaProject, CSURF_EXT_SETBPMANDPLAYRATE, CSURF_EXT_SETFOCUSEDFX,
    CSURF_EXT_SETFXCHANGE, CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN, CSURF_EXT_SETFXPARAM,
    CSURF_EXT_SETFXPARAM_RECFX, CSURF_EXT_SETINPUTMONITOR, CSURF_EXT_SETLASTTOUCHEDFX,
    CSURF_EXT_SETSENDPAN, CSURF_EXT_SETSENDVOLUME,
};
use crate::medium_level::ControlSurface;
use c_str_macro::c_str;
use rxrust::prelude::*;
use std::borrow::Cow;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::Occupied;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::mpsc::Receiver;

const BULK_TASK_EXECUTION_COUNT: usize = 100;

pub struct HelperControlSurface {
    task_receiver: Receiver<Task>,
    last_active_project: Cell<Project>,
    num_track_set_changes_left_to_be_propagated: Cell<u32>,
    fx_has_been_touched_just_a_moment_ago: Cell<bool>,
    project_datas: RefCell<ProjectDataMap>,
    fx_chain_pair_by_media_track: RefCell<HashMap<*mut MediaTrack, FxChainPair>>,

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
    number: i32,
    recmonitor: i32,
    recinput: i32,
    guid: Guid,
}

#[derive(Default)]
struct FxChainPair {
    input_fx_guids: HashSet<Guid>,
    output_fx_guids: HashSet<Guid>,
}

type ProjectDataMap = HashMap<*mut ReaProject, TrackDataMap>;
type TrackDataMap = HashMap<*mut MediaTrack, TrackData>;

impl HelperControlSurface {
    pub fn new(task_receiver: Receiver<Task>) -> HelperControlSurface {
        let reaper = Reaper::instance();
        let version = reaper.get_version();
        let surface = HelperControlSurface {
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
        // REAPER doesn't seem to call this automatically when the surface is registered. In our case it's important
        // to call this not at the first change of something (e.g. arm button pressed) but immediately. Because it
        // captures the initial project/track/FX state. If we don't do this immediately, then it happens that change
        // events (e.g. track arm changed) are not reported because the initial state was unknown.
        // TODO-low This executes a bunch of REAPER functions right on start. Maybe do more lazily on activate?
        //  But before activate we can do almost nothing because execute_on_main_thread doesn't work.
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

    fn find_track_data_in_normal_state<'a>(
        &self,
        track: *mut MediaTrack,
    ) -> Option<RefMut<TrackData>> {
        if self.get_state() == State::PropagatingTrackSetChanges {
            return None;
        }
        self.find_track_data(track)
    }

    fn find_track_data_map(&self) -> Option<RefMut<TrackDataMap>> {
        let rea_project = Reaper::instance().get_current_project().get_rea_project();
        if (!self.project_datas.borrow().contains_key(&rea_project)) {
            return None;
        }
        Some(RefMut::map(self.project_datas.borrow_mut(), |tds| {
            tds.get_mut(&rea_project).unwrap()
        }))
    }

    fn find_track_data<'a>(&self, track: *mut MediaTrack) -> Option<RefMut<TrackData>> {
        let track_data_map = self.find_track_data_map()?;
        if (!track_data_map.contains_key(&track)) {
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
        let env = Reaper::instance()
            .medium
            .get_track_envelope_by_name(track.get_media_track(), parameter_name);
        if env.is_null() {
            return false;
        }
        use AutomationMode::*;
        match track.get_automation_mode() {
            // Is not automated
            Bypass | TrimRead | Write => false,
            // Is automated
            _ => true,
        }
    }

    fn remove_invalid_rea_projects(&self) {
        self.project_datas.borrow_mut().retain(|rea_project, _| {
            if Reaper::instance().medium.validate_ptr_2(
                null_mut(),
                *rea_project as *mut c_void,
                c_str!("ReaProject*"),
            ) {
                true
            } else {
                Reaper::instance()
                    .subjects
                    .project_closed
                    .borrow_mut()
                    .next(Project::new(*rea_project));
                false
            }
        });
    }

    fn detect_track_set_changes(&self) {
        let project = Reaper::instance().get_current_project();
        let mut project_datas = self.project_datas.borrow_mut();
        let mut track_datas = project_datas.entry(project.get_rea_project()).or_default();
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
            let media_track = t.get_media_track();
            track_datas.entry(media_track).or_insert_with(|| {
                let reaper = Reaper::instance();
                let m = &reaper.medium;
                let td = TrackData {
                    // TODO-medium Make functions more accessible in medium API (they are called elsewhere here as well)
                    volume: m.get_media_track_info_value(media_track, c_str!("D_VOL")),
                    pan: m.get_media_track_info_value(media_track, c_str!("D_PAN")),
                    selected: m.get_media_track_info_value(media_track, c_str!("I_SELECTED"))
                        != 0.0,
                    mute: m.get_media_track_info_value(media_track, c_str!("B_MUTE")) != 0.0,
                    solo: m.get_media_track_info_value(media_track, c_str!("I_SOLO")) != 0.0,
                    recarm: m.get_media_track_info_value(media_track, c_str!("I_RECARM")) != 0.0,
                    number: m
                        .get_set_media_track_info(media_track, c_str!("IP_TRACKNUMBER"), null_mut())
                        .0 as i32,
                    recmonitor: m.get_media_track_info_value(media_track, c_str!("I_RECMON"))
                        as i32,
                    recinput: m.get_media_track_info_value(media_track, c_str!("I_RECINPUT"))
                        as i32,
                    guid: get_media_track_guid(media_track),
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
        let mut fx_chain_pair = fx_chain_pairs.entry(track.get_media_track()).or_default();
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
            Reaper::instance()
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
                    Reaper::instance()
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
                Reaper::instance()
                    .subjects
                    .fx_added
                    .borrow_mut()
                    .next(fx.into());
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
            let reaper = Reaper::instance();
            if reaper.medium.validate_ptr_2(
                project.get_rea_project(),
                *media_track as *mut c_void,
                c_str!("MediaTrack*"),
            ) {
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
        let reaper = Reaper::instance();
        for (media_track, track_data) in track_datas.iter_mut() {
            if !reaper.medium.validate_ptr_2(
                project.get_rea_project(),
                *media_track as *mut c_void,
                c_str!("MediaTrack*"),
            ) {
                continue;
            }
            let new_number = reaper
                .medium
                .get_set_media_track_info(*media_track, c_str!("IP_TRACKNUMBER"), null_mut())
                .0 as i32;
            if (new_number != track_data.number) {
                tracks_have_been_reordered = true;
                track_data.number = new_number;
            }
        }
        if tracks_have_been_reordered {
            reaper.subjects.tracks_reordered.borrow_mut().next(project);
        }
    }

    fn csurf_ext_setinputmonitor(&self, track: *mut MediaTrack, recmonitor: *mut i32) {
        if (track.is_null() || recmonitor.is_null()) {
            return;
        }
        let mut td = match self.find_track_data_in_normal_state(track) {
            None => return,
            Some(td) => td,
        };
        let recmonitor = unsafe { *recmonitor };
        let reaper = Reaper::instance();
        if td.recmonitor != recmonitor {
            td.recmonitor = recmonitor;
            reaper
                .subjects
                .track_input_monitoring_changed
                .borrow_mut()
                .next(Track::new(track, null_mut()));
        }
        let recinput = reaper
            .medium
            .get_media_track_info_value(track, c_str!("I_RECINPUT")) as i32;
        if td.recinput != recinput {
            td.recinput = recinput;
            reaper
                .subjects
                .track_input_changed
                .borrow_mut()
                .next(Track::new(track, null_mut()));
        }
    }

    fn csurf_ext_setfxparam(
        &self,
        track: *mut MediaTrack,
        fxidx_and_paramidx: *mut i32,
        normalized_value: *mut f64,
    ) {
        self.fx_param_set(track, fxidx_and_paramidx, normalized_value, false);
    }

    fn csurf_ext_setfxparam_recfx(
        &self,
        track: *mut MediaTrack,
        fxidx_and_paramidx: *mut i32,
        normalized_value: *mut f64,
    ) {
        self.fx_param_set(track, fxidx_and_paramidx, normalized_value, true);
    }

    fn fx_param_set(
        &self,
        track: *mut MediaTrack,
        fxidx_and_paramidx: *mut i32,
        normalized_value: *mut f64,
        is_input_fx_if_supported: bool,
    ) {
        if (track.is_null() || fxidx_and_paramidx.is_null() || normalized_value.is_null()) {
            return;
        }
        let fxidx_and_paramidx = unsafe { *fxidx_and_paramidx };
        let fx_index = (fxidx_and_paramidx >> 16) & 0xffff;
        let param_index = fxidx_and_paramidx & 0xffff;
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(track, null_mut());
        let normalized_value = unsafe { *normalized_value };
        let is_input_fx = if self.supports_detection_of_input_fx {
            is_input_fx_if_supported
        } else {
            self.is_probably_input_fx(&track, fx_index, Some(param_index), Some(normalized_value))
        };
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        if let Some(fx) = fx_chain.get_fx_by_index(fx_index as u32) {
            let fx_param = fx.get_parameter_by_index(param_index as u32);
            let reaper = Reaper::instance();
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
        fx_index: i32,
        param_index: Option<i32>,
        normalized_value: Option<f64>,
    ) -> bool {
        let pairs = self.fx_chain_pair_by_media_track.borrow();
        let pair = match pairs.get(&track.get_media_track()) {
            None => {
                // Should not happen. In this case, an FX yet unknown to Realearn has sent a parameter change
                return false;
            }
            Some(pair) => pair,
        };
        let could_be_input_fx = fx_index < pair.input_fx_guids.len() as i32;
        let could_be_output_fx = fx_index < pair.output_fx_guids.len() as i32;
        if !could_be_input_fx && !could_be_output_fx {
            false
        } else if could_be_input_fx && !could_be_output_fx {
            true
        } else {
            // Could be both
            let param_index = match param_index {
                None => {
                    // We don't have a parameter number at our disposal so we need to guess - we guess normal FX TODO-low
                    return false;
                }
                Some(i) => i,
            };
            // Compare parameter values (a heuristic but so what, it's just for MIDI learn)
            match track.get_normal_fx_chain().get_fx_by_index(fx_index as u32) {
                None => true,
                Some(output_fx) => {
                    let output_fx_param = output_fx.get_parameter_by_index(param_index as u32);
                    let is_probably_output_fx =
                        Some(output_fx_param.get_reaper_value()) == normalized_value;
                    !is_probably_output_fx
                }
            }
        }
    }

    fn csurf_ext_setfxenabled(&self, track: *mut MediaTrack, fxidx: *mut i32, enabled: bool) {
        if track.is_null() || fxidx.is_null() {
            return;
        }
        let fxidx = unsafe { *fxidx };
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(track, null_mut());
        if let Some(fx) = self.get_fx_from_parm_fx_index(&track, fxidx, None, None) {
            Reaper::instance()
                .subjects
                .fx_enabled_changed
                .borrow_mut()
                .next(fx);
        }
    }

    // From REAPER > 5.95, parmFxIndex should be interpreted as query index. For earlier versions it's a normal index
    // - which unfortunately doesn't contain information if the FX is on the normal FX chain or the input FX chain.
    // In this case a heuristic is applied to determine which chain it is. It gets more accurate when paramIndex
    // and paramValue are supplied.
    fn get_fx_from_parm_fx_index(
        &self,
        track: &Track,
        parm_fx_index: i32,
        param_index: Option<i32>,
        param_value: Option<f64>,
    ) -> Option<Fx> {
        if self.supports_detection_of_input_fx {
            track.get_fx_by_query_index(parm_fx_index)
        } else {
            let is_input_fx =
                self.is_probably_input_fx(track, parm_fx_index, param_index, param_value);
            let fx_chain = if is_input_fx {
                track.get_input_fx_chain()
            } else {
                track.get_normal_fx_chain()
            };
            fx_chain.get_fx_by_index(parm_fx_index as u32)
        }
    }

    fn csurf_ext_setsendvolume(&self, track: *mut MediaTrack, sendidx: *mut i32, volume: *mut f64) {
        if track.is_null() || sendidx.is_null() || volume.is_null() {
            return;
        }
        let sendidx = unsafe { *sendidx };
        let track = Track::new(track, null_mut());
        let track_send = track.get_index_based_send_by_index(sendidx as u32);
        let reaper = Reaper::instance();
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
    }

    fn csurf_ext_setsendpan(&self, track: *mut MediaTrack, sendidx: *mut i32, pan: *mut f64) {
        if track.is_null() || sendidx.is_null() || pan.is_null() {
            return;
        }
        let sendidx = unsafe { *sendidx };
        let track = Track::new(track, null_mut());
        let track_send = track.get_index_based_send_by_index(sendidx as u32);
        let reaper = Reaper::instance();
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
    }

    fn csurf_ext_setfocusedfx(
        &self,
        track: *mut MediaTrack,
        mediaitemidx: *mut i32,
        fxidx: *mut i32,
    ) {
        let reaper = Reaper::instance();
        if track.is_null() || !mediaitemidx.is_null() || fxidx.is_null() {
            // Clear focused FX
            reaper.subjects.fx_focused.borrow_mut().next(Payload(None));
            return;
        }
        let fxidx = unsafe { *fxidx };
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(track, null_mut());
        if let Some(fx) = self.get_fx_from_parm_fx_index(&track, fxidx, None, None) {
            // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2 and on chunk manipulations
            self.detect_fx_changes_on_track(track, true, !fx.is_input_fx(), fx.is_input_fx());
            reaper
                .subjects
                .fx_focused
                .borrow_mut()
                .next(Payload(Some(fx)));
        }
    }

    fn csurf_ext_setfxopen(&self, track: *mut MediaTrack, fxidx: *mut i32, ui_open: bool) {
        if track.is_null() || fxidx.is_null() {
            return;
        }
        let fxidx = unsafe { *fxidx };
        // Unfortunately, we don't have a ReaProject* here. Therefore we pass a nullptr.
        let track = Track::new(track, null_mut());
        if let Some(fx) = self.get_fx_from_parm_fx_index(&track, fxidx, None, None) {
            // Because CSURF_EXT_SETFXCHANGE doesn't fire if FX pasted in REAPER < 5.95-pre2 and on chunk manipulations
            self.detect_fx_changes_on_track(track, true, !fx.is_input_fx(), fx.is_input_fx());
            let reaper = Reaper::instance();
            let subject = if ui_open {
                &reaper.subjects.fx_opened
            } else {
                &reaper.subjects.fx_closed
            };
            subject.borrow_mut().next(fx);
        }
    }

    fn csurf_ext_setfxchange(&self, track: *mut MediaTrack, flags: i32) {
        if track.is_null() {
            return;
        }
        let track = Track::new(track, null_mut());
        if self.supports_detection_of_input_fx_in_set_fx_change {
            let is_input_fx = (flags & 1) == 1;
            self.detect_fx_changes_on_track(track, true, !is_input_fx, is_input_fx);
        } else {
            // REAPER < 5.95, we don't know if the change happened on input or normal FX chain
            self.detect_fx_changes_on_track(track, true, true, true);
        }
    }

    fn csurf_ext_setlasttouchedfx(
        &self,
        track: *mut MediaTrack,
        mediaitemidx: *mut i32,
        fxidx: *mut i32,
    ) {
        self.fx_has_been_touched_just_a_moment_ago.replace(true);
    }

    fn csurf_ext_setbpmandplayrate(&self, bpm: *mut f64, playrate: *mut f64) {
        let reaper = Reaper::instance();
        if !bpm.is_null() {
            reaper.subjects.master_tempo_changed.borrow_mut().next(true);
            // If there's a tempo envelope, there are just tempo notifications when the tempo is actually changed.
            // So that's okay for "touched".
            // TODO-low What about gradual tempo changes?
            reaper.subjects.master_tempo_touched.borrow_mut().next(true);
        }
        if !playrate.is_null() {
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
        Reaper::instance()
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
            task();
        }
    }

    fn set_track_list_change(&self) {
        // TODO-low Not multi-project compatible!
        let reaper = Reaper::instance();
        let new_active_project = reaper.get_current_project();
        if (new_active_project != self.last_active_project.get()) {
            self.last_active_project.replace(new_active_project);
            reaper
                .subjects
                .project_switched
                .borrow_mut()
                .next(new_active_project);
        }
        self.num_track_set_changes_left_to_be_propagated
            .replace(new_active_project.get_track_count())
            + 1;
        self.remove_invalid_rea_projects();
        self.detect_track_set_changes();
    }

    fn set_surface_pan(&self, trackid: *mut MediaTrack, pan: f64) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.pan == pan {
            return;
        }
        td.pan = pan;
        let track = Track::new(trackid, null_mut());
        let reaper = Reaper::instance();
        reaper
            .subjects
            .track_pan_changed
            .borrow_mut()
            .next(track.clone());
        if !self.track_parameter_is_automated(&track, c_str!("Pan")) {
            reaper.subjects.track_pan_touched.borrow_mut().next(track);
        }
    }

    fn set_surface_volume(&self, trackid: *mut MediaTrack, volume: f64) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.volume == volume {
            return;
        }
        td.volume = volume;
        let track = Track::new(trackid, null_mut());
        let reaper = Reaper::instance();
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

    fn set_surface_mute(&self, trackid: *mut MediaTrack, mute: bool) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.mute != mute {
            td.mute = mute;
            let track = Track::new(trackid, null_mut());
            let reaper = Reaper::instance();
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

    fn set_surface_selected(&self, trackid: *mut MediaTrack, selected: bool) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.selected != selected {
            td.selected = selected;
            let track = Track::new(trackid, null_mut());
            Reaper::instance()
                .subjects
                .track_selected_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_solo(&self, trackid: *mut MediaTrack, solo: bool) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.solo != solo {
            td.solo = solo;
            let track = Track::new(trackid, null_mut());
            Reaper::instance()
                .subjects
                .track_solo_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn set_surface_rec_arm(&self, trackid: *mut MediaTrack, recarm: bool) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td,
        };
        if td.recarm != recarm {
            td.recarm = recarm;
            let track = Track::new(trackid, null_mut());
            Reaper::instance()
                .subjects
                .track_arm_changed
                .borrow_mut()
                .next(track);
        }
    }

    fn extended(
        &self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        match call as u32 {
            CSURF_EXT_SETINPUTMONITOR => {
                self.csurf_ext_setinputmonitor(parm1 as *mut MediaTrack, parm2 as *mut i32);
                // TODO-medium Why do we return 0 in all csurf_ext_ functions? The doc says this should be returned if unsupported.
                0
            }
            CSURF_EXT_SETFXPARAM => {
                self.csurf_ext_setfxparam(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut f64,
                );
                0
            }
            CSURF_EXT_SETFXPARAM_RECFX => {
                self.csurf_ext_setfxparam_recfx(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut f64,
                );
                0
            }
            CSURF_EXT_SETFXENABLED => {
                self.csurf_ext_setfxenabled(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as usize != 0,
                );
                0
            }
            CSURF_EXT_SETSENDVOLUME => {
                self.csurf_ext_setsendvolume(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut f64,
                );
                0
            }
            CSURF_EXT_SETSENDPAN => {
                self.csurf_ext_setsendpan(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut f64,
                );
                0
            }
            CSURF_EXT_SETFOCUSEDFX => {
                self.csurf_ext_setfocusedfx(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut i32,
                );
                0
            }
            CSURF_EXT_SETFXOPEN => {
                self.csurf_ext_setfxopen(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as usize != 0,
                );
                0
            }
            CSURF_EXT_SETFXCHANGE => {
                self.csurf_ext_setfxchange(parm1 as *mut MediaTrack, parm2 as usize as i32);
                0
            }
            CSURF_EXT_SETLASTTOUCHEDFX => {
                self.csurf_ext_setlasttouchedfx(
                    parm1 as *mut MediaTrack,
                    parm2 as *mut i32,
                    parm3 as *mut i32,
                );
                0
            }
            CSURF_EXT_SETBPMANDPLAYRATE => {
                self.csurf_ext_setbpmandplayrate(parm1 as *mut f64, parm2 as *mut f64);
                0
            }
            _ => 0,
        }
    }

    fn set_track_title(&self, trackid: *mut MediaTrack, title: &CStr) {
        if self.get_state() == State::PropagatingTrackSetChanges {
            self.decrease_num_track_set_changes_left_to_be_propagated();
            return;
        }
        let track = Track::new(trackid, null_mut());
        Reaper::instance()
            .subjects
            .track_name_changed
            .borrow_mut()
            .next(track);
    }
}
