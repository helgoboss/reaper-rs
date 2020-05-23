use crate::FxChain;
use reaper_medium::MediaItemTake;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Take {
    raw: MediaItemTake,
}

impl Take {
    pub fn new(raw: MediaItemTake) -> Take {
        Take { raw }
    }

    pub fn fx_chain(&self) -> FxChain {
        FxChain::from_take(*self)
    }
}
