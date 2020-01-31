use std::os::raw::c_void;
use crate::low_level::MediaTrack;
use crate::medium_level::ControlSurface;
use std::ffi::CStr;
use std::borrow::Cow;
use crate::high_level::{Reaper, Project};
use rxrust::prelude::*;
use std::cell::RefCell;

pub struct HelperControlSurface {
    last_active_project: RefCell<Project>
}

impl HelperControlSurface {
    pub fn new() -> HelperControlSurface {
        let reaper = Reaper::instance();
        HelperControlSurface {
            last_active_project: RefCell::new(reaper.get_current_project())
        }
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&self) {
//        println!("Hello from high-level control surface!")
    }

    fn set_track_list_change(&self) {
        let reaper = Reaper::instance();
        let new_active_project = reaper.get_current_project();
        if (new_active_project != *self.last_active_project.borrow()) {
            self.last_active_project.replace(new_active_project);
            reaper.project_switched_subject.borrow_mut().next(new_active_project);
        }
    }
}