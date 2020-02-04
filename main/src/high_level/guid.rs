use crate::low_level::GUID;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Guid {
    internal: GUID
}

impl Guid {
    pub fn new(internal: GUID) -> Guid {
        Guid {
            internal
        }
    }
}