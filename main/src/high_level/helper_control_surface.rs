use std::os::raw::c_void;
use crate::low_level::{MediaTrack, ReaProject};
use crate::medium_level::ControlSurface;
use std::ffi::CStr;
use std::borrow::Cow;
use crate::high_level::{Reaper, Project, Task, Track, LightTrack};
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
    num_track_set_changes_left_to_be_propagated: Cell<i32>,
    fx_has_been_touched_just_a_moment_ago: Cell<bool>,
    track_datas: RefCell<ProjectTrackDataMap>,
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

struct FxChainPair {
    input_fx_guids: HashSet<Guid>,
    output_fx_guids: HashSet<Guid>,
}

type ProjectTrackDataMap = HashMap<*mut ReaProject, TrackDataMap>;
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
            track_datas: Default::default(),
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
        if (!self.track_datas.borrow().contains_key(&rea_project)) {
            return None;
        }
        Some(RefMut::map(self.track_datas.borrow_mut(), |tds| tds.get_mut(&rea_project).unwrap()))
    }

    fn find_track_data<'a>(&self, track: *mut MediaTrack) -> Option<RefMut<TrackData>> {
        let track_data_map = self.find_track_data_map()?;
        if (!track_data_map.contains_key(&track)) {
            return None;
        }
        Some(RefMut::map(track_data_map, |tdm| tdm.get_mut(&track).unwrap()))
    }

    fn track_parameter_is_automated(&self, track: *mut MediaTrack, parameter_name: &CStr) -> bool {
        unimplemented!()
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&mut self) {
        for task in self.task_receiver.try_iter().take(BULK_TASK_EXECUTION_COUNT) {
            task();
        }
    }

    fn set_track_list_change(&self) {
        let reaper = Reaper::instance();
        let new_active_project = reaper.get_current_project();
        if (new_active_project != self.last_active_project.get()) {
            self.last_active_project.replace(new_active_project);
            reaper.subjects.project_switched.borrow_mut().next(new_active_project);
        }
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
        if !self.track_parameter_is_automated(trackid, c_str!("Pan")) {
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
        if !self.track_parameter_is_automated(trackid, c_str!("Volume")) {
            reaper.subjects.track_volume_touched.borrow_mut().next(track);
        }
    }
}