use reaper_medium::ReaperWidthValue;

pub struct Width {
    normalized_value: f64,
}

impl Width {
    pub fn from_normalized_value(normalized_value: f64) -> Width {
        assert!((0.0..=1.0).contains(&normalized_value));
        Width { normalized_value }
    }

    pub fn from_reaper_value(reaper_value: ReaperWidthValue) -> Width {
        Width::from_normalized_value((reaper_value.get() + 1.0) / 2.0)
    }

    pub fn normalized_value(&self) -> f64 {
        self.normalized_value
    }

    pub fn reaper_value(&self) -> ReaperWidthValue {
        ReaperWidthValue::new(self.normalized_value * 2.0 - 1.0)
    }
}
