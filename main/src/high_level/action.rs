use std::ffi::{CString, CStr};
use crate::high_level::{Section, Project, Reaper, ActionCharacter, ParameterType};
use std::borrow::Cow;
use std::cell::{RefCell, Ref};
use c_str_macro::c_str;
use once_cell::unsync::OnceCell;
use std::ptr::null_mut;

#[derive(Debug, Copy, Clone)]
struct RuntimeData {
    section: Section,
    // Sometimes shortly named cmd in REAPER API. Unique within section. Might be filled lazily.
    // For built-in actions this ID is globally stable and will always be found. For custom actions, this ID is only
    // stable at runtime and it might be that it can't be found - which means the action is not available.
    command_id: i64,
    cached_index: Option<u32>,
}

// TODO Use separate classes for loaded and not loaded actions
#[derive(Debug, Clone)]
pub struct Action {
    runtime_data: RefCell<Option<RuntimeData>>,
    // Used to represent custom actions that are not available (they don't have a commandId) or for which is not yet
    // known if they are available. Globally unique, not within one section.
    // TODO But currently only mainSection supported. How support other sections?
    command_name: Option<CString>,
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self.runtime_data.borrow().as_ref(), other.runtime_data.borrow().as_ref()) {
            (Some(self_rd), Some(other_rd)) => {
                self_rd.section == other_rd.section && self_rd.command_id == other_rd.command_id
            }
            _ => {
                self.command_name == other.command_name
            }
        }
    }
}

impl Action {
    pub(super) fn command_name_based(command_name: CString) -> Action {
        Action {
            command_name: Some(command_name),
            runtime_data: RefCell::new(None),
        }
    }

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
        if let Some(cached_index) = self.load_if_necessary_or_complain().cached_index {
            return cached_index;
        }
        let mut opt_runtime_data = self.runtime_data.borrow_mut();
        let mut runtime_data = opt_runtime_data.as_mut().unwrap();
        let index = self.find_index(runtime_data).expect("Index couldn't be found");
        runtime_data.cached_index = Some(index);
        index
    }

    pub fn get_section(&self) -> Section {
        let rd = self.load_if_necessary_or_complain();
        rd.section
    }

    fn find_index(&self, runtime_data: &RuntimeData) -> Option<u32> {
        // TODO Use kbd_enumerateActions
        runtime_data.section.get_kbd_cmds().enumerate()
            .find(|(i, kbd_cmd)| {
                kbd_cmd.cmd as i64 == runtime_data.command_id
            })
            .map(|(i, _)| i as u32)
    }

    pub fn is_available(&self) -> bool {
        if let Some(runtime_data) = self.runtime_data.borrow().as_ref() {
            // See if we can get a description. If yes, the action actually exists. If not, then not.
            let text = Reaper::instance().medium
                .kbd_get_text_from_cmd(runtime_data.command_id as u32, runtime_data.section.get_raw_section_info());
            return text.filter(|t| t.to_bytes().len() > 0).is_some();
        }
        self.load_by_command_name()
    }

    pub fn get_character(&self) -> ActionCharacter {
        let rd = self.load_if_necessary_or_complain();
        if Reaper::instance().medium.get_toggle_command_state_2(rd.section.get_raw_section_info(), rd.command_id as i32) == -1 {
            ActionCharacter::Trigger
        } else {
            ActionCharacter::Toggle
        }
    }

    pub fn is_on(&self) -> bool {
        let rd = self.load_if_necessary_or_complain();
        Reaper::instance().medium.get_toggle_command_state_2(rd.section.get_raw_section_info(), rd.command_id as i32) == 1
    }

    // TODO "ParameterType" is not a good name for that
    pub fn get_parameter_type(&self) -> ParameterType {
        ParameterType::Action
    }

    pub fn get_command_id(&self) -> i64 {
        let rd = self.load_if_necessary_or_complain();
        rd.command_id
    }

    pub fn get_command_name(&self) -> Option<&CStr> {
        self.command_name.as_ref().map(|cn| cn.as_c_str()).or_else(|| {
            let rd = self.load_if_necessary_or_complain();
            Reaper::instance().medium
                .reverse_named_command_lookup(rd.command_id as i32)
        })
    }

    // Returns None if action disappeared TODO This is not consequent
    pub fn get_name(&self) -> Option<&CStr> {
        let rd = self.load_if_necessary_or_complain();
        Reaper::instance().medium
            .kbd_get_text_from_cmd(rd.command_id as u32, rd.section.get_raw_section_info())
    }

    pub fn invoke_as_trigger(&self, project: Option<Project>) {
        self.invoke(1.0, false, project)
    }

    pub fn invoke(&self, normalized_value: f64, is_step_count: bool, project: Option<Project>) {
        // TODO I have no idea how to launch an action in a specific section. The first function doesn't seem to launch the action :(
        // bool (*kbd_RunCommandThroughHooks)(KbdSectionInfo* section, int* actionCommandID, int* val, int* valhw, int* relmode, HWND hwnd);
        // int (*KBD_OnMainActionEx)(int cmd, int val, int valhw, int relmode, HWND hwnd, ReaProject* proj);
        let rd = self.load_if_necessary_or_complain();
        let action_command_id = rd.command_id;
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

    fn load_if_necessary_or_complain(&self) -> Ref<RuntimeData> {
        if (self.runtime_data.borrow().is_none() && self.load_by_command_name()) {
            panic!("Action not loadable")
        }
        Ref::map(self.runtime_data.borrow(), |rd| rd.as_ref().unwrap())
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