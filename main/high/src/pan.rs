use crate::Reaper;
use reaper_medium::ReaperPanValue;
use std::fmt;
use std::str::FromStr;

pub struct Pan {
    normalized_value: f64,
}

impl Pan {
    pub fn from_normalized_value(normalized_value: f64) -> Pan {
        assert!((0.0..=1.0).contains(&normalized_value));
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

impl FromStr for Pan {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // At the moment this doesn't fail. But in future we could add extra checks.
        let value = Reaper::get().medium_reaper().parse_pan_str(s);
        Ok(Pan::from_reaper_value(value))
    }
}

impl fmt::Display for Pan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pan_string = Reaper::get()
            .medium_reaper()
            .mk_pan_str(self.reaper_value())
            .into_string();
        write!(f, "{}", pan_string)
    }
}
