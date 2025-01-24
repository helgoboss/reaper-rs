use crate::{Reaper, ReaperError};
use reaper_medium::{ReaperPanValue, ReaperWidthValue};
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
        ReaperPanValue::new_panic(self.normalized_value * 2.0 - 1.0)
    }
}

impl FromStr for Pan {
    type Err = ReaperError;

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
        write!(f, "{pan_string}")
    }
}

pub trait PanExt {
    /// Returns the pan value. In case of dual-pan, returns the left pan value.
    fn main_pan(self) -> ReaperPanValue;
    fn width(self) -> Option<ReaperWidthValue>;
}

impl PanExt for reaper_medium::Pan {
    /// Returns the pan value. In case of dual-pan, returns the left pan value.
    fn main_pan(self) -> ReaperPanValue {
        use reaper_medium::Pan::*;
        match self {
            BalanceV1(p) => p,
            BalanceV4(p) => p,
            StereoPan { pan, .. } => pan,
            DualPan { left, .. } => left,
            Unknown(_) => ReaperPanValue::CENTER,
        }
    }

    fn width(self) -> Option<ReaperWidthValue> {
        if let reaper_medium::Pan::StereoPan { width, .. } = self {
            Some(width)
        } else {
            None
        }
    }
}
