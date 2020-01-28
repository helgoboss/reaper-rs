use crate::medium_level;
use crate::bindings::{ReaProject, MediaTrack, gaccel_register_t, ACCEL};
use std::ptr::{null_mut, null};
use std::os::raw::{c_void, c_ushort};
use c_str_macro::c_str;
use std::ffi::{CStr, CString};
use std::borrow::{Cow, Borrow, BorrowMut};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::{RefCell, Ref, RefMut};
use std::sync::Once;
use crate::high_level::ActionKind::Toggleable;
use crate::high_level::{Reaper, Track};

pub struct Project {
    rea_project: *mut ReaProject,
}

impl Project {
    pub fn new(rea_project: *mut ReaProject) -> Project {
        Project { rea_project }
    }

    pub fn get_first_track(&self) -> Option<Track> {
        self.get_track_by_index(0)
    }

    pub fn get_file_path(&self) -> Option<String> {
        Reaper::instance().medium.enum_projects(self.get_index(), 5000).1
    }

    pub fn get_index(&self) -> i32 {
        self.complain_if_not_available();
        let rea_project = self.rea_project;
        Reaper::instance().get_projects()
            .enumerate()
            .find(|(_, rp)| rp.rea_project == rea_project)
            .map(|(i, _)| i)
            .unwrap() as i32
    }

    /// It's correct that this returns an Option because the index isn't a stable identifier of a
    /// track. The track could move. So this should do a runtime lookup of the track and return a
    /// stable MediaTrack-backed Some(Track) if a track exists at that index. 0 is first normal
    /// track (master track is not obtainable via this method).
    pub fn get_track_by_index(&self, idx: u32) -> Option<Track> {
        self.complain_if_not_available();
        let media_track = Reaper::instance().medium.get_track(self.rea_project, idx as i32);
        if media_track.is_null() {
            return None;
        }
        Some(Track::new(media_track, self.rea_project))
    }

    pub fn is_available(&self) -> bool {
        Reaper::instance().medium.validate_ptr_2(null_mut(), self.rea_project as *mut c_void, c_str!("ReaProject*"))
    }

    fn complain_if_not_available(&self) {
        if !self.is_available() {
            panic!("Project not available")
        }
    }
}