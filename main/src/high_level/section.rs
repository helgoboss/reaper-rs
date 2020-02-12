use crate::low_level::{KbdSectionInfo, KbdCmd};
use crate::high_level::Action;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Section {
    section_info: *mut KbdSectionInfo,
}

impl Section {
    pub(super) fn new(section_info: *mut KbdSectionInfo) -> Section {
        Section { section_info }
    }

    pub fn get_action_by_command_id(&self, command_id: i32) -> Action {
        // TODO Why sometimes i32 and sometimes i64 for command_id (also in original ReaPlus)?
        //  Maybe u32/u64?
        Action::new(*self, command_id as i64, None)
    }

    pub fn get_action_by_index(&self, index: u32) -> Action {
        if index >= self.get_action_count() {
            panic!("No such action index in section")
        }
        self.get_action_by_index_unchecked(index)
    }

    pub fn get_action_count(&self) -> u32 {
        self.get_section_info().action_list_cnt as u32
    }

    // TODO Rename all pointer-returning methods to get_raw_*()
    pub fn get_raw_section_info(&self) -> *mut KbdSectionInfo {
        self.section_info
    }

    pub(super) fn get_kbd_cmds(&self) -> impl Iterator<Item=&KbdCmd> + '_ {
        (0..self.get_action_count()).map(move |i| {
            self.get_kbd_cmd_by_index(i)
        })
    }

    fn get_kbd_cmd_by_index(&self, index: u32) -> &KbdCmd {
        unsafe { &*self.get_section_info().action_list.offset(index as isize) }
    }

    fn get_action_by_index_unchecked(&self, index: u32) -> Action {
        let kbd_cmd = self.get_kbd_cmd_by_index(index);
        Action::new(*self, kbd_cmd.cmd as i64, Some(index))
    }

    fn get_section_info(&self) -> &KbdSectionInfo {
        unsafe { &*self.section_info }
    }
}