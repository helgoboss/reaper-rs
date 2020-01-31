use crate::low_level::KbdSectionInfo;
use crate::high_level::Action;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Section {
    section_info: *mut KbdSectionInfo,
}

impl Section {
    pub(super) fn new(section_info: *mut KbdSectionInfo) -> Section {
        Section { section_info }
    }

    pub fn action_by_command_id(&self, command_id: i32) -> Action {
        // TODO Why sometimes i32 and sometimes i64 for command_id (also in original ReaPlus)?
        Action::new(*self, command_id as i64, None)
    }
}