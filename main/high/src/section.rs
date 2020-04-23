use crate::Action;
use reaper_rs_medium::{KbdCmd, KbdSectionInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    section_info: KbdSectionInfo,
}

impl Section {
    pub(super) fn new(section_info: KbdSectionInfo) -> Section {
        Section { section_info }
    }

    pub fn get_action_by_command_id(&self, command_id: u32) -> Action {
        Action::new(*self, command_id, None)
    }

    pub fn get_action_by_index(&self, index: u32) -> Action {
        if index >= self.get_action_count() {
            panic!("No such action index in section")
        }
        self.get_action_by_index_unchecked(index)
    }

    pub fn get_action_count(&self) -> u32 {
        unsafe { self.section_info.action_list_cnt() }
    }

    pub fn get_raw(&self) -> KbdSectionInfo {
        self.section_info
    }

    pub fn get_actions(&self) -> impl Iterator<Item = Action> + '_ {
        (0..self.get_action_count()).map(move |i| self.get_action_by_index_unchecked(i))
    }

    pub(super) fn get_kbd_cmds(&self) -> impl Iterator<Item = KbdCmd> + '_ {
        (0..self.get_action_count())
            .map(move |i| unsafe { self.section_info.get_action_by_index(i) }.unwrap())
    }

    fn get_action_by_index_unchecked(&self, index: u32) -> Action {
        let kbd_cmd = unsafe { self.section_info.get_action_by_index(index) }.unwrap();
        Action::new(*self, kbd_cmd.cmd(), Some(index))
    }
}
