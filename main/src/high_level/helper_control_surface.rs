use std::os::raw::c_void;
use crate::low_level::{MediaTrack, ReaProject};
use crate::medium_level::ControlSurface;
use std::ffi::CStr;
use std::borrow::Cow;
use crate::high_level::{Reaper, Project, Task, Track, LightTrack, AutomationMode, get_media_track_guid};
use rxrust::prelude::*;
use std::cell::{RefCell, Cell, RefMut, Ref};
use std::sync::mpsc::Receiver;
use crate::high_level::guid::Guid;
use std::collections::{HashSet, HashMap};
use c_str_macro::c_str;
use std::ptr::null_mut;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::Occupied;

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
        // TODO This executes a bunch of REAPER functions right on start. Maybe do more lazily on activate?
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

    fn find_track_data_in_normal_state<'a>(&self, track: *mut MediaTrack) -> Option<RefMut<TrackData>> {
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
        Some(RefMut::map(self.project_datas.borrow_mut(), |tds| tds.get_mut(&rea_project).unwrap()))
    }

    fn find_track_data<'a>(&self, track: *mut MediaTrack) -> Option<RefMut<TrackData>> {
        let track_data_map = self.find_track_data_map()?;
        if (!track_data_map.contains_key(&track)) {
            return None;
        }
        Some(RefMut::map(track_data_map, |tdm| tdm.get_mut(&track).unwrap()))
    }

    fn track_parameter_is_automated(&self, track: Track, parameter_name: &CStr) -> bool {
        if !track.is_available() {
            return false;
        }
        let env = Reaper::instance().medium.get_track_envelope_by_name(track.get_media_track(), parameter_name);
        if env.is_null() {
            return false;
        }
        use AutomationMode::*;
        match track.get_automation_mode() {
            // Is not automated
            Bypass | TrimRead | Write => false,
            // Is automated
            _ => true
        }
    }

    fn remove_invalid_rea_projects(&self) {
        self.project_datas.borrow_mut().retain(|rea_project, _| {
            if Reaper::instance().medium.validate_ptr_2(null_mut(), *rea_project as *mut c_void, c_str!("ReaProject*")) {
                true
            } else {
                Reaper::instance().subjects.project_closed.borrow_mut().next(Project::new(*rea_project));
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
                    volume: m.get_media_track_info_value(media_track, c_str!("D_VOL")),
                    pan: m.get_media_track_info_value(media_track, c_str!("D_PAN")),
                    selected: m.get_media_track_info_value(media_track, c_str!("I_SELECTED")) != 0.0,
                    mute: m.get_media_track_info_value(media_track, c_str!("B_MUTE")) != 0.0,
                    solo: m.get_media_track_info_value(media_track, c_str!("I_SOLO")) != 0.0,
                    recarm: m.get_media_track_info_value(media_track, c_str!("I_RECARM")) != 0.0,
                    number: m.convenient_get_media_track_info_i32(media_track, c_str!("IP_TRACKNUMBER")),
                    recmonitor: m.get_media_track_info_value(media_track, c_str!("I_RECMON")) as i32,
                    recinput: m.get_media_track_info_value(media_track, c_str!("I_RECINPUT")) as i32,
                    guid: get_media_track_guid(media_track),
                };
                reaper.subjects.track_added.borrow_mut().next(t.clone().into());
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
            self.detect_fx_changes_on_track_internal(&track, &mut fx_chain_pair.output_fx_guids,
                                                     false, notify_listeners_about_changes)
        } else {
            false
        };
        let added_or_removed_input_fx = if check_input_fx_chain {
            self.detect_fx_changes_on_track_internal(&track, &mut fx_chain_pair.input_fx_guids,
                                                     true, notify_listeners_about_changes)
        } else {
            false
        };
        if notify_listeners_about_changes && !added_or_removed_input_fx && !added_or_removed_output_fx {
            Reaper::instance().subjects.fx_reordered.borrow_mut().next(track.into());
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
            self.remove_invalid_fx(track, old_fx_guids, is_input_fx, notify_listeners_about_changes);
            true
        } else if new_fx_count > old_fx_count {
            self.add_missing_fx(track, old_fx_guids, is_input_fx, notify_listeners_about_changes);
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
                    Reaper::instance().subjects.fx_removed.borrow_mut().next(removed_fx.into());
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
                Reaper::instance().subjects.fx_added.borrow_mut().next(fx.into());
            }
        }
    }

    fn get_fx_guids_on_track(&self, track: &Track, is_input_fx: bool) -> HashSet<Guid> {
        let fx_chain = if is_input_fx {
            track.get_input_fx_chain()
        } else {
            track.get_normal_fx_chain()
        };
        fx_chain.get_fxs().map(|fx| fx.get_guid().expect("No FX GUID set")).collect()
    }

    fn remove_invalid_media_tracks(&self, project: Project, track_datas: &mut TrackDataMap) {
        track_datas.retain(|media_track, data| {
            let reaper = Reaper::instance();
            if reaper.medium.validate_ptr_2(project.get_rea_project(), *media_track as *mut c_void, c_str!("MediaTrack*")) {
                true
            } else {
                self.fx_chain_pair_by_media_track.borrow_mut().remove(media_track);
                let track = project.get_track_by_guid(&data.guid);
                reaper.subjects.track_removed.borrow_mut().next(track.into());
                false
            }
        });
    }

    fn update_media_track_positions(&self, project: Project, track_datas: &mut TrackDataMap) {
        let mut tracks_have_been_reordered = false;
        let reaper = Reaper::instance();
        for (media_track, track_data) in track_datas.iter_mut() {
            if !reaper.medium.validate_ptr_2(project.get_rea_project(), *media_track as *mut c_void, c_str!("MediaTrack*")) {
                continue;
            }
            let new_number = reaper.medium.convenient_get_media_track_info_i32(*media_track, c_str!("IP_TRACKNUMBER"));
            if (new_number != track_data.number) {
                tracks_have_been_reordered = true;
                track_data.number = new_number;
            }
        }
        if tracks_have_been_reordered {
            reaper.subjects.tracks_reordered.borrow_mut().next(project);
        }
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&mut self) {
        for task in self.task_receiver.try_iter().take(BULK_TASK_EXECUTION_COUNT) {
            task();
        }
    }

    fn set_track_list_change(&self) {
        // TODO Not multi-project compatible!
        let reaper = Reaper::instance();
        let new_active_project = reaper.get_current_project();
        if (new_active_project != self.last_active_project.get()) {
            self.last_active_project.replace(new_active_project);
            reaper.subjects.project_switched.borrow_mut().next(new_active_project);
        }
        self.num_track_set_changes_left_to_be_propagated.replace(new_active_project.get_track_count()) + 1;
        self.remove_invalid_rea_projects();
        self.detect_track_set_changes();
    }

    fn set_surface_pan(&self, trackid: *mut MediaTrack, pan: f64) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td
        };
        if td.pan == pan {
            return;
        }
        td.pan = pan;
        let track = LightTrack::new(trackid, null_mut());
        let reaper = Reaper::instance();
        reaper.subjects.track_pan_changed.borrow_mut().next(track);
        if !self.track_parameter_is_automated(track.into(), c_str!("Pan")) {
            reaper.subjects.track_pan_touched.borrow_mut().next(track);
        }
    }


    fn set_surface_volume(&self, trackid: *mut MediaTrack, volume: f64) {
        let mut td = match self.find_track_data_in_normal_state(trackid) {
            None => return,
            Some(td) => td
        };
        if td.volume == volume {
            return;
        }
        td.volume = volume;
        let track = LightTrack::new(trackid, null_mut());
        let reaper = Reaper::instance();
        reaper.subjects.track_volume_changed.borrow_mut().next(track);
        if !self.track_parameter_is_automated(track.into(), c_str!("Volume")) {
            reaper.subjects.track_volume_touched.borrow_mut().next(track);
        }
    }
}