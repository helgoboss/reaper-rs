use crate::{Reaper, ReaperFunctionPointers};

impl Reaper {
    pub fn pointers(&self) -> &ReaperFunctionPointers {
        &self.pointers
    }
}
