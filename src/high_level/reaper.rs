use crate::medium_level;
use crate::bindings::{ReaProject, MediaTrack, gaccel_register_t, ACCEL};
use std::ptr::{null_mut, null};
use std::os::raw::{c_void, c_ushort};
use c_str_macro::c_str;
use std::ffi::{CStr, CString};
use std::borrow::{Cow, Borrow, BorrowMut};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::{RefCell, Ref, RefMut};
use std::sync::Once;
use crate::high_level::ActionKind::Toggleable;
use crate::high_level::Project;

// We can't use Once because we need multiple writes (not just for initialization).
// We use thread_local instead of Mutex to have less overhead because we know this is accessed
// from one thread only.
// However, we still need to use RefCell, otherwise we don't get (interior) mutability with
// thread_local.
thread_local! {
    static MAIN_THREAD_STATE: RefCell<MainThreadState> = RefCell::new(MainThreadState {
            command_by_index: HashMap::new(),
    });
}

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut INSTALLED_REAPER: Option<Reaper> = None;
static INIT_INSTALLED_REAPER: Once = Once::new();

struct MainThreadState {
    command_by_index: HashMap<i32, Command>,
}

// Only for main section
fn hook_command(command_index: i32, flag: i32) -> bool {
    MAIN_THREAD_STATE.with(|mut state| {
        let mut state = state.borrow_mut();
        let command = match state.command_by_index.get_mut(&command_index) {
            None => return false,
            Some(c) => c
        };
        command.operation.call_mut(());
        true
    })
}


// Only for main section
fn toggle_action(command_index: i32) -> i32 {
    MAIN_THREAD_STATE.with(|state| {
        let state = state.borrow();
        let command = match state.command_by_index.get(&command_index) {
            None => return -1,
            Some(c) => c
        };
        match &command.kind {
            ActionKind::Toggleable(is_on) => if is_on() { 1 } else { 0 },
            ActionKind::NotToggleable => -1
        }
    })
}



pub struct Reaper {
    pub medium: medium_level::Reaper,
}

pub enum ActionKind {
    NotToggleable,
    Toggleable(Box<dyn Fn() -> bool>),
}

pub fn toggleable(is_on: impl Fn() -> bool + 'static) -> ActionKind {
    Toggleable(Box::new(is_on))
}

impl Reaper {
    pub fn setup(medium: medium_level::Reaper) {
        let reaper = Reaper {
            medium,
        };
        reaper.medium.plugin_register(c_str!("hookcommand"), hook_command as *mut c_void);
        reaper.medium.plugin_register(c_str!("toggleaction"), toggle_action as *mut c_void);
        unsafe {
            INIT_INSTALLED_REAPER.call_once(|| {
                INSTALLED_REAPER = Some(reaper);
            });
        }
    }

    pub fn instance() -> &'static Reaper {
        unsafe {
            INSTALLED_REAPER.as_ref().unwrap()
        }
    }

    pub fn register_action(
        &self,
        command_id: &CStr,
        description: impl Into<Cow<'static, CStr>>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    ) -> RegisteredAction
    {
        let command_index = self.medium.plugin_register(c_str!("command_id"), command_id.as_ptr() as *mut c_void);
        let command = Command::new(command_index, description.into(), Box::new(operation), kind);
        self.register_command(command_index, command);
        RegisteredAction::new(command_index)
    }

    fn register_command(&self, command_index: i32, command: Command) {
        MAIN_THREAD_STATE.with(|state| {
            if let Entry::Vacant(p) = state.borrow_mut().command_by_index.entry(command_index) {
                let command = p.insert(command);
                let acc = &mut command.accelerator_register;
                self.medium.plugin_register(c_str!("gaccel"), acc as *mut _ as *mut c_void);
            }
        });
    }

    fn unregister_command(&self, command_index: i32) {
        // TODO Use RAII
        MAIN_THREAD_STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(command) = state.command_by_index.get_mut(&command_index) {
                let acc = &mut command.accelerator_register;
                self.medium.plugin_register(c_str!("-gaccel"), acc as *mut _ as *mut c_void);
                state.command_by_index.remove(&command_index);
            }
        });
    }

    pub fn show_console_msg(&self, msg: &CStr) {
        self.medium.show_console_msg(msg);
    }

    pub fn get_current_project(&self) -> Project {
        let (rp, _) = self.medium.enum_projects(-1, 0);
        Project::new(rp)
    }

    pub fn get_projects(&self) -> impl Iterator<Item=Project> + '_ {
        (0..)
            .map(move |i| self.medium.enum_projects(i, 0).0)
            .take_while(|p| !p.is_null())
            .map(|p| { Project::new(p) })
    }

    pub fn get_project_count(&self) -> i32 {
        self.get_projects().count() as i32
    }
}

struct Command {
    description: Cow<'static, CStr>,
    operation: Box<dyn FnMut()>,
    kind: ActionKind,
    accelerator_register: gaccel_register_t,
}

impl Command {
    fn new(command_index: i32, description: Cow<'static, CStr>, operation: Box<dyn FnMut()>, kind: ActionKind) -> Command {
        let mut c = Command {
            description,
            operation,
            kind,
            accelerator_register: gaccel_register_t {
                accel: ACCEL {
                    fVirt: 0,
                    key: 0,
                    cmd: command_index as c_ushort,
                },
                desc: null(),
            },
        };
        c.accelerator_register.desc = c.description.as_ptr();
        c
    }
}

pub struct RegisteredAction {
    command_index: i32,
}

impl RegisteredAction {
    fn new(command_index: i32) -> RegisteredAction {
        RegisteredAction {
            command_index,
        }
    }

    pub fn unregister(&self) {
        Reaper::instance().unregister_command(self.command_index);
    }
}