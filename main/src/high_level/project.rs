use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr::null_mut;

use crate::high_level::guid::Guid;
use crate::high_level::{Reaper, Tempo, Track};
use crate::low_level::raw;
use crate::medium_level::{
    ProjectRef, ReaProject, ReaperPointer, TrackRef, WantDefaults, WantMaster, WantUndo,
};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Project {
    rea_project: ReaProject,
}

impl Project {
    pub fn new(rea_project: ReaProject) -> Project {
        Project { rea_project }
    }

    pub fn get_raw(&self) -> ReaProject {
        self.rea_project
    }

    pub fn get_first_track(&self) -> Option<Track> {
        self.get_track_by_index(0)
    }

    pub fn get_file_path(&self) -> Option<PathBuf> {
        Reaper::get()
            .medium
            .enum_projects(ProjectRef::TabIndex(self.get_index()), 5000)
            .unwrap()
            .file_path
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
        let media_track = Reaper::get()
            .medium
            .get_track(Some(self.rea_project), idx)?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    // TODO Probably an unnecessary method
    pub fn get_track_by_ref(&self, track_ref: TrackRef) -> Option<Track> {
        use TrackRef::*;
        match track_ref {
            MasterTrack => Some(self.get_master_track()),
            TrackIndex(idx) => self.get_track_by_index(idx),
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
            let media_track = Reaper::get()
                .medium
                .get_track(Some(self.rea_project), i)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        })
    }

    pub fn is_available(&self) -> bool {
        Reaper::get().medium.validate_ptr_2(None, self.rea_project)
    }

    pub fn get_selected_track_count(&self, want_master: WantMaster) -> u32 {
        Reaper::get()
            .medium
            .count_selected_tracks_2(Some(self.rea_project), want_master) as u32
    }

    pub fn get_first_selected_track(&self, want_master: WantMaster) -> Option<Track> {
        let media_track =
            Reaper::get()
                .medium
                .get_selected_track_2(Some(self.rea_project), 0, want_master)?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    pub fn unselect_all_tracks(&self) {
        // TODO-low No project context
        Reaper::get().medium.set_only_track_selected(None);
    }

    pub fn get_selected_tracks(&self, want_master: WantMaster) -> impl Iterator<Item = Track> + '_ {
        self.complain_if_not_available();
        (0..self.get_selected_track_count(want_master)).map(move |i| {
            let media_track = Reaper::get()
                .medium
                .get_selected_track_2(Some(self.rea_project), i, want_master)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        })
    }

    pub fn get_track_count(&self) -> u32 {
        self.complain_if_not_available();
        Reaper::get().medium.count_tracks(Some(self.rea_project)) as u32
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
        reaper.medium.insert_track_at_index(index, WantDefaults::No);
        reaper.medium.track_list_update_all_external_surfaces();
        let media_track = reaper
            .medium
            .get_track(Some(self.rea_project), index)
            .unwrap();
        Track::new(media_track, Some(self.rea_project))
    }

    pub fn get_master_track(&self) -> Track {
        self.complain_if_not_available();
        let mt = Reaper::get()
            .medium
            .get_master_track(Some(self.rea_project));
        Track::new(mt, Some(self.rea_project))
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
        Reaper::get().medium.undo_do_undo_2(Some(self.rea_project))
    }

    pub fn redo(&self) -> bool {
        Reaper::get().medium.undo_do_redo_2(Some(self.rea_project))
    }

    pub fn mark_as_dirty(&self) {
        Reaper::get()
            .medium
            .mark_project_dirty(Some(self.rea_project));
    }

    pub fn is_dirty(&self) -> bool {
        Reaper::get()
            .medium
            .is_project_dirty(Some(self.rea_project))
    }

    pub fn get_label_of_last_undoable_action(&self) -> Option<CString> {
        self.complain_if_not_available();
        Reaper::get()
            .medium
            .undo_can_undo_2(Some(self.rea_project), |s| s.into())
    }

    pub fn get_label_of_last_redoable_action(&self) -> Option<CString> {
        self.complain_if_not_available();
        Reaper::get()
            .medium
            .undo_can_redo_2(Some(self.rea_project), |s| s.into())
    }

    pub fn get_tempo(&self) -> Tempo {
        // TODO This is not project-specific ... why?
        let tempo = Reaper::get().medium.master_get_tempo();
        Tempo::from_bpm(tempo)
    }

    pub fn set_tempo(&self, tempo: Tempo, undo_hint: WantUndo) {
        self.complain_if_not_available();
        Reaper::get()
            .medium
            .set_current_bpm(Some(self.rea_project), tempo.get_bpm(), undo_hint);
    }

    fn complain_if_not_available(&self) {
        if !self.is_available() {
            panic!("Project not available");
        }
    }
}
