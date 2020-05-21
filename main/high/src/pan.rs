use reaper_medium::ReaperPanValue;

pub struct Pan {
    normalized_value: f64,
}

impl Pan {
    pub fn from_normalized_value(normalized_value: f64) -> Pan {
        assert!(0.0 <= normalized_value && normalized_value <= 1.0);
        Pan { normalized_value }
    }

    pub fn from_reaper_value(reaper_value: ReaperPanValue) -> Pan {
        Pan::from_normalized_value((reaper_value.get() + 1.0) / 2.0)
    }

    pub fn normalized_value(&self) -> f64 {
        self.normalized_value
    }

    pub fn reaper_value(&self) -> ReaperPanValue {
        ReaperPanValue::new(self.normalized_value * 2.0 - 1.0)
    }
}
