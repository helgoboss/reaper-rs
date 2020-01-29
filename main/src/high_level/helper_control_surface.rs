use std::os::raw::c_void;
use crate::low_level::MediaTrack;
use crate::medium_level::ControlSurface;
use std::ffi::CStr;
use std::borrow::Cow;
use crate::high_level::Reaper;
use rxrust::prelude::*;

pub struct HelperControlSurface {}

impl HelperControlSurface {
    pub fn new() -> HelperControlSurface {
        HelperControlSurface {}
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&self) {
//        println!("Hello from high-level control surface!")
    }

    fn set_track_list_change(&self) {
        Reaper::instance().dummy_subject.borrow_mut().next(42)
    }
}