use crate::fx::Fx;

use crate::Reaper;
use reaper_medium::{GetParameterStepSizesResult, MediaTrack, ReaperNormalizedFxParamValue};
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

    // Returns normalized value [0, 1] TODO WRONG!
    pub fn normalized_value(&self) -> ReaperNormalizedFxParamValue {
        // TODO-low deal with nullptr MediaTrack (empty string)
        self.reaper_value()
    }

    pub fn set_normalized_value(&self, normalized_value: ReaperNormalizedFxParamValue) {
        let _ = unsafe {
            Reaper::get().medium_reaper().track_fx_set_param_normalized(
                self.track_raw(),
                self.fx.query_index(),
                self.index,
                normalized_value,
            )
        };
    }

    pub fn reaper_value(&self) -> ReaperNormalizedFxParamValue {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_param_normalized(
                    self.fx.track().raw(),
                    self.fx.query_index(),
                    self.index,
                )
                .unwrap()
        }
    }

    pub fn is_available(&self) -> bool {
        self.fx.is_available() && self.index < self.fx.parameter_count()
    }

    pub fn name(&self) -> CString {
        unsafe {
            Reaper::get().medium_reaper().track_fx_get_param_name(
                self.track_raw(),
                self.fx.query_index(),
                self.index,
                256,
            )
        }
        .expect("Couldn't get FX parameter name")
    }

    fn track_raw(&self) -> MediaTrack {
        self.fx.track().raw()
    }

    pub fn character(&self) -> FxParameterCharacter {
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_parameter_step_sizes(
                    self.track_raw(),
                    self.fx.query_index(),
                    self.index,
                )
        };
        use GetParameterStepSizesResult::*;
        match result {
            None => FxParameterCharacter::Continuous,
            Some(Toggle) => FxParameterCharacter::Toggle,
            Some(Normal { .. }) => FxParameterCharacter::Discrete,
        }
    }

    pub fn formatted_value(&self) -> CString {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_formatted_param_value(
                    self.track_raw(),
                    self.fx.query_index(),
                    self.index,
                    256,
                )
        }
        .expect("Couldn't format FX param value")
    }

    pub fn fx(&self) -> Fx {
        self.fx.clone()
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn format_normalized_value(
        &self,
        normalized_value: ReaperNormalizedFxParamValue,
    ) -> CString {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_format_param_value_normalized(
                    self.track_raw(),
                    self.fx.query_index(),
                    self.index,
                    normalized_value,
                    256,
                )
        }
        .expect("Couldn't format normalized value")
    }

    // Returns a normalized value
    // Returns None if no step size (continuous character)
    // TODO-low This is a too opinionated function in that it already interprets and processes some
    // of REAPER's return  values.
    pub fn step_size(&self) -> Option<f64> {
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_parameter_step_sizes(
                    self.track_raw(),
                    self.fx.query_index(),
                    self.index,
                )
        }?;
        use GetParameterStepSizesResult::*;
        match result {
            Normal {
                normal_step,
                small_step,
                ..
            } => {
                let range = self.value_range();
                // We are primarily interested in the smallest step size that makes sense. We can
                // always create multiples of it.
                let span = (range.max_val - range.min_val).abs();
                if span == 0.0 {
                    return None;
                }
                let pref_step_size = small_step.unwrap_or(normal_step);
                Some(pref_step_size / span)
            }
            Toggle => Some(1.0),
        }
    }

    // Doesn't necessarily return normalized values
    pub fn value_range(&self) -> FxParameterValueRange {
        let result = unsafe {
            Reaper::get().medium_reaper().track_fx_get_param_ex(
                self.track_raw(),
                self.fx.query_index(),
                self.index,
            )
        };
        FxParameterValueRange {
            min_val: result.min_value,
            mid_val: result.mid_value,
            max_val: result.max_value,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FxParameterCharacter {
    Toggle,
    Discrete,
    Continuous,
}

#[derive(Debug, PartialEq)]
pub struct FxParameterValueRange {
    pub min_val: f64,
    pub mid_val: f64,
    pub max_val: f64,
}
