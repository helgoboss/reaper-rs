use crate::{FxChain, Track};
use reaper_medium::MediaItemTake;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

    pub fn track(&self) -> &Track {
        todo!()
    }
}
