use crate::{Reaper, Take};
use reaper_low::raw::PCM_source;
use reaper_medium::PcmSource;
use std::ptr::NonNull;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Source {
    raw: PcmSource,
}

impl Source {
    pub fn new(raw: PcmSource) -> Source {
        Source { raw }
    }

    pub fn raw(self) -> PcmSource {
        self.raw
    }
}
