use crate::Reaper;
use reaper_medium::{NormalizedPlayRate, PlaybackSpeedFactor};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
pub struct PlayRate {
    factor: PlaybackSpeedFactor,
}

impl PlayRate {
    pub fn from_normalized_value(value: NormalizedPlayRate) -> PlayRate {
        let factor = Reaper::get()
            .medium_reaper()
            .master_normalize_play_rate_denormalize(value);
        PlayRate { factor }
    }

    pub fn from_playback_speed_factor(factor: PlaybackSpeedFactor) -> PlayRate {
        PlayRate { factor }
    }

    pub fn playback_speed_factor(self) -> PlaybackSpeedFactor {
        self.factor
    }

    pub fn normalized_value(self) -> NormalizedPlayRate {
        Reaper::get()
            .medium_reaper()
            .master_normalize_play_rate_normalize(self.factor)
    }
}
