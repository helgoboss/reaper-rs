use crate::{ActionCharacter, Project, Reaper, Section};
use c_str_macro::c_str;
use reaper_rs_medium::{ActionValueChange, SectionContext};

use helgoboss_midi::U7;
use reaper_rs_medium::ProjectContext::{CurrentProject, Proj};
use reaper_rs_medium::SectionContext::Sec;
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;

#[derive(Debug, Copy, Clone)]
struct RuntimeData {
    section: Section,
    // Sometimes shortly named cmd in REAPER API. Unique within section. Might be filled lazily.
    // For built-in actions this ID is globally stable and will always be found. For custom
    // actions, this ID is only stable at runtime and it might be that it can't be found -
    // which means the action is not available.
    command_id: u32,
    cached_index: Option<u32>,
}

// TODO-low Use separate classes for loaded and not loaded actions
#[derive(Debug, Clone)]
pub struct Action {
    runtime_data: RefCell<Option<RuntimeData>>,
    // Used to represent custom actions that are not available (they don't have a commandId) or for
    // which is not yet known if they are available. Globally unique, not within one section.
    // TODO-low But currently only mainSection supported. How support other sections?
    command_name: Option<CString>,
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.runtime_data.borrow().as_ref(),
            other.runtime_data.borrow().as_ref(),
        ) {
            (Some(self_rd), Some(other_rd)) => {
                self_rd.section == other_rd.section && self_rd.command_id == other_rd.command_id
            }
            _ => self.command_name == other.command_name,
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

    pub(super) fn new(section: Section, command_id: u32, index: Option<u32>) -> Action {
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
        let index = self
            .find_index(runtime_data)
            .expect("Index couldn't be found");
        runtime_data.cached_index = Some(index);
        index
    }

    pub fn get_section(&self) -> Section {
        let rd = self.load_if_necessary_or_complain();
        rd.section
    }

    fn find_index(&self, runtime_data: &RuntimeData) -> Option<u32> {
        // TODO-low Use kbd_enumerateActions
        let section_id = runtime_data.section.id();
        Reaper::get()
            .medium
            .section_from_unique_id(section_id, |s| {
                (0..s.action_list_cnt())
                    .map(move |i| s.get_action_by_index(i).unwrap())
                    .enumerate()
                    .find(|(_, kbd_cmd)| kbd_cmd.cmd() == runtime_data.command_id)
                    .map(|(i, _)| i as u32)
            })
            .unwrap()
    }

    pub fn is_available(&self) -> bool {
        if let Some(runtime_data) = self.runtime_data.borrow().as_ref() {
            // See if we can get a description. If yes, the action actually exists. If not, then
            // not.
            let result = runtime_data
                .section
                .with_raw(|s| unsafe {
                    Reaper::get().medium.kbd_get_text_from_cmd(
                        runtime_data.command_id as u32,
                        Sec(s),
                        |_| (),
                    )
                })
                .flatten();
            return result.is_some();
        }
        self.load_by_command_name()
    }

    pub fn get_character(&self) -> ActionCharacter {
        let rd = self.load_if_necessary_or_complain();
        let state = unsafe {
            Reaper::get()
                .medium
                .get_toggle_command_state_2(Sec(&rd.section.get_raw()), rd.command_id)
        };
        match state {
            Some(_) => ActionCharacter::Toggle,
            None => ActionCharacter::Trigger,
        }
    }

    pub fn is_on(&self) -> bool {
        let rd = self.load_if_necessary_or_complain();
        let state = unsafe {
            Reaper::get()
                .medium
                .get_toggle_command_state_2(Sec(&rd.section.get_raw()), rd.command_id)
        };
        state == Some(true)
    }

    pub fn get_command_id(&self) -> u32 {
        let rd = self.load_if_necessary_or_complain();
        rd.command_id
    }

    // TODO-low Don't copy into string. Split into command-based action and command-id based action
    //  and then return Option<&CStr> for the first (with same lifetime like self) and
    //  ReaperStringPtr alternative for the second
    pub fn get_command_name(&self) -> Option<CString> {
        self.command_name
            .as_ref()
            .map(|cn| cn.as_c_str().to_owned())
            .or_else(|| {
                let rd = self.runtime_data.borrow();
                Reaper::get()
                    .medium
                    .reverse_named_command_lookup(rd.as_ref().unwrap().command_id, |s| s.into())
            })
    }

    pub fn get_name(&self) -> Option<CString> {
        let rd = self.load_if_necessary_or_complain();
        unsafe {
            Reaper::get().medium.kbd_get_text_from_cmd(
                rd.command_id as u32,
                SectionContext::Sec(&rd.section.get_raw()),
                |s| s.into(),
            )
        }
    }

    pub fn invoke_as_trigger(&self, project: Option<Project>) {
        self.invoke(1.0, false, project)
    }

    pub fn invoke(&self, normalized_value: f64, is_step_count: bool, project: Option<Project>) {
        // TODO-low I have no idea how to launch an action in a specific section. The first function
        // doesn't seem to launch the action :(
        // bool (*kbd_RunCommandThroughHooks)(KbdSectionInfo* section, int* actionCommandID, int*
        // val, int* valhw, int* relmode, HWND hwnd); int (*KBD_OnMainActionEx)(int cmd, int
        // val, int valhw, int relmode, HWND hwnd, ReaProject* proj);
        let rd = self.load_if_necessary_or_complain();
        let action_command_id = rd.command_id;
        let reaper = Reaper::get();
        if is_step_count {
            let relative_value = 64 + normalized_value as i32;
            let cropped_relative_value =
                unsafe { U7::new_unchecked(relative_value.clamp(0, 127) as u8) };
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val,
            // &valhw, &relmode, reaper::GetMainHwnd());
            unsafe {
                reaper.medium.kbd_on_main_action_ex(
                    action_command_id,
                    ActionValueChange::Relative2(cropped_relative_value),
                    Some(reaper.medium.get_main_hwnd()),
                    match project {
                        None => CurrentProject,
                        Some(p) => Proj(p.get_raw()),
                    },
                );
            }
        } else {
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val,
            // &valhw, &relmode, reaper::GetMainHwnd());
            let discrete_value =
                unsafe { U7::new_unchecked((normalized_value * 127 as f64).round() as u8) };
            unsafe {
                reaper.medium.kbd_on_main_action_ex(
                    action_command_id,
                    ActionValueChange::AbsoluteLowRes(discrete_value),
                    Some(reaper.medium.get_main_hwnd()),
                    match project {
                        None => CurrentProject,
                        Some(p) => Proj(p.get_raw()),
                    },
                );
            }
            // Main_OnCommandEx would trigger the actionInvoked event but it has not enough
            // parameters for passing values etc.          reaper::
            // Main_OnCommandEx(actionCommandId, 0, project ? project->reaProject() : nullptr);
        }
    }

    fn load_if_necessary_or_complain(&self) -> Ref<RuntimeData> {
        let is_loaded = self.runtime_data.borrow().is_none();
        if is_loaded && !self.load_by_command_name() {
            panic!("Action not loadable")
        }
        Ref::map(self.runtime_data.borrow(), |rd| rd.as_ref().unwrap())
    }

    fn load_by_command_name(&self) -> bool {
        let fixed_command_name =
            Self::fix_command_name(self.command_name.as_ref().expect("Command name not set"));
        let reaper = Reaper::get();
        let command_id = reaper
            .medium
            .named_command_lookup(fixed_command_name.as_ref());
        if command_id == 0 {
            return false;
        }
        self.runtime_data.replace(Some(RuntimeData {
            section: reaper.get_main_section(),
            command_id,
            cached_index: None,
        }));
        true
    }

    fn fix_command_name<'a>(command_name: &'a CStr) -> Cow<'a, CStr> {
        let bytes = command_name.to_bytes();
        if !bytes.len() == 0 && bytes[0] == b'_' {
            // Command already contains underscore. Great.
            return Cow::from(command_name);
        }
        if bytes.len() == 32 && contains_digits_only(command_name) {
            return Cow::from(command_name);
        }
        // Doesn't contain underscore but should contain one because it's a custom action or an
        // explicitly named command.
        let with_underscore = CString::new([c_str!("_").to_bytes(), bytes].concat()).unwrap();
        return Cow::from(with_underscore);
    }
}

fn contains_digits_only(command_name: &CStr) -> bool {
    let digit_regex = regex!("[^0-9]");
    digit_regex.find(command_name.to_str().unwrap()).is_none()
}
