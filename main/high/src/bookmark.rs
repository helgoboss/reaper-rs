use crate::{Project, Reaper};
use reaper_medium::{BookmarkId, EnumProjectMarkers3Result, NativeColor, PositionInSeconds};
use std::cell::Cell;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Bookmark {
    IndexBased(IndexBasedBookmark),
    IdBased(IdBasedBookmark),
}

impl Bookmark {
    pub fn project(&self) -> Project {
        match self {
            Bookmark::IndexBased(b) => b.project,
            Bookmark::IdBased(b) => b.project,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IndexBasedBookmark {
    project: Project,
    index: u32,
}

impl IndexBasedBookmark {
    pub fn new(project: Project, index: u32) -> Self {
        Self { project, index }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn project(&self) -> Project {
        self.project
    }

    pub fn info(&self) -> Result<BookmarkInfo, &'static str> {
        Reaper::get()
            .medium_reaper()
            .enum_project_markers_3(self.project.context(), self.index, |res| Some(res?.into()))
            .ok_or("bookmark doesn't exist")
    }

    pub fn with_info<R>(
        &self,
        use_result: impl FnOnce(Option<EnumProjectMarkers3Result>) -> R,
    ) -> R {
        Reaper::get().medium_reaper().enum_project_markers_3(
            self.project.context(),
            self.index,
            use_result,
        )
    }

    pub fn make_id_based(&self) -> Result<IdBasedBookmark, &'static str> {
        let id = self.id_internal()?;
        let res = IdBasedBookmark {
            project: self.project,
            id,
            index: Cell::new(self.index),
        };
        Ok(res)
    }

    fn id_internal(&self) -> Result<BookmarkId, &'static str> {
        Reaper::get()
            .medium_reaper()
            .enum_project_markers_3(self.project.context(), self.index, |res| Some(res?.id))
            .ok_or("bookmark doesn't exist")
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct IdBasedBookmark {
    project: Project,
    id: BookmarkId,
    index: Cell<u32>,
}

impl IdBasedBookmark {
    pub(crate) fn new(project: Project, id: BookmarkId, index_hint: Option<u32>) -> Self {
        Self {
            project,
            id,
            index: Cell::new(index_hint.unwrap_or_default()),
        }
    }

    pub fn project(&self) -> Project {
        self.project
    }

    pub fn id(&self) -> BookmarkId {
        self.id
    }

    pub fn make_index_based(&self) -> Result<IndexBasedBookmark, &'static str> {
        self.update_index_if_necessary()?;
        let res = IndexBasedBookmark {
            project: self.project,
            index: self.index.get(),
        };
        Ok(res)
    }

    fn update_index_if_necessary(&self) -> Result<(), &'static str> {
        let id_at_index = Reaper::get()
            .medium_reaper()
            .enum_project_markers_3(self.project.context(), self.index.get(), |res| {
                Some(res?.id)
            })
            .ok_or("no bookmark at that index")?;
        if id_at_index == self.id {
            // Index still valid.
            return Ok(());
        }
        // Index not valid. Search new one.
        let new_index = self
            .project
            .bookmarks()
            .find_map(|b| {
                let id = b.id_internal().ok()?;
                if id == self.id { Some(b.index) } else { None }
            })
            .ok_or("bookmark with that ID not found")?;
        self.index.set(new_index);
        Ok(())
    }
}

pub struct BookmarkInfo {
    pub id: BookmarkId,
    pub position: PositionInSeconds,
    pub region_end_position: Option<PositionInSeconds>,
    pub name: String,
    pub color: NativeColor,
}

impl BookmarkInfo {
    pub fn is_region(&self) -> bool {
        self.region_end_position.is_some()
    }
}

impl From<EnumProjectMarkers3Result<'_>> for BookmarkInfo {
    fn from(res: EnumProjectMarkers3Result) -> Self {
        Self {
            id: res.id,
            position: res.position,
            region_end_position: res.region_end_position,
            name: res.name.to_str().to_owned(),
            color: res.color,
        }
    }
}
