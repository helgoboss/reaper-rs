use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;

use crate::high_level::guid::Guid;
use crate::high_level::{Reaper, Tempo, Track};
use crate::low_level::raw::ReaProject;

use crate::medium_level::{ReaperPointerType, ReaperStringPtr};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Project {
    rea_project: *mut ReaProject,
}

impl Project {
    pub fn new(rea_project: *mut ReaProject) -> Project {
        Project { rea_project }
    }

    pub fn get_raw(&self) -> *mut ReaProject {
        self.rea_project
    }

    pub fn get_first_track(&self) -> Option<Track> {
        self.get_track_by_index(0)
    }

    pub fn get_file_path(&self) -> Option<PathBuf> {
        let (_, path_c_string) = Reaper::get()
            .medium
            .enum_projects(self.get_index() as i32, 5000);
        Some(path_c_string)
            .filter(|path_c_string| path_c_string.to_bytes().len() > 0)
            .map(|path_c_string| {
                let path_str = path_c_string
                    .to_str()
                    .expect("Path contains non-UTF8 characters");
                PathBuf::from_str(path_str).expect("Malformed path")
            })
    }

    pub fn get_index(&self) -> u32 {
        self.complain_if_not_available();
        let rea_project = self.rea_project;
        Reaper::get()
            .get_projects()
            .enumerate()
            .find(|(_, rp)| rp.rea_project == rea_project)
            .map(|(i, _)| i)
            .unwrap() as u32
    }

    /// It's correct that this returns an Option because the index isn't a stable identifier of a
    /// track. The track could move. So this should do a runtime lookup of the track and return a
    /// stable MediaTrack-backed Some(Track) if a track exists at that index. 0 is first normal
    /// track (master track is not obtainable via this method).
    pub fn get_track_by_index(&self, idx: u32) -> Option<Track> {
        self.complain_if_not_available();
        let media_track = Reaper::get().medium.get_track(self.rea_project, idx);
        if media_track.is_null() {
            return None;
        }
        Some(Track::new(media_track, self.rea_project))
    }

    // 0 is master track, 1 is first normal track
    pub fn get_track_by_number(&self, number: u32) -> Option<Track> {
        if number == 0 {
            Some(self.get_master_track())
        } else {
            self.get_track_by_index(number - 1)
        }
    }

    // This returns a non-optional in order to support not-yet-loaded tracks. GUID is a perfectly
    // stable identifier of a track!
    pub fn get_track_by_guid(&self, guid: &Guid) -> Track {
        self.complain_if_not_available();
        Track::from_guid(*self, *guid)
    }

    pub fn get_tracks(&self) -> impl Iterator<Item = Track> + '_ {
        self.complain_if_not_available();
        (0..self.get_track_count()).map(move |i| {
            let media_track = Reaper::get().medium.get_track(self.rea_project, i);
            Track::new(media_track, self.rea_project)
        })
    }

    pub fn is_available(&self) -> bool {
        Reaper::get().medium.validate_ptr_2(
            null_mut(),
            self.rea_project as *mut c_void,
            ReaperPointerType::ReaProject,
        )
    }

    pub fn get_selected_track_count(&self, want_master: bool) -> u32 {
        Reaper::get()
            .medium
            .count_selected_tracks_2(self.rea_project, want_master) as u32
    }

    pub fn get_first_selected_track(&self, want_master: bool) -> Option<Track> {
        let media_track =
            Reaper::get()
                .medium
                .get_selected_track_2(self.rea_project, 0, want_master);
        if media_track.is_null() {
            return None;
        }
        Some(Track::new(media_track, self.rea_project))
    }

    pub fn unselect_all_tracks(&self) {
        // TODO-low No project context
        Reaper::get().medium.set_only_track_selected(null_mut());
    }

    pub fn get_selected_tracks(&self, want_master: bool) -> impl Iterator<Item = Track> + '_ {
        self.complain_if_not_available();
        (0..self.get_selected_track_count(want_master)).map(move |i| {
            let media_track =
                Reaper::get()
                    .medium
                    .get_selected_track_2(self.rea_project, i, want_master);
            Track::new(media_track, self.rea_project)
        })
    }

    pub fn get_track_count(&self) -> u32 {
        self.complain_if_not_available();
        Reaper::get().medium.count_tracks(self.rea_project) as u32
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn add_track(&self) -> Track {
        self.complain_if_not_available();
        self.insert_track_at(self.get_track_count())
    }

    pub fn remove_track(&self, track: &Track) {
        Reaper::get().medium.delete_track(track.get_raw());
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn insert_track_at(&self, index: u32) -> Track {
        self.complain_if_not_available();
        // TODO-low reaper::InsertTrackAtIndex unfortunately doesn't allow to specify ReaProject :(
        let reaper = Reaper::get();
        reaper.medium.insert_track_at_index(index, false);
        reaper.medium.track_list_update_all_external_surfaces();
        let media_track = reaper.medium.get_track(self.rea_project, index);
        Track::new(media_track, self.rea_project)
    }

    pub fn get_master_track(&self) -> Track {
        self.complain_if_not_available();
        let mt = Reaper::get().medium.get_master_track(self.rea_project);
        Track::new(mt, self.rea_project)
    }

    pub fn undoable<F, R>(&self, label: &CStr, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let reaper = Reaper::get();
        if reaper.get_currently_loading_or_saving_project().is_some() {
            operation()
        } else {
            let _undo_block = reaper.enter_undo_block_internal(*self, label);
            operation()
        }
    }

    pub fn undo(&self) -> bool {
        self.complain_if_not_available();
        Reaper::get().medium.undo_do_undo_2(self.rea_project)
    }

    pub fn redo(&self) -> bool {
        Reaper::get().medium.undo_do_redo_2(self.rea_project)
    }

    pub fn mark_as_dirty(&self) {
        Reaper::get().medium.mark_project_dirty(self.rea_project);
    }

    pub fn is_dirty(&self) -> bool {
        Reaper::get().medium.is_project_dirty(self.rea_project)
    }

    pub fn get_label_of_last_undoable_action(&self) -> ReaperStringPtr {
        self.complain_if_not_available();
        Reaper::get().medium.undo_can_undo_2(self.rea_project)
    }

    pub fn get_label_of_last_redoable_action(&self) -> Option<&CStr> {
        self.complain_if_not_available();
        let ptr = Reaper::get().medium.undo_can_redo_2(self.rea_project);
        unsafe { ptr.into_c_str() }
    }

    pub fn get_tempo(&self) -> Tempo {
        // TODO This is not project-specific ... why?
        let tempo = Reaper::get().medium.master_get_tempo();
        Tempo::from_bpm(tempo)
    }

    pub fn set_tempo(&self, tempo: Tempo, want_undo: bool) {
        self.complain_if_not_available();
        Reaper::get()
            .medium
            .set_current_bpm(self.rea_project, tempo.get_bpm(), want_undo);
    }

    fn complain_if_not_available(&self) {
        if !self.is_available() {
            panic!("Project not available");
        }
    }
}
