use std::ffi::{CString, CStr};
use crate::high_level::{Section, Project, Reaper};
use std::borrow::Cow;
use std::cell::{RefCell, Ref};
use c_str_macro::c_str;
use once_cell::unsync::OnceCell;
use std::ptr::null_mut;

struct RuntimeData {
    section: Section,
    // Sometimes shortly named cmd in REAPER API. Unique within section. Might be filled lazily.
    // For built-in actions this ID is globally stable and will always be found. For custom actions, this ID is only
    // stable at runtime and it might be that it can't be found - which means the action is not available.
    command_id: i64,
    cached_index: Option<u32>,
}

pub struct Action {
    runtime_data: RefCell<Option<RuntimeData>>,
    // Used to represent custom actions that are not available (they don't have a commandId) or for which is not yet
    // known if they are available. Globally unique, not within one section.
    // TODO But currently only mainSection supported. How support other sections?
    command_name: Option<CString>,
}

impl Action {
    pub(super) fn new(section: Section, command_id: i64, index: Option<u32>) -> Action {
        Action {
            command_name: None,
            runtime_data: RefCell::new(Some(RuntimeData {
                section,
                command_id,
                cached_index: index,
            })),
        }
    }

    pub fn get_index(&self) -> u32 {
        self.load_if_necessary_or_complain();
        let mut opt_runtime_data = self.runtime_data.borrow_mut();
        let mut runtime_data = opt_runtime_data.as_mut().unwrap();
        match runtime_data.cached_index {
            None => {
                let index = self.find_index(runtime_data).expect("Index couldn't be found");
                runtime_data.cached_index = Some(index);
                index
            }
            Some(index) => index
        }
    }

    fn find_index(&self, runtime_data: &RuntimeData) -> Option<u32> {
        // TODO Use kbd_enumerateActions
        runtime_data.section.get_kbd_cmds().enumerate()
            .find(|(i, kbd_cmd)| {
                kbd_cmd.cmd as i64 == runtime_data.command_id
            })
            .map(|(i, _)| i as u32)
    }

    pub fn invoke_as_trigger(&self, project: Option<Project>) {
        self.invoke(1.0, false, project)
    }

    pub fn invoke(&self, normalized_value: f64, is_step_count: bool, project: Option<Project>) {
        // TODO I have no idea how to launch an action in a specific section. The first function doesn't seem to launch the action :(
        // bool (*kbd_RunCommandThroughHooks)(KbdSectionInfo* section, int* actionCommandID, int* val, int* valhw, int* relmode, HWND hwnd);
        // int (*KBD_OnMainActionEx)(int cmd, int val, int valhw, int relmode, HWND hwnd, ReaProject* proj);
        self.load_if_necessary_or_complain();
        let action_command_id = self.runtime_data.borrow().as_ref().unwrap().command_id;
        let reaper = Reaper::instance();
        if is_step_count {
            let relative_value = 64 + normalized_value as i32;
            let cropped_relative_value = relative_value.clamp(0, 127);
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val, &valhw, &relmode, reaper::GetMainHwnd());
            reaper.medium.kbd_on_main_action_ex(
                action_command_id as i32,
                cropped_relative_value,
                0,
                2,
                reaper.medium.get_main_hwnd(),
                project.map(|p| p.get_rea_project()).unwrap_or(null_mut()),
            );
        } else {
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val, &valhw, &relmode, reaper::GetMainHwnd());
            reaper.medium.kbd_on_main_action_ex(
                action_command_id as i32,
                (normalized_value * 127 as f64).round() as i32,
                -1,
                0,
                reaper.medium.get_main_hwnd(),
                project.map(|p| p.get_rea_project()).unwrap_or(null_mut()),
            );
            // Main_OnCommandEx would trigger the actionInvoked event but it has not enough parameters for passing values etc.
//          reaper::Main_OnCommandEx(actionCommandId, 0, project ? project->reaProject() : nullptr);
        }
    }

    // TODO Expose runtime data as return value to get rid of the unwraps
    fn load_if_necessary_or_complain(&self) {
        if (self.runtime_data.borrow().is_none() && self.load_by_command_name()) {
            panic!("Action not loadable")
        }
    }

    fn load_by_command_name(&self) -> bool {
        let fixed_command_name = Self::fix_command_name(self.command_name.as_ref().expect("Command name not set"));
        let reaper = Reaper::instance();
        let command_id = reaper.medium.named_command_lookup(&fixed_command_name);
        if command_id == 0 {
            return false;
        }
        self.runtime_data.replace(Some(RuntimeData {
            section: reaper.get_main_section(),
            command_id: command_id as i64,
            cached_index: None,
        }));
        true
    }

    fn fix_command_name<'a>(command_name: &'a CStr) -> Cow<'a, CStr> {
        let bytes = command_name.to_bytes();
        if (!bytes.len() == 0 && bytes[0] == b'_') {
            // Command already contains underscore. Great.
            return Cow::from(command_name);
        }
        if (bytes.len() == 32 && contains_digits_only(command_name)) {
            return Cow::from(command_name);
        }
        // Doesn't contain underscore but should contain one because it's a custom action or an explicitly named command.
        let with_underscore = CString::new([c_str!("_").to_bytes(), bytes].concat()).unwrap();
        return Cow::from(with_underscore);
    }
}

fn contains_digits_only(command_name: &CStr) -> bool {
    let digit_regex = regex!("[^0-9]");
    digit_regex.find(command_name.to_str().unwrap()).is_none()
}