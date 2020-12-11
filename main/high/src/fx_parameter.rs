use crate::fx::Fx;

use crate::{FxChain, FxChainContext, Reaper};
use reaper_medium::{
    GetParameterStepSizesResult, ReaperFunctionError, ReaperNormalizedFxParamValue, ReaperString,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxParameter {
    fx: Fx,
    index: u32,
}

impl FxParameter {
    pub(super) fn new(fx: Fx, index: u32) -> FxParameter {
        FxParameter { fx, index }
    }

    pub fn set_reaper_normalized_value(
        &self,
        reaper_value: impl Into<ReaperNormalizedFxParamValue>,
    ) -> Result<(), ReaperFunctionError> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_set_param_normalized(
                        track.raw(),
                        location,
                        self.index,
                        reaper_value.into(),
                    )
                }
            }
        }
    }

    pub fn reaper_normalized_value(
        &self,
    ) -> Result<ReaperNormalizedFxParamValue, ReaperFunctionError> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_get_param_normalized(
                        track.raw(),
                        location,
                        self.index,
                    )
                }
            }
        }
    }

    fn chain(&self) -> &FxChain {
        self.fx().chain()
    }

    pub fn is_available(&self) -> bool {
        self.fx.is_available() && self.index < self.fx.parameter_count()
    }

    pub fn name(&self) -> ReaperString {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_param_name(track.raw(), location, self.index, 256)
                        .expect("Couldn't get FX parameter name")
                }
            }
        }
    }

    pub fn character(&self) -> FxParameterCharacter {
        let result = self.step_sizes();
        use GetParameterStepSizesResult::*;
        match result {
            None => FxParameterCharacter::Continuous,
            Some(Toggle) => FxParameterCharacter::Toggle,
            Some(Normal { .. }) => FxParameterCharacter::Discrete,
        }
    }

    pub fn step_sizes(&self) -> Option<GetParameterStepSizesResult> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_parameter_step_sizes(track.raw(), location, self.index)
                }
            }
        }
    }

    pub fn formatted_value(&self) -> ReaperString {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_formatted_param_value(track.raw(), location, self.index, 256)
                        .expect("Couldn't format FX param value")
                }
            }
        }
    }

    pub fn fx(&self) -> &Fx {
        &self.fx
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn format_reaper_normalized_value(
        &self,
        reaper_value: ReaperNormalizedFxParamValue,
    ) -> Result<ReaperString, ReaperFunctionError> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_format_param_value_normalized(
                            track.raw(),
                            location,
                            self.index,
                            reaper_value,
                            256,
                        )
                }
            }
        }
    }

    // Returns a normalized value
    // Returns None if no step size (continuous character)
    // TODO-low This is a too opinionated function in that it already interprets and processes some
    // of REAPER's return  values.
    pub fn step_size(&self) -> Option<f64> {
        let result = self.step_sizes()?;
        use GetParameterStepSizesResult::*;
        match result {
            Normal {
                normal_step,
                small_step,
                ..
            } => {
                // The reported step sizes relate to the reported value range, which is not always
                // the unit interval! Easy to test with JS FX.
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
        let result = match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_get_param_ex(
                        track.raw(),
                        location,
                        self.index,
                    )
                }
            }
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
