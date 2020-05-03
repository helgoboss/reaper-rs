use crate::{Action, Reaper};
use reaper_rs_medium::{CommandId, KbdSectionInfo, SectionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    id: SectionId,
}

impl Section {
    pub(super) fn new(id: SectionId) -> Section {
        Section { id }
    }

    pub fn id(&self) -> SectionId {
        self.id
    }

    pub fn with_raw<R>(&self, f: impl FnOnce(&KbdSectionInfo) -> R) -> Option<R> {
        Reaper::get()
            .medium()
            .functions()
            .section_from_unique_id(self.id, f)
    }

    pub unsafe fn get_raw(&self) -> KbdSectionInfo {
        Reaper::get()
            .medium()
            .functions()
            .section_from_unique_id_unchecked(self.id)
            .unwrap()
    }

    pub fn get_action_by_command_id(&self, command_id: CommandId) -> Action {
        Action::new(*self, command_id, None)
    }

    pub fn get_action_by_index(&self, index: u32) -> Action {
        self.with_raw(|s| {
            assert!(
                index < s.action_list_cnt(),
                "No such action index in section"
            );
            let kbd_cmd = s.get_action_by_index(index).unwrap();
            Action::new(*self, kbd_cmd.cmd(), Some(index))
        })
        .unwrap()
    }

    pub fn get_action_count(&self) -> u32 {
        self.with_raw(|s| s.action_list_cnt()).unwrap()
    }

    // Unsafe because at the time when the iterator is evaluated, the section could be gone
    pub unsafe fn get_actions(&self) -> impl Iterator<Item = Action> + '_ {
        let sec = Reaper::get()
            .medium()
            .functions()
            .section_from_unique_id_unchecked(self.id)
            .unwrap();
        (0..sec.action_list_cnt()).map(move |i| {
            let kbd_cmd = sec.get_action_by_index(i).unwrap();
            Action::new(*self, kbd_cmd.cmd(), Some(i))
        })
    }
}
