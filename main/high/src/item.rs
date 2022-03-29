use crate::{Project, Reaper, Take};
use reaper_medium::{
    DurationInSeconds, MediaItem, PositionInSeconds, ReaperFunctionError, UiRefreshBehavior,
};

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

    pub fn add_take(&self) -> Result<Take, ReaperFunctionError> {
        let raw_take = unsafe {
            Reaper::get()
                .medium_reaper
                .add_take_to_media_item(self.raw())?
        };
        Ok(Take::new(raw_take))
    }

    pub fn set_position(
        &self,
        pos: PositionInSeconds,
        refresh_behavior: UiRefreshBehavior,
    ) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .set_media_item_position(self.raw, pos, refresh_behavior)
        }
    }

    pub fn set_length(
        &self,
        length: DurationInSeconds,
        refresh_behavior: UiRefreshBehavior,
    ) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .set_media_item_length(self.raw, length, refresh_behavior)
        }
    }
}
