use crate::high_level::Reaper;

pub struct Volume {
    normalized_value: f64
}

const LN10_OVER_TWENTY: f64 = 0.11512925464970228420089957273422;

impl Volume {
    pub fn of_normalized_value(normalized_value: f64) -> Volume {
        Volume {
            normalized_value
        }
    }

    pub fn of_reaper_value(reaper_value: f64) -> Volume {
        Volume::of_db(reaper_value.ln() / LN10_OVER_TWENTY)
    }

    pub fn of_db(db: f64) -> Volume {
        Volume::of_normalized_value(Reaper::instance().medium.db2slider(db) / 1000.0)
    }

    pub fn get_normalized_value(&self) -> f64 {
        self.normalized_value
    }

    pub fn get_reaper_value(&self) -> f64 {
        (self.get_db() * LN10_OVER_TWENTY).exp()
    }

    pub fn get_db(&self) -> f64 {
        Reaper::instance().medium.slider2db(self.normalized_value * 1000.0)
    }
}