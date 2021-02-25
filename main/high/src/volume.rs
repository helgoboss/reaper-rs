use crate::Reaper;
use reaper_medium::{Db, ReaperVolumeValue, VolumeSliderValue};
use std::convert::TryFrom;
use std::fmt;

/// TODO-medium This struct needs an overhaul, not ready for prime time at all.
#[derive(Debug)]
pub struct Volume {
    soft_normalized_value: f64,
}

const LN10_OVER_TWENTY: f64 = 0.115_129_254_649_702_28;

impl Volume {
    pub const MIN: Volume = Volume {
        soft_normalized_value: 0.0,
    };

    // Attention! Because of the fact that REAPER allows exceeding the soft maximum of 12 dB,
    // the VolumeSliderValue can go beyond 1000, which means that this "normalized value" can go
    // beyond 1.0!
    pub fn try_from_soft_normalized_value(
        soft_normalized_value: f64,
    ) -> Result<Volume, &'static str> {
        if soft_normalized_value < 0.0 && !soft_normalized_value.is_nan() {
            return Err(
                "soft-normalized value must be positive or NaN in order to represent a volume",
            );
        }
        let volume = Volume {
            soft_normalized_value,
        };
        Ok(volume)
    }

    pub fn from_reaper_value(reaper_value: ReaperVolumeValue) -> Volume {
        let raw_db = reaper_value.get().ln() / LN10_OVER_TWENTY;
        let db = if raw_db == f64::NEG_INFINITY {
            // REAPER doesn't represent negative infinity as f64::NEG_INFINITY, so we must replace
            // this with REAPER's negative infinity.
            Db::MINUS_INF
        } else {
            // We don't want this to panic and the consumer can expect to get some kind of value
            // if the given input value was a valid ReaperVolumeValue.
            if let Ok(db) = Db::try_from(raw_db) {
                db
            } else {
                return Volume::MIN;
            }
        };
        Volume::from_db(db)
    }

    pub fn from_db(db: Db) -> Volume {
        Volume::try_from_soft_normalized_value(
            Reaper::get().medium_reaper().db2slider(db).get() / 1000.0,
        )
        .unwrap_or(Volume::MIN)
    }

    pub fn soft_normalized_value(&self) -> f64 {
        self.soft_normalized_value
    }

    pub fn reaper_value(&self) -> ReaperVolumeValue {
        ReaperVolumeValue::new((self.db().get() * LN10_OVER_TWENTY).exp())
    }

    pub fn db(&self) -> Db {
        Reaper::get()
            .medium_reaper()
            .slider2db(VolumeSliderValue::new(self.soft_normalized_value * 1000.0))
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vol_string = Reaper::get()
            .medium_reaper()
            .mk_vol_str(self.reaper_value())
            .into_string();
        write!(f, "{}", vol_string)
    }
}
