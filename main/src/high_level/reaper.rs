use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void};
use std::ptr::{null, null_mut};
use std::sync::Once;

use c_str_macro::c_str;

use crate::high_level::ActionKind::Toggleable;
use crate::high_level::Project;
use crate::low_level::{ACCEL, gaccel_register_t, MediaTrack, ReaProject};
use crate::low_level;
use crate::medium_level;
use rxrust::observable::Observable;
use rxrust::subscriber::Subscriber;
use crate::high_level::helper_control_surface::HelperControlSurface;
use rxrust::subscription::SubscriptionLike;
use rxrust::observer::Observer;
use rxrust::prelude::*;
use rxrust::subject::{LocalSubjectObserver, SubjectValue};

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut REAPER_INSTANCE: Option<Reaper> = None;
static INIT_REAPER_INSTANCE: Once = Once::new();

// Only for main section
fn hook_command(command_index: i32, flag: i32) -> bool {
    if let Some(command) = Reaper::instance().command_by_index.borrow().get(&command_index) {
        command.operation.borrow_mut().call_mut(());
        true
    } else {
        false
    }
}


// Only for main section
fn toggle_action(command_index: i32) -> i32 {
    if let Some(command) = Reaper::instance().command_by_index.borrow().get(&command_index) {
        match &command.kind {
            ActionKind::Toggleable(is_on) => if is_on() { 1 } else { 0 },
            ActionKind::NotToggleable => -1
        }
    } else {
        -1
    }
}


pub struct Reaper {
    pub medium: medium_level::Reaper,
    // We take a mutable reference from this RefCell in order to add/remove commands.
    // TODO Adding an action in an action would panic because we have an immutable borrow of the map
    //  to obtain and execute the command, plus a mutable borrow of the map to add the new command.
    //  (the latter being unavoidable because we somehow need to modify the map!).
    //  That's not good. Is there a way to avoid this constellation? It's probably hard to avoid the
    //  immutable borrow because the `operation` is part of the map after all. And we can't just
    //  copy it before execution, at least not when it captures and mutates state, which might not
    //  be copyable (which we want to explicitly allow, that's why we accept FnMut!). Or is it
    //  possible to give up the map borrow after obtaining the command/operation reference???
    //  Look into that!!!
    command_by_index: RefCell<HashMap<i32, Command>>,
    // This is a RefCell. So calling next() while another next() is still running will panic.
    // I guess it's good that way because this is very generic code, panicking or not panicking
    // depending on the user's code. And getting a panic is good for becoming aware of the problem
    // instead of running into undefined behavior. The developer can always choose to defer to
    // the next `ControlSurface::run()` invocation (execute things in next main loop cycle).
    pub(super) dummy_subject: EventStreamSubject<i32>,
    pub(super) project_switched_subject: EventStreamSubject<Project>,
}

pub enum ActionKind {
    NotToggleable,
    Toggleable(Box<dyn Fn() -> bool>),
}

pub fn toggleable(is_on: impl Fn() -> bool + 'static) -> ActionKind {
    Toggleable(Box::new(is_on))
}

type EventStreamSubject<T> = RefCell<EventStream<T>>;
type EventStream<T> = LocalSubject<'static, SubjectValue<T>, SubjectValue<()>>;

impl Reaper {
    pub fn setup(medium: medium_level::Reaper) {
        let reaper = Reaper {
            medium,
            command_by_index: RefCell::new(HashMap::new()),
            dummy_subject: RefCell::new(Subject::local()),
            project_switched_subject: RefCell::new(Subject::local()),
        };
        unsafe {
            INIT_REAPER_INSTANCE.call_once(|| {
                REAPER_INSTANCE = Some(reaper);
            });
        }
        Reaper::instance().init();
    }

    fn init(&self) {
        self.medium.plugin_register(c_str!("hookcommand"), hook_command as *mut c_void);
        self.medium.plugin_register(c_str!("toggleaction"), toggle_action as *mut c_void);
        self.medium.install_control_surface(HelperControlSurface::new());
        self.medium.register_control_surface();
    }

    // Allowing global access to native REAPER functions at all times is valid in my opinion.
    // Because REAPER itself is not written in Rust and therefore cannot take part in Rust's compile
    // time guarantees anyway. We need to rely on REAPER at that point and also take care not to do
    // something which is not allowed in Reaper (by reading documentation and learning from
    // mistakes ... no compiler is going to save us from them). REAPER as a whole is always mutable
    // from the perspective of extensions.
    //
    // We express that in Rust by making `Reaper` class an immutable (in the sense of non-`&mut`)
    // singleton and allowing all REAPER functions to be called from an immutable context ...
    // although they can and often will lead to mutations within REAPER!
    pub fn instance() -> &'static Reaper {
        unsafe {
            REAPER_INSTANCE.as_ref().unwrap()
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
        let command = Command::new(command_index, description.into(), RefCell::new(Box::new(operation)), kind);
        self.register_command(command_index, command);
        RegisteredAction::new(command_index)
    }

    fn register_command(&self, command_index: i32, command: Command) {
        if let Entry::Vacant(p) = self.command_by_index.borrow_mut().entry(command_index) {
            let command = p.insert(command);
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("gaccel"), acc as *mut _ as *mut c_void);
        }
    }

    fn unregister_command(&self, command_index: i32) {
        // TODO Use RAII
        if let Some(command) = self.command_by_index.borrow_mut().get_mut(&command_index) {
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("-gaccel"), acc as *mut _ as *mut c_void);
            self.command_by_index.borrow_mut().remove(&command_index);
        }
    }

    pub fn show_console_msg(&self, msg: &CStr) {
        self.medium.show_console_msg(msg);
    }

    pub fn dummy_event_invoked(&self) -> EventStream<i32> {
        self.dummy_subject.borrow().fork()
    }

    pub fn project_switched(&self) -> EventStream<Project> {
        self.project_switched_subject.borrow().fork()
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
    operation: RefCell<Box<dyn FnMut()>>,
    kind: ActionKind,
    accelerator_register: gaccel_register_t,
}

impl Command {
    fn new(command_index: i32, description: Cow<'static, CStr>, operation: RefCell<Box<dyn FnMut()>>, kind: ActionKind) -> Command {
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