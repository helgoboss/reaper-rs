use crate::{Reaper, ReaperFunctionPointers};

impl Reaper {
    /// Gives access to the REAPER function pointers.
    pub fn pointers(&self) -> &ReaperFunctionPointers {
        &self.pointers
    }
}
