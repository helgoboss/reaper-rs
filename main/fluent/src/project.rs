use crate::{Model, Reaper, Track};
use reaper_medium::{MediaTrack, ProjectContext, ReaProject, TrackDefaultsBehavior};
use std::iter::FusedIterator;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ProjectDesc {
    raw: ReaProject,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Project<'a> {
    model: &'a Model,
    raw: ReaProject,
}

impl ProjectDesc {
    pub(crate) fn new(raw: ReaProject) -> Self {
        Self { raw }
    }

    // pub fn raw(&self) -> ReaProject {
    //     self.0
    // }

    // pub fn get(&self) -> Option<Project> {
    //     if !self.is_valid() {
    //         return None;
    //     }
    //     Some(Project(&self))
    // }
    //
    // pub fn is_valid(&self) -> bool {
    //     Reaper::get()
    //         .medium_reaper()
    //         .validate_ptr_2(ProjectContext::CurrentProject, self.raw)
    // }
}

impl<'a> Project<'a> {
    pub(crate) fn new(model: &'a Model, raw: ReaProject) -> Self {
        Self { model, raw }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn desc(&self) -> ProjectDesc {
        ProjectDesc::new(self.raw())
    }

    pub fn raw(&self) -> ReaProject {
        self.raw
    }

    pub fn insert_track_at(&mut self, index: u32, behavior: TrackDefaultsBehavior) -> Track {
        let r = Reaper::get().medium_reaper();
        r.insert_track_at_index(index, behavior);
        let media_track = r
            .get_track(ProjectContext::CurrentProject, index)
            .expect("impossible");
        Track::new(self, media_track)
    }

    pub fn delete_track(&mut self, track: MediaTrack) {
        unsafe {
            Reaper::get().medium_reaper().delete_track(track);
        }
    }

    pub fn tracks(
        &self,
    ) -> impl Iterator<Item = Track> + ExactSizeIterator + FusedIterator + DoubleEndedIterator {
        let r = Reaper::get().medium_reaper();
        (0..self.track_count()).map(|i| {
            let media_track = r.get_track(self.context(), i).expect("must exist");
            Track::new(self, media_track)
        })
    }

    pub fn track_count(&self) -> u32 {
        Reaper::get().medium_reaper().count_tracks(self.context())
    }

    pub fn context(&self) -> ProjectContext {
        ProjectContext::Proj(self.raw)
    }
}
