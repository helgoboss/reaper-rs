use crate::Reaper;
use reaper_medium::{Db, ReaperVolumeValue, VolumeSliderValue};
use std::fmt;

/// A type that wraps a normalized REAPER slider value (= slider value / 1000) and is therefore best suited for
/// representing a fader position within the typical REAPER volume range from -inf to 12 dB.
/// TODO-medium This struct needs an overhaul, not ready for prime time at all.
#[derive(Debug)]
pub struct SliderVolume {
    /// A value where 0.0 represents -inf dB and 1.0 represents 12 dB (the "soft maximum").
    ///
    /// It can also be larger than 1.0 but not negative.
    normalized_slider_value: f64,
}

impl Default for SliderVolume {
    fn default() -> Self {
        Self::MIN
    }
}

impl SliderVolume {
    /// Represents -inf dB.
    pub const MIN: SliderVolume = SliderVolume {
        normalized_slider_value: 0.0,
    };

    /// Expects a value where 0.0 represents -inf dB and 1.0 represents 12 dB (the "soft maximum").
    ///
    /// Attention! Because of the fact that REAPER allows exceeding the soft maximum of 12 dB,
    /// the VolumeSliderValue can go beyond 1000, which means that this "normalized value" can go
    /// beyond 1.0!
    pub fn try_from_normalized_slider_value(
        soft_normalized_value: f64,
    ) -> Result<SliderVolume, &'static str> {
        if soft_normalized_value < 0.0 && !soft_normalized_value.is_nan() {
            return Err(
                "soft-normalized value must be positive or NaN in order to represent a volume",
            );
        }
        let volume = SliderVolume {
            normalized_slider_value: soft_normalized_value,
        };
        Ok(volume)
    }

    /// Calculates the volume from the given REAPER volume value.
    pub fn from_reaper_value(reaper_value: ReaperVolumeValue) -> SliderVolume {
        let db = reaper_value.to_db(Db::MINUS_INF);
        SliderVolume::from_db(db)
    }

    /// Calculates the volume from the given dB value.
    pub fn from_db(db: Db) -> SliderVolume {
        let slider_value = Reaper::get().medium_reaper().db2slider(db);
        let soft_normalized_value = slider_value.get() / VolumeSliderValue::TWELVE_DB.get();
        let volume_result = SliderVolume::try_from_normalized_slider_value(soft_normalized_value);
        volume_result.unwrap_or(SliderVolume::MIN)
    }

    /// Returns a number where 0.0 represents -inf dB and 1.0 represents 12 dB (the "soft maximum").
    ///
    /// [`reaper_medium::VolumeSliderValue`] divided by 1000.
    pub fn normalized_slider_value(&self) -> f64 {
        self.normalized_slider_value
    }

    /// Returns the corresponding REAPER volume value.
    pub fn reaper_value(&self) -> ReaperVolumeValue {
        self.db().to_reaper_volume_value()
    }

    /// Returns the corresponding dB value.
    pub fn db(&self) -> Db {
        let slider_value = VolumeSliderValue::new(
            self.normalized_slider_value * VolumeSliderValue::TWELVE_DB.get(),
        );
        Reaper::get().medium_reaper().slider2db(slider_value)
    }
}

impl fmt::Display for SliderVolume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vol_string = Reaper::get()
            .medium_reaper()
            .mk_vol_str(self.reaper_value())
            .into_string();
        write!(f, "{vol_string}")
    }
}
