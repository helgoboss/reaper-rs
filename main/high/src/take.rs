use crate::{FxChain, OwnedSource, Reaper, ReaperSource, Track};
use reaper_medium::{
    FullPitchShiftMode, MediaItemTake, NativeColorValue, PlaybackSpeedFactor, PositionInSeconds,
    ReaperFunctionError, ReaperStringArg, ReaperVolumeValue, RgbColor, TakeAttributeKey,
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Take {
    raw: MediaItemTake,
}

unsafe impl Send for Take {}

impl Take {
    pub fn new(raw: MediaItemTake) -> Take {
        Take { raw }
    }

    pub fn raw(&self) -> MediaItemTake {
        self.raw
    }

    pub fn fx_chain(&self) -> FxChain {
        FxChain::from_take(*self)
    }

    pub fn track(&self) -> &Track {
        todo!()
    }

    pub fn name(&self) -> String {
        Reaper::get()
            .medium_reaper
            .get_take_name(self.raw, |result| {
                result.expect("take not valid").to_string()
            })
    }

    pub fn set_name<'a>(&self, name: impl Into<ReaperStringArg<'a>>) {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_item_take_info_set_name(self.raw(), name);
        }
    }

    pub fn source(&self) -> Option<ReaperSource> {
        let raw_source = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_take_source(self.raw)?
        };
        Some(ReaperSource::new(raw_source))
    }

    pub fn set_source(&self, source: OwnedSource) -> Option<OwnedSource> {
        let previous_source = unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_take_info_set_source(self.raw, source.into_raw())
        };
        previous_source.map(OwnedSource::new)
    }

    pub fn play_rate(&self) -> PlaybackSpeedFactor {
        let val = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_take_info_value(self.raw, TakeAttributeKey::PlayRate)
        };
        PlaybackSpeedFactor::new(val)
    }

    pub fn set_play_rate(&self, factor: f64) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_take_info_value(
                self.raw,
                TakeAttributeKey::PlayRate,
                factor,
            )
        }
    }

    pub fn set_preserve_pitch(&self, value: bool) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_take_info_value(
                self.raw,
                TakeAttributeKey::PPitch,
                if value { 1.0 } else { 0.0 },
            )
        }
    }

    pub fn start_offset(&self) -> PositionInSeconds {
        let pos = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_take_info_value(self.raw, TakeAttributeKey::StartOffs)
        };
        PositionInSeconds::new_panic(pos)
    }

    pub fn set_start_offset(&self, length: PositionInSeconds) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper.set_media_item_take_info_value(
                self.raw,
                TakeAttributeKey::StartOffs,
                length.get(),
            )
        }
    }

    pub fn set_volume(&self, volume: ReaperVolumeValue) -> Result<(), ReaperFunctionError> {
        // TODO-medium Support polarity (negative values)
        unsafe {
            Reaper::get().medium_reaper.set_media_item_take_info_value(
                self.raw,
                TakeAttributeKey::Vol,
                volume.get(),
            )
        }
    }

    pub fn pitch_mode(&self) -> Option<FullPitchShiftMode> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_take_info_get_pitch_mode(self.raw)
        }
    }

    pub fn set_pitch_mode(&self, pitch_mode: Option<FullPitchShiftMode>) {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_take_info_set_pitch_mode(self.raw, pitch_mode);
        }
    }

    pub fn custom_color(&self) -> Option<RgbColor> {
        let reaper = Reaper::get().medium_reaper();
        let res = unsafe { reaper.get_set_media_item_take_info_get_custom_color(self.raw) };
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
        unsafe { reaper.get_set_media_item_take_info_set_custom_color(self.raw, value) };
    }
}
