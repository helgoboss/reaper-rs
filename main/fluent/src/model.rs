use crate::{Project, ProjectDesc, Reaper, TrackDesc};
use reaper_medium::{ProjectContext, ProjectRef, TrackDefaultsBehavior};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Model(pub(crate) ());

impl Model {
    pub fn current_project(&self) -> Project {
        let raw = Reaper::get()
            .medium_reaper()
            .enum_projects(ProjectRef::Current, 0)
            .expect("must exist")
            .project;
        Project::new(self, raw)
    }
}
