use crate::{FxChain, Reaper, Source, Track};
use reaper_medium::MediaItemTake;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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

    pub fn source(&self) -> Option<Source> {
        let raw_source = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_take_source(self.raw)?
        };
        Some(Source::new(raw_source))
    }
}
