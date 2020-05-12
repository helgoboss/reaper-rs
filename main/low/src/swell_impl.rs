use crate::{Swell, SwellFunctionPointers};

impl Swell {
    /// Gives access to the SWELL function pointers.
    pub fn pointers(&self) -> &SwellFunctionPointers {
        &self.pointers
    }
}
