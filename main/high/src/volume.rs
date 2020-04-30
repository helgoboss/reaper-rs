use crate::Reaper;
use reaper_rs_medium::{Db, ReaperVolumeValue, VolumeSliderValue};

pub struct Volume {
    normalized_value: f64,
}

const LN10_OVER_TWENTY: f64 = 0.11512925464970228420089957273422;

impl Volume {
    // TODO Attention! Because of the fact that REAPER allows exceeding the soft maximum of 12 dB,
    //  the VolumeSliderValue can go beyond 1000, which means that this "normalized value" can go
    //  beyond 1.0! Maybe we should call that value range SoftNormalizedValue.
    pub fn from_normalized_value(normalized_value: f64) -> Volume {
        assert!(0.0 <= normalized_value || normalized_value.is_nan());
        Volume { normalized_value }
    }

    pub fn from_reaper_value(reaper_value: ReaperVolumeValue) -> Volume {
        Volume::from_db(Db::new(reaper_value.get().ln() / LN10_OVER_TWENTY))
    }

    pub fn from_db(db: Db) -> Volume {
        Volume::from_normalized_value(
            Reaper::get().medium().functions().db2slider(db).get() / 1000.0,
        )
    }

    pub fn get_normalized_value(&self) -> f64 {
        self.normalized_value
    }

    pub fn get_reaper_value(&self) -> ReaperVolumeValue {
        ReaperVolumeValue::new((self.get_db().get() * LN10_OVER_TWENTY).exp())
    }

    pub fn get_db(&self) -> Db {
        Reaper::get()
            .medium()
            .functions()
            .slider2db(VolumeSliderValue::new(self.normalized_value * 1000.0))
    }
}
