use crate::{Project, Reaper, Take};
use reaper_medium::MediaItem;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Item {
    raw: MediaItem,
}

impl Item {
    pub fn new(raw: MediaItem) -> Item {
        Item { raw }
    }

    pub fn raw(self) -> MediaItem {
        self.raw
    }

    pub fn project(self) -> Option<Project> {
        let raw_project = unsafe {
            Reaper::get()
                .medium_reaper
                .get_item_project_context(self.raw)?
        };
        Some(Project::new(raw_project))
    }

    pub fn active_take(self) -> Option<Take> {
        let raw_take = unsafe { Reaper::get().medium_reaper.get_active_take(self.raw)? };
        Some(Take::new(raw_take))
    }
}
