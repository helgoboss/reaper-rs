use std::ffi::{CStr, CString};

use crate::guid::Guid;
use crate::{Reaper, Tempo, Track};

use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::{
    MasterTrackBehavior, ProjectRef, ReaProject, TrackDefaultsBehavior, TrackRef, UndoBehavior,
};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Project {
    rea_project: ReaProject,
}

impl Project {
    pub fn new(rea_project: ReaProject) -> Project {
        Project { rea_project }
    }

    pub fn get_raw(self) -> ReaProject {
        self.rea_project
    }

    pub fn get_first_track(self) -> Option<Track> {
        self.get_track_by_index(0)
    }

    pub fn get_file_path(self) -> Option<PathBuf> {
        Reaper::get()
            .medium_reaper()
            .enum_projects(ProjectRef::Tab(self.get_index()), 5000)
            .unwrap()
            .file_path
    }

    pub fn get_index(self) -> u32 {
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
    pub fn get_track_by_index(self, idx: u32) -> Option<Track> {
        self.complain_if_not_available();
        let media_track = Reaper::get()
            .medium_reaper()
            .get_track(Proj(self.rea_project), idx)?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    // TODO Probably an unnecessary method
    pub fn get_track_by_ref(self, track_ref: TrackRef) -> Option<Track> {
        use TrackRef::*;
        match track_ref {
            MasterTrack => Some(self.get_master_track()),
            NormalTrack(idx) => self.get_track_by_index(idx),
        }
    }

    // This returns a non-optional in order to support not-yet-loaded tracks. GUID is a perfectly
    // stable identifier of a track!
    pub fn get_track_by_guid(self, guid: &Guid) -> Track {
        self.complain_if_not_available();
        Track::from_guid(self, *guid)
    }

    pub fn get_tracks(self) -> impl Iterator<Item = Track> + 'static {
        self.complain_if_not_available();
        (0..self.get_track_count()).map(move |i| {
            let media_track = Reaper::get()
                .medium_reaper()
                .get_track(Proj(self.rea_project), i)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        })
    }

    pub fn is_available(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .validate_ptr_2(CurrentProject, self.rea_project)
    }

    pub fn get_selected_track_count(self, want_master: MasterTrackBehavior) -> u32 {
        Reaper::get()
            .medium_reaper()
            .count_selected_tracks_2(Proj(self.rea_project), want_master) as u32
    }

    pub fn get_first_selected_track(self, want_master: MasterTrackBehavior) -> Option<Track> {
        let media_track = Reaper::get().medium_reaper().get_selected_track_2(
            Proj(self.rea_project),
            0,
            want_master,
        )?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    pub fn unselect_all_tracks(self) {
        // TODO-low No project context
        unsafe {
            Reaper::get().medium_reaper().set_only_track_selected(None);
        }
    }

    pub fn get_selected_tracks(
        self,
        want_master: MasterTrackBehavior,
    ) -> impl Iterator<Item = Track> + 'static {
        self.complain_if_not_available();
        (0..self.get_selected_track_count(want_master)).map(move |i| {
            let media_track = Reaper::get()
                .medium_reaper()
                .get_selected_track_2(Proj(self.rea_project), i, want_master)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        })
    }

    pub fn get_track_count(self) -> u32 {
        self.complain_if_not_available();
        Reaper::get()
            .medium_reaper()
            .count_tracks(Proj(self.rea_project)) as u32
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn add_track(self) -> Track {
        self.complain_if_not_available();
        self.insert_track_at(self.get_track_count())
    }

    pub fn remove_track(self, track: &Track) {
        unsafe {
            Reaper::get().medium_reaper().delete_track(track.get_raw());
        }
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn insert_track_at(self, index: u32) -> Track {
        self.complain_if_not_available();
        // TODO-low reaper::InsertTrackAtIndex unfortunately doesn't allow to specify ReaProject :(
        let reaper = Reaper::get().medium_reaper();
        reaper.insert_track_at_index(index, TrackDefaultsBehavior::OmitDefaultEnvAndFx);
        reaper.track_list_update_all_external_surfaces();
        let media_track = reaper.get_track(Proj(self.rea_project), index).unwrap();
        Track::new(media_track, Some(self.rea_project))
    }

    pub fn get_master_track(self) -> Track {
        self.complain_if_not_available();
        let mt = Reaper::get()
            .medium_reaper()
            .get_master_track(Proj(self.rea_project));
        Track::new(mt, Some(self.rea_project))
    }

    pub fn undoable<F, R>(self, label: &CStr, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        if Reaper::get()
            .get_currently_loading_or_saving_project()
            .is_some()
        {
            operation()
        } else {
            let _undo_block = Reaper::get().enter_undo_block_internal(self, label);
            operation()
        }
    }

    pub fn undo(self) -> bool {
        self.complain_if_not_available();
        Reaper::get()
            .medium_reaper()
            .undo_do_undo_2(Proj(self.rea_project))
    }

    pub fn redo(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .undo_do_redo_2(Proj(self.rea_project))
    }

    pub fn mark_as_dirty(self) {
        Reaper::get()
            .medium_reaper()
            .mark_project_dirty(Proj(self.rea_project));
    }

    pub fn is_dirty(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .is_project_dirty(Proj(self.rea_project))
    }

    pub fn get_label_of_last_undoable_action(self) -> Option<CString> {
        self.complain_if_not_available();
        Reaper::get()
            .medium_reaper()
            .undo_can_undo_2(Proj(self.rea_project), |s| s.into())
    }

    pub fn get_label_of_last_redoable_action(self) -> Option<CString> {
        self.complain_if_not_available();
        Reaper::get()
            .medium_reaper()
            .undo_can_redo_2(Proj(self.rea_project), |s| s.into())
    }

    pub fn get_tempo(self) -> Tempo {
        // TODO This is not project-specific ... why?
        let bpm = Reaper::get().medium_reaper().master_get_tempo();
        Tempo::from_bpm(bpm)
    }

    pub fn set_tempo(self, tempo: Tempo, undo_hint: UndoBehavior) {
        self.complain_if_not_available();
        Reaper::get().medium_reaper().set_current_bpm(
            Proj(self.rea_project),
            tempo.get_bpm(),
            undo_hint,
        );
    }

    fn complain_if_not_available(self) {
        if !self.is_available() {
            panic!("Project not available");
        }
    }
}
