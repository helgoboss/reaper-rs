use crate::{ActionCharacter, Project, Reaper, Section};
use c_str_macro::c_str;
use reaper_medium::{ActionValueChange, CommandId, ReaperStr, ReaperString, SectionContext};

use helgoboss_midi::{U14, U7};
use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::SectionContext::Sec;
use reaper_medium::WindowContext::Win;
use std::borrow::Cow;
use std::cell::{Ref, RefCell};

use std::ffi::CString;

#[derive(Debug, Copy, Clone)]
struct RuntimeData {
    section: Section,
    // Sometimes shortly named cmd in REAPER API. Unique within section. Might be filled lazily.
    // For built-in actions this ID is globally stable and will always be found. For custom
    // actions, this ID is only stable at runtime and it might be that it can't be found -
    // which means the action is not available.
    command_id: CommandId,
    cached_index: Option<u32>,
}

// TODO-low Use separate classes for loaded and not loaded actions
#[derive(Debug, Clone)]
pub struct Action {
    runtime_data: RefCell<Option<RuntimeData>>,
    // Used to represent custom actions that are not available (they don't have a commandId) or for
    // which is not yet known if they are available. Globally unique, not within one section.
    // TODO-low But currently only mainSection supported. How support other sections?
    command_name: Option<ReaperString>,
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
    pub(super) fn command_name_based(command_name: ReaperString) -> Action {
        Action {
            command_name: Some(command_name),
            runtime_data: RefCell::new(None),
        }
    }

    pub(super) fn new(section: Section, command_id: CommandId, index: Option<u32>) -> Action {
        Action {
            command_name: None,
            runtime_data: RefCell::new(Some(RuntimeData {
                section,
                command_id,
                cached_index: index,
            })),
        }
    }

    pub fn index(&self) -> u32 {
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

    pub fn section(&self) -> Section {
        let rd = self.load_if_necessary_or_complain();
        rd.section
    }

    fn find_index(&self, runtime_data: &RuntimeData) -> Option<u32> {
        // TODO-low Use kbd_enumerateActions
        let section_id = runtime_data.section.id();
        Reaper::get()
            .medium_reaper()
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
                    Reaper::get().medium_reaper().kbd_get_text_from_cmd(
                        runtime_data.command_id,
                        Sec(s),
                        |_| (),
                    )
                })
                .flatten();
            return result.is_some();
        }
        self.load_by_command_name()
    }

    pub fn character(&self) -> ActionCharacter {
        let rd = self.load_if_necessary_or_complain();
        let state = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_toggle_command_state_2(Sec(&rd.section.raw()), rd.command_id)
        };
        match state {
            Some(_) => ActionCharacter::Toggle,
            None => ActionCharacter::Trigger,
        }
    }

    /// reaper-rs listens to MIDI CC/mousewheel action invocations since REAPER 6.19+dev1226
    /// and tracks their latest *absolute* value invocations if there are any. This returns the
    /// latest absolute value *if there is any*. Only works for main section.
    pub fn normalized_value(&self) -> Option<f64> {
        let rd = self.load_if_necessary_or_complain();
        let last_change = Reaper::get().find_last_action_value_change(rd.command_id)?;
        use ActionValueChange::*;
        let normalized_value = match last_change {
            AbsoluteLowRes(v) => v.get() as f64 / U7::MAX.get() as f64,
            AbsoluteHighRes(v) => v.get() as f64 / U14::MAX.get() as f64,
            _ => return None,
        };
        Some(normalized_value)
    }

    pub fn is_on(&self) -> Option<bool> {
        let rd = self.load_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_toggle_command_state_2(Sec(&rd.section.raw()), rd.command_id)
        }
    }

    pub fn command_id(&self) -> CommandId {
        let rd = self.load_if_necessary_or_complain();
        rd.command_id
    }

    // TODO-low Don't copy into string. Split into command-based action and command-id based action
    //  and then return Option<&CStr> for the first (with same lifetime like self) and
    //  ReaperStringPtr alternative for the second
    pub fn command_name(&self) -> Option<ReaperString> {
        self.command_name.clone().or_else(|| {
            let rd = self.runtime_data.borrow();
            Reaper::get()
                .medium_reaper()
                .reverse_named_command_lookup(rd.as_ref().unwrap().command_id, |s| {
                    s.to_reaper_string()
                })
        })
    }

    pub fn name(&self) -> ReaperString {
        let rd = self.load_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .kbd_get_text_from_cmd(rd.command_id, SectionContext::Sec(&rd.section.raw()), |s| {
                    s.to_owned()
                })
                .expect("action not existing")
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
        let reaper = Reaper::get().medium_reaper();
        if is_step_count {
            let relative_value = 64 + normalized_value as i32;
            let cropped_relative_value =
                unsafe { U7::new_unchecked(relative_value.clamp(0, 127) as u8) };
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val,
            // &valhw, &relmode, reaper::GetMainHwnd());
            unsafe {
                reaper.kbd_on_main_action_ex(
                    action_command_id,
                    ActionValueChange::Relative2(cropped_relative_value),
                    Win(reaper.get_main_hwnd()),
                    match project {
                        None => CurrentProject,
                        Some(p) => Proj(p.raw()),
                    },
                );
            }
        } else {
            // reaper::kbd_RunCommandThroughHooks(section_.sectionInfo(), &actionCommandId, &val,
            // &valhw, &relmode, reaper::GetMainHwnd());
            let discrete_value = unsafe {
                U14::new_unchecked((normalized_value * U14::MAX.get() as f64).round() as u16)
            };
            unsafe {
                reaper.kbd_on_main_action_ex(
                    action_command_id,
                    ActionValueChange::AbsoluteHighRes(discrete_value),
                    Win(reaper.get_main_hwnd()),
                    match project {
                        None => CurrentProject,
                        Some(p) => Proj(p.raw()),
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
        let command_id = match Reaper::get()
            .medium_reaper()
            .named_command_lookup(fixed_command_name.as_ref())
        {
            None => return false,
            Some(id) => id,
        };
        self.runtime_data.replace(Some(RuntimeData {
            section: Reaper::get().main_section(),
            command_id,
            cached_index: None,
        }));
        true
    }

    fn fix_command_name(command_name: &ReaperStr) -> Cow<ReaperStr> {
        let bytes = command_name.as_c_str().to_bytes();
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
        Cow::from(unsafe { ReaperString::new_unchecked(with_underscore) })
    }
}

fn contains_digits_only(command_name: &ReaperStr) -> bool {
    let digit_regex = regex!("[^0-9]");
    digit_regex.find(command_name.to_str()).is_none()
}
