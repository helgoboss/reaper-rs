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

thread_local! {
    static MAIN_THREAD_STATE: RefCell<MainThreadState> = RefCell::new(MainThreadState {
            command_by_index: HashMap::new(),
    });

    static INSTALLED_REAPER: RefCell<Option<Reaper>> = RefCell::new(None);
}

struct MainThreadState {
    command_by_index: HashMap<i32, Command>,
}

// Only for main section
fn static_hook_command(command_index: i32, flag: i32) -> bool {
    MAIN_THREAD_STATE.with(|state| {
        let state = state.borrow();
        let command = match state.command_by_index.get(&command_index) {
            None => return false,
            Some(c) => c
        };
        command.operation.borrow_mut().call_mut(());
        true
    })
}


pub struct InstalledReaper {
    pub reaper: Reaper
}

pub struct Reaper {
    pub medium: medium_level::Reaper,
}

pub enum ActionKind {
    NotToggleable,
    Toggleable(Box<dyn Fn() -> bool + 'static>),
}

impl Reaper {
    pub fn new(medium: medium_level::Reaper) -> Reaper {
        Reaper {
            medium,
        }
    }

    // Makes Reaper instance available globally using Reaper::with_installed().
    // Optional. If you have an appropriate owner mechanism already (e.g. in VST plugin),
    // you don't need this.
    pub fn install(reaper: Reaper) {
        reaper.medium.plugin_register(c_str!("hookcommand"), static_hook_command as *mut c_void);
        INSTALLED_REAPER.with(|r| {
           *r.borrow_mut() = Some(reaper);
        });
    }

    pub fn with_installed<T>(op: impl FnOnce(&Reaper) -> T) -> T {
        INSTALLED_REAPER.with(|r| {
           op(r.borrow().as_ref().unwrap())
        })
    }

    pub fn register_action(
        &self,
        command_id: &CStr,
        description: impl Into<Cow<'static, CStr>>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    )
//        -> RegisteredAction
    {
        let command_index = self.medium.plugin_register(c_str!("command_id"), command_id.as_ptr() as *mut c_void);
        let mut command = Command::new(command_index, description.into(), RefCell::new(Box::new(operation)), kind);
        self.register_command(command_index, command);
//        RegisteredAction::new(self, command_index)
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

    // TODO
//    fn unregister_command(&mut self, command_index: i32) {
//        // TODO Use RAII
//        if let Some(command) = self.command_by_index.get_mut(&command_index) {
//            let acc = &mut command.accelerator_register;
//            self.medium.plugin_register(c_str!("-gaccel"), acc as *mut _ as *mut c_void);
//            self.command_by_index.remove(&command_index);
//        }
//    }

    pub fn show_console_msg(&self, msg: &CStr) {
        self.medium.show_console_msg(msg);
    }

    pub fn get_current_project(&self) -> Project {
        let (rp, _) = self.medium.enum_projects(-1, 0);
        Project::new(&self.medium, rp)
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

pub struct RegisteredAction<'a> {
    reaper: &'a mut Reaper,
    command_index: i32,
}

//impl<'a> RegisteredAction<'a> {
//    fn new(reaper: &'a mut Reaper, command_index: i32) -> RegisteredAction {
//        RegisteredAction {
//            reaper,
//            command_index,
//        }
//    }
//
//    pub fn unregister(&mut self) {
//        self.reaper.unregister_command(self.command_index);
//    }
//}


pub struct Project<'a> {
    medium: &'a medium_level::Reaper,
    rea_project: *mut ReaProject,
}

impl<'a> Project<'a> {
    pub fn new(medium: &medium_level::Reaper, rea_project: *mut ReaProject) -> Project {
        Project { medium, rea_project }
    }

    pub fn get_first_track(&self) -> Option<Track> {
        self.get_track_by_index(0)
    }

    /// It's correct that this returns an Option because the index isn't a stable identifier of a
    /// track. The track could move. So this should do a runtime lookup of the track and return a
    /// stable MediaTrack-backed Some(Track) if a track exists at that index. 0 is first normal
    /// track (master track is not obtainable via this method).
    pub fn get_track_by_index(&self, idx: u32) -> Option<Track> {
        self.complain_if_not_available();
        let media_track = self.medium.get_track(self.rea_project, idx as i32);
        if media_track.is_null() {
            return None;
        }
        Some(Track::new(self.medium, media_track, self.rea_project))
    }

    pub fn is_available(&self) -> bool {
        self.medium.validate_ptr_2(null_mut(), self.rea_project as *mut c_void, c_str!("ReaProject*"))
    }

    fn complain_if_not_available(&self) {
        if !self.is_available() {
            panic!("Project not available")
        }
    }
}

pub struct Track<'a> {
    medium: &'a medium_level::Reaper,
    media_track: *mut MediaTrack,
    rea_project: *mut ReaProject,
}

impl<'a> Track<'a> {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions < 5.95
    pub fn new(medium: &medium_level::Reaper, media_track: *mut MediaTrack, rea_project: *mut ReaProject) -> Track {
        Track { medium, media_track, rea_project }
    }

    pub fn get_name(&self) -> String {
        self.medium.convenient_get_media_track_info_string(self.get_media_track(), c_str!("P_NAME"))
    }

    pub fn get_media_track(&self) -> *mut MediaTrack {
        self.media_track
    }
}