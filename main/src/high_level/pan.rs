use crate::high_level::Reaper;

pub struct Pan {
    normalized_value: f64
}

impl Pan {
    pub fn of_normalized_value(normalized_value: f64) -> Pan {
        assert!(0.0 <= normalized_value && normalized_value <= 1.0);
        Pan {
            normalized_value
        }
    }

    pub fn of_reaper_value(reaper_value: f64) -> Pan {
        assert!(-1.0 <= reaper_value && reaper_value <= 1.0);
        Pan::of_normalized_value((reaper_value + 1.0) / 2.0)
    }

    pub fn get_normalized_value(&self) -> f64 {
        self.normalized_value
    }

    pub fn get_reaper_value(&self) -> f64 {
        self.normalized_value * 2.0 - 1.0
    }
}