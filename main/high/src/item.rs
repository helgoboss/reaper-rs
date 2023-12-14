use crate::error::ReaperResult;
use crate::{Project, Reaper, Take, Volume};
use reaper_medium::{
    BeatAttachMode, DurationInSeconds, FadeCurvature, FadeShape, ItemAttributeKey, ItemGroupId,
    MediaItem, NativeColor, NativeColorValue, PositionInSeconds, ProjectContext,
    ReaperFunctionError, ReaperVolumeValue, RgbColor, UiRefreshBehavior,
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

    pub fn is_available(&self) -> bool {
        Reaper::get()
            .medium_reaper()
            .validate_ptr_2(ProjectContext::CurrentProject, self.raw)
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

    pub fn position(&self) -> PositionInSeconds {
        let pos = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::Position)
        };
        PositionInSeconds::new(pos)
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

    pub fn length(&self) -> DurationInSeconds {
        let val = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::Length)
        };
        DurationInSeconds::new(val)
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

    pub fn set_selected(&self, selected: bool) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .set_media_item_selected(self.raw, selected);
        }
    }

    pub fn set_mute(&self, mute: bool) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::Mute,
                if mute { 1.0 } else { 0.0 },
            )
        }
    }

    pub fn set_loop_source(&self, value: bool) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::LoopSrc,
                if value { 1.0 } else { 0.0 },
            )
        }
    }

    pub fn beat_attach_mode(&self) -> Option<BeatAttachMode> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_beat_attach_mode(self.raw)
        }
    }

    pub fn set_beat_attach_mode(&self, value: Option<BeatAttachMode>) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_beat_attach_mode(self.raw, value)
        }
    }

    pub fn auto_stretch(&self) -> bool {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::AutoStretch)
                == 1.0
        }
    }

    pub fn set_auto_stretch(&self, value: bool) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::AutoStretch,
                if value { 1.0 } else { 0.0 },
            )
        }
    }

    pub fn volume(&self) -> ReaperVolumeValue {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_vol(self.raw)
        }
    }

    pub fn set_volume(&self, value: ReaperVolumeValue) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_vol(self.raw, value);
        }
    }

    pub fn snap_offset(&self) -> DurationInSeconds {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_snap_offset(self.raw)
        }
    }

    pub fn set_snap_offset(&self, value: DurationInSeconds) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_snap_offset(self.raw, value);
        }
    }

    pub fn fade_in_length(&self) -> DurationInSeconds {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_in_len(self.raw)
        }
    }

    pub fn set_fade_in_length(&self, value: DurationInSeconds) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_in_len(self.raw, value);
        }
    }

    pub fn fade_out_length(&self) -> DurationInSeconds {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_out_len(self.raw)
        }
    }

    pub fn set_fade_out_length(&self, value: DurationInSeconds) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_out_len(self.raw, value);
        }
    }

    pub fn fade_in_curvature(&self) -> FadeCurvature {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_in_dir(self.raw)
        }
    }

    pub fn set_fade_in_curvature(&self, value: FadeCurvature) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_in_dir(self.raw, value);
        }
    }

    pub fn fade_in_shape(&self) -> FadeShape {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_in_shape(self.raw)
        }
    }

    pub fn set_fade_in_shape(&self, value: FadeShape) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_in_shape(self.raw, value);
        }
    }

    pub fn fade_out_shape(&self) -> FadeShape {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_out_shape(self.raw)
        }
    }

    pub fn set_fade_out_shape(&self, value: FadeShape) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_out_shape(self.raw, value);
        }
    }

    pub fn fade_out_curvature(&self) -> FadeCurvature {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_out_dir(self.raw)
        }
    }

    pub fn set_fade_out_curvature(&self, value: FadeCurvature) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_out_dir(self.raw, value);
        }
    }

    pub fn auto_fade_in_length(&self) -> Option<DurationInSeconds> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_in_len_auto(self.raw)
        }
    }

    pub fn auto_set_fade_in_length(&self, value: Option<DurationInSeconds>) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_in_len_auto(self.raw, value);
        }
    }

    pub fn auto_fade_out_length(&self) -> Option<DurationInSeconds> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_fade_out_len_auto(self.raw)
        }
    }

    pub fn auto_set_fade_out_length(&self, value: Option<DurationInSeconds>) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_fade_out_len_auto(self.raw, value);
        }
    }

    pub fn group_id(&self) -> Option<ItemGroupId> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_get_group_id(self.raw)
        }
    }

    pub fn set_group_id(&self, value: Option<ItemGroupId>) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_info_set_group_id(self.raw, value);
        }
    }

    pub fn custom_color(&self) -> Option<RgbColor> {
        let reaper = Reaper::get().medium_reaper();
        let res = unsafe { reaper.get_set_media_item_info_get_custom_color(self.raw) };
        if !res.is_used {
            return None;
        }
        Some(reaper.color_from_native(res.color))
    }

    pub fn set_custom_color(&self, color: Option<RgbColor>) {
        let reaper = Reaper::get().medium_reaper();
        let value = match color {
            None => NativeColorValue {
                color: Default::default(),
                is_used: false,
            },
            Some(c) => NativeColorValue {
                color: reaper.color_to_native(c),
                is_used: true,
            },
        };
        unsafe { reaper.get_set_media_item_info_set_custom_color(self.raw, value) };
    }

    pub fn free_mode_y_pos(&self) -> f64 {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::FreeModeY)
        }
    }

    pub fn set_free_mode_y_pos(&self, pos: f64) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::FreeModeY,
                pos,
            )
        }
    }

    pub fn free_mode_height(&self) -> f64 {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::FreeModeH)
        }
    }

    pub fn set_free_mode_height(&self, height: f64) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::FreeModeH,
                height,
            )
        }
    }

    pub fn fixed_lane(&self) -> u32 {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_info_value(self.raw, ItemAttributeKey::FixedLane) as u32
        }
    }

    pub fn set_fixed_lane(&self, lane: u32) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_info_value(
                self.raw,
                ItemAttributeKey::FixedLane,
                lane as _,
            )
        }
    }
}
