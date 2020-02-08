use crate::high_level::fx::{Fx, LightFx};


/// The difference to FxParameter is that this implements Copy (not just Clone). See LightTrack for explanation.
#[derive(Clone, Copy, Debug)]
pub struct LightFxParameter {
    fx: LightFx,
    index: u32,
}

impl From<LightFxParameter> for FxParameter {
    fn from(light: LightFxParameter) -> Self {
        FxParameter {
            fx: light.fx.into(),
            index: light.index,
        }
    }
}

impl From<FxParameter> for LightFxParameter {
    fn from(heavy: FxParameter) -> Self {
        LightFxParameter {
            fx: heavy.fx.into(),
            index: heavy.index,
        }
    }
}

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
}