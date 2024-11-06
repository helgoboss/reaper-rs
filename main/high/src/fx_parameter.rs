use crate::fx::Fx;

use crate::error::ReaperResult;
use crate::{FxChain, FxChainContext, Reaper};
use reaper_medium::{
    GetParamExResult, GetParameterStepSizesResult, ReaperFunctionError,
    ReaperNormalizedFxParamValue, ReaperString,
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
        Reaper::get().require_main_thread();
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_set_param_normalized(
                        track.raw_unchecked(),
                        location,
                        self.index,
                        reaper_value.into(),
                    )
                }
            }
        }
    }

    pub fn reaper_normalized_value(&self) -> ReaperNormalizedFxParamValue {
        Reaper::get().require_main_thread();
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_get_param_normalized(
                        track.raw_unchecked(),
                        location,
                        self.index,
                    )
                }
            }
        }
    }

    pub fn end_edit(&self) -> Result<(), ReaperFunctionError> {
        Reaper::get().require_main_thread();
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_end_param_edit(
                        track.raw_unchecked(),
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

    pub fn name(&self) -> ReaperResult<ReaperString> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                let name = unsafe {
                    Reaper::get().medium_reaper().track_fx_get_param_name(
                        track.raw_unchecked(),
                        location,
                        self.index,
                        256,
                    )?
                };
                Ok(name)
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
        Reaper::get().require_main_thread();
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                let result = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_parameter_step_sizes(
                            track.raw_unchecked(),
                            location,
                            self.index,
                        )?
                };
                // Try to fix some invalid results (which are most likely invalid because of messy
                // plug-ins, not because of REAPER itself)
                if let GetParameterStepSizesResult::Normal { normal_step, .. } = result {
                    if normal_step.is_infinite() {
                        // There was a bug (REAPER <= 6.12) which makes JS FX "Bypass" and "Wet"
                        // parameters return an infinite step size. This
                        // isn't correct, therefore we fix it here.
                        return None;
                    }
                    if normal_step == 0.0 {
                        // Some plug-ins report a parameter as discrete but then report a step size
                        // of zero, which is of course pointless.
                        return None;
                    }
                }
                Some(result)
            }
        }
    }

    pub fn formatted_value(&self) -> Result<ReaperString, ReaperFunctionError> {
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_formatted_param_value(
                            track.raw_unchecked(),
                            location,
                            self.index,
                            256,
                        )
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
                            track.raw_unchecked(),
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
                let span = (range.max_value - range.min_value).abs();
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
    pub fn value_range(&self) -> GetParamExResult {
        Reaper::get().require_main_thread();
        match self.chain().context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.fx().track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_get_param_ex(
                        track.raw_unchecked(),
                        location,
                        self.index,
                    )
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FxParameterCharacter {
    Toggle,
    Discrete,
    Continuous,
}
