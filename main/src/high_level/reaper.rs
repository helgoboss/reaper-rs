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

// We can't use Once because we need multiple writes (not just for initialization).
// We use thread_local instead of Mutex to have less overhead because we know this is accessed
// from one thread only.
// However, we still need to use RefCell, otherwise we don't get (interior) mutability with
// thread_local.
thread_local! {
    // TODO We probably should move the RefCell more to the leafs in order to have less chances
    //  to have overlapping mutable references (e.g. a dedicated RefCell for command_by_index)
    //  and a dedicated one for the Command oepration in there (to execute it)

    // TODO It don't know anymore what's the point to have this thread_local if we make
    //  many fine-grained RefCells to get interior mutability. We could just slap this into the
    //  global REAPER_INSTANCE and be done with it. Easier to use (no "with") and by REAPER's
    //  architecture guaranteed to be called within main thread only anyway.
    static MAIN_THREAD_STATE: RefCell<MainThreadState> = RefCell::new(MainThreadState {
            command_by_index: HashMap::new(),
    });
}

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut REAPER_INSTANCE: Option<Reaper> = None;
static INIT_REAPER_INSTANCE: Once = Once::new();

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


#[derive()]
pub struct Reaper {
    pub medium: medium_level::Reaper,
    pub(super) dummy_subject: RefCell<LocalSubject<'static, SubjectValue<i32>, SubjectValue<()>>>
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
            dummy_subject: RefCell::new(Subject::local())
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

    pub fn project_switched1<O, U>(&self) -> Observable<impl FnOnce(Subscriber<O, U>) + Clone, Project, ()>
        where O: Observer<Project, ()>, U: SubscriptionLike {
        Observable::new(move |mut subscriber: Subscriber<O, U>| {
            for p in Reaper::instance().get_projects() {
                subscriber.next(p);
            }
            subscriber.complete();
        })
    }
    pub fn project_switched2<O, U>(&self) -> Subject<LocalSubjectObserver<'_, SubjectValue<i32>, ()>, LocalSubscription>
        where O: Observer<i32, ()>, U: SubscriptionLike {

        let mut s = Subject::local();
        s.next(5);
        s.fork()
    }

    pub fn project_switched3(&self) -> LocalSubject<SubjectValue<i32>, SubjectValue<()>> {
        let mut s = Subject::local();
        s.next(5);
        s
    }

    pub fn project_switched4(&self) -> LocalSubject<'static, SubjectValue<i32>, SubjectValue<()>> {
        self.dummy_subject.borrow().fork()
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