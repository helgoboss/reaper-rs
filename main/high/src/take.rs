use crate::{FxChain, OwnedSource, Reaper, ReaperSource, Track};
use reaper_medium::MediaItemTake;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Take {
    raw: MediaItemTake,
}

unsafe impl Send for Take {}

impl Take {
    pub fn new(raw: MediaItemTake) -> Take {
        Take { raw }
    }

    pub fn raw(&self) -> MediaItemTake {
        self.raw
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

    pub fn set_source(&self, source: OwnedSource) -> Option<OwnedSource> {
        let previous_source = unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_item_take_info_set_source(self.raw, source.into_raw())
        };
        previous_source.map(OwnedSource::new)
    }
}
