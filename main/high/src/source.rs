use crate::{Reaper, Take};
use reaper_low::raw::PCM_source;
use std::ptr::NonNull;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Source {
    raw: NonNull<PCM_source>,
}

impl Source {
    pub fn new(raw: NonNull<PCM_source>) -> Source {
        Source { raw }
    }

    pub fn raw(self) -> NonNull<PCM_source> {
        self.raw
    }
}
