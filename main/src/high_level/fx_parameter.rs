use crate::high_level::fx::Fx;
use crate::high_level::{Reaper, Track};
use crate::low_level::MediaTrack;
use rxrust::prelude::PayloadCopy;
use std::ffi::CString;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxParameter {
    fx: Fx,
    index: u32,
}

impl PayloadCopy for FxParameter {}

impl FxParameter {
    pub(super) fn new(fx: Fx, index: u32) -> FxParameter {
        FxParameter { fx, index }
    }

    // Returns normalized value [0, 1]
    pub fn get_normalized_value(&self) -> f64 {
        // TODO-low deal with nullptr MediaTrack (empty string)
        self.get_reaper_value()
    }

    pub fn set_normalized_value(&self, normalized_value: f64) {
        Reaper::instance().medium.track_fx_set_param_normalized(
            self.get_media_track(),
            self.fx.get_query_index(),
            self.index,
            normalized_value,
        );
    }

    pub fn get_reaper_value(&self) -> f64 {
        Reaper::instance().medium.track_fx_get_param_normalized(
            self.fx.get_track().get_media_track(),
            self.fx.get_query_index(),
            self.index,
        )
    }

    pub fn is_available(&self) -> bool {
        self.fx.is_available() && self.index < self.fx.get_parameter_count()
    }

    pub fn get_name(&self) -> CString {
        Reaper::instance()
            .medium
            .track_fx_get_param_name(
                self.get_media_track(),
                self.fx.get_query_index(),
                self.index,
                256,
            )
            .expect("Couldn't get FX parameter name")
    }

    fn get_media_track(&self) -> *mut MediaTrack {
        self.fx.get_track().get_media_track()
    }

    pub fn get_character(&self) -> FxParameterCharacter {
        let result = Reaper::instance().medium.track_fx_get_parameter_step_sizes(
            self.get_media_track(),
            self.fx.get_query_index(),
            self.index,
        );
        let result = match result {
            None => return FxParameterCharacter::Continuous,
            Some(r) => r,
        };
        if result.is_toggle {
            return FxParameterCharacter::Toggle;
        }
        // TODO-medium Use options instead of -1.0 as soon as clear constellations are possible
        if result.small_step != -1.0 || result.step != -1.0 || result.large_step != -1.0 {
            return FxParameterCharacter::Discrete;
        }
        FxParameterCharacter::Continuous
    }

    pub fn get_formatted_value(&self) -> CString {
        Reaper::instance()
            .medium
            .track_fx_get_formatted_param_value(
                self.get_media_track(),
                self.fx.get_query_index(),
                self.index,
                256,
            )
            .expect("Couldn't format FX param value")
    }

    pub fn get_fx(&self) -> Fx {
        self.fx.clone()
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn format_normalized_value(&self, normalized_value: f64) -> CString {
        Reaper::instance()
            .medium
            .track_fx_format_param_value_normalized(
                self.get_media_track(),
                self.fx.get_query_index(),
                self.index,
                normalized_value,
                256,
            )
            .expect("Couldn't format normalized value")
    }

    // Returns a normalized value
    // Returns None if no step size (continuous character)
    // TODO-low This is a too opinionated function in that it already interprets and processes some of REAPER's return
    //  values.
    pub fn get_step_size(&self) -> Option<f64> {
        let result = Reaper::instance().medium.track_fx_get_parameter_step_sizes(
            self.get_media_track(),
            self.fx.get_query_index(),
            self.index,
        );
        result.and_then(move |r| {
            if r.is_toggle {
                return Some(1.0);
            }
            let range = self.get_value_range();
            // We are primarily interested in the smallest step size that makes sense. We can always create multiples of it.
            let span = (range.max_val - range.min_val).abs();
            if span == 0.0 {
                return None;
            }
            // TODO-medium Use options instead of -1.0 as soon as clear constellations are possible
            // TODO-medium Use chaining then (coalesce-like)
            let pref_step_size = if r.small_step != -1.0 {
                r.small_step
            } else if r.step != -1.0 {
                r.step
            } else {
                r.large_step
            };
            if pref_step_size == 1.0 {
                return None;
            }
            Some(pref_step_size / span)
        })
    }

    // Doesn't necessarily return normalized values
    pub fn get_value_range(&self) -> FxParameterValueRange {
        let result = Reaper::instance().medium.track_fx_get_param_ex(
            self.get_media_track(),
            self.fx.get_query_index(),
            self.index,
        );
        FxParameterValueRange {
            min_val: result.min_val,
            mid_val: result.mid_val,
            max_val: result.max_val,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FxParameterCharacter {
    Toggle,
    Discrete,
    Continuous,
}

pub struct FxParameterValueRange {
    pub min_val: f64,
    pub mid_val: f64,
    pub max_val: f64,
}
