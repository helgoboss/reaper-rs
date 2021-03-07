use crate::{Project, Reaper};
use reaper_medium::{BookmarkId, EnumProjectMarkers3Result, NativeColor, PositionInSeconds};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BookmarkType {
    Marker,
    Region,
}

/// A region or marker identified by a region/marker-spanning index.
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

    pub fn basic_info(&self) -> BasicBookmarkInfo {
        self.with_full_info(|res| res.into())
    }

    pub fn name(&self) -> String {
        self.with_full_info(|res| res.name.to_str().to_owned())
    }

    pub fn with_full_info<R>(&self, use_result: impl FnOnce(EnumProjectMarkers3Result) -> R) -> R {
        Reaper::get()
            .medium_reaper()
            .enum_project_markers_3(self.project.context(), self.index, |res| {
                Some(use_result(res?))
            })
            .expect("bookmark doesn't exist")
    }
}

pub struct BasicBookmarkInfo {
    pub id: BookmarkId,
    pub position: PositionInSeconds,
    pub region_end_position: Option<PositionInSeconds>,
    pub color: NativeColor,
}

impl BasicBookmarkInfo {
    pub fn bookmark_type(&self) -> BookmarkType {
        if self.region_end_position.is_some() {
            BookmarkType::Region
        } else {
            BookmarkType::Marker
        }
    }
}

impl From<EnumProjectMarkers3Result<'_>> for BasicBookmarkInfo {
    fn from(res: EnumProjectMarkers3Result) -> Self {
        Self {
            id: res.id,
            position: res.position,
            region_end_position: res.region_end_position,
            color: res.color,
        }
    }
}
