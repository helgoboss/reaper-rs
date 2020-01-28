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
use crate::high_level::Reaper;

pub struct Track {
    media_track: *mut MediaTrack,
    rea_project: *mut ReaProject,
}

impl Track {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions < 5.95
    pub fn new(media_track: *mut MediaTrack, rea_project: *mut ReaProject) -> Track {
        Track { media_track, rea_project }
    }

    pub fn get_name(&self) -> String {
        Reaper::instance().medium.convenient_get_media_track_info_string(self.get_media_track(), c_str!("P_NAME"))
    }

    pub fn get_media_track(&self) -> *mut MediaTrack {
        self.media_track
    }
}