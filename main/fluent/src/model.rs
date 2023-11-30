use crate::access::{ReadAccess, WriteAccess};
use crate::{Project, Reaper};
use reaper_medium::ProjectRef;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Model<A>(pub(crate) A);

impl<A> Model<A> {
    pub fn current_project(&self) -> Project<ReadAccess> {
        self.current_project_internal()
    }

    pub fn current_project_mut(&mut self) -> Project<WriteAccess> {
        self.current_project_internal()
    }

    fn current_project_internal<B>(&self) -> Project<B> {
        let raw = Reaper::get()
            .medium_reaper()
            .enum_projects(ProjectRef::Current, 0)
            .expect("must exist")
            .project;
        Project::new(raw)
    }
}
