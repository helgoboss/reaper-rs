use crate::{FxChain, Reaper, ReaperSource, Track};
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

    pub fn name(&self) -> String {
        Reaper::get()
            .medium_reaper
            .get_take_name(self.raw, |result| {
                result.expect("take not valid").to_string()
            })
    }

    pub fn source(&self) -> Option<ReaperSource> {
        let raw_source = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_item_take_source(self.raw)?
        };
        Some(ReaperSource::new(raw_source))
    }
}
