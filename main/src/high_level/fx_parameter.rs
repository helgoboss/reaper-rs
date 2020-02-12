use crate::high_level::fx::Fx;
use crate::high_level::Reaper;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxParameter {
    fx: Fx,
    index: u32,
}

impl FxParameter {
    pub(super) fn new(fx: Fx, index: u32) -> FxParameter {
        FxParameter {
            fx,
            index,
        }
    }

    pub fn get_reaper_value(&self) -> f64 {
        Reaper::instance().medium.track_fx_get_param_normalized(
            self.fx.get_track().get_media_track(),
            self.fx.get_query_index(),
            self.index as i32
        )
    }
}