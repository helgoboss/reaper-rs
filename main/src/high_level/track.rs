use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::Once;

use c_str_macro::c_str;

use crate::high_level::{Project, Reaper};
use crate::high_level::ActionKind::Toggleable;
use crate::high_level::guid::Guid;
use crate::low_level::{MediaTrack, ReaProject};
use crate::medium_level;

/// The difference to Track is that this implements Copy (not just Clone)
// TODO Maybe it's more efficient to use a moving or copying pointer for track Observables? Anyway,
//  this would require rxRust subjects to work with elements that are not copyable (because Rc,
//  RefCell, Box, Arc and all that stuff are never copyable) but just cloneable
#[derive(Clone, Copy, Debug)]
pub struct LightTrack {
    media_track: *mut MediaTrack,
    rea_project: *mut ReaProject,
    guid: Guid,
}

impl LightTrack {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions < 5.95
    pub fn new(media_track: *mut MediaTrack, rea_project: *mut ReaProject) -> LightTrack {
        LightTrack {
            media_track,
            rea_project: {
                if rea_project.is_null() {
                    get_media_track_rea_project(media_track)
                } else {
                    rea_project
                }
            },
            // We load the GUID eagerly because we want to make comparability possible even in the following case:
            // Track A has been initialized with a GUID not been loaded yet, track B has been initialized with a MediaTrack*
            // (this constructor) but has rendered invalid in the meantime. Now there would not be any way to compare them
            // because I can neither compare MediaTrack* pointers nor GUIDs. Except I extract the GUID eagerly.
            guid: get_media_track_guid(media_track),
        }
    }
}

// TODO Think hard about what equality means here!
#[derive(Clone, Debug, PartialEq, Eq)]
// TODO Add Copy again and remove LightTrack if possible one day, see https://github.com/rust-lang/rust/issues/20813
// TODO Reconsider design. Maybe don't do that interior mutability stuff. By moving from lazy to
//  eager (determining rea_project and media_track at construction time).
pub struct Track {
    // Only filled if track loaded.
    media_track: Cell<*mut MediaTrack>,
    // TODO Do we really need this pointer? Makes copying a tiny bit more expensive than just copying a MediaTrack*.
    rea_project: Cell<*mut ReaProject>,
    // Possible states:
    // a) guid, project, !mediaTrack (guid-based and not yet loaded)
    // b) guid, mediaTrack (guid-based and loaded)
    // TODO This is not super cheap to copy. Do we really need to initialize this eagerly?
    guid: Guid,
}

impl From<LightTrack> for Track {
    fn from(light: LightTrack) -> Self {
        Track {
            media_track: Cell::new(light.media_track),
            rea_project: Cell::new(light.rea_project),
            guid: light.guid
        }
    }
}

impl Track {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions < 5.95
    pub fn new(media_track: *mut MediaTrack, rea_project: *mut ReaProject) -> Track {
        Track {
            media_track: Cell::new(media_track),
            rea_project: {
                let actual = if rea_project.is_null() {
                    get_media_track_rea_project(media_track)
                } else {
                    rea_project
                };
                Cell::new(actual)
            },
            // We load the GUID eagerly because we want to make comparability possible even in the following case:
            // Track A has been initialized with a GUID not been loaded yet, track B has been initialized with a MediaTrack*
            // (this constructor) but has rendered invalid in the meantime. Now there would not be any way to compare them
            // because I can neither compare MediaTrack* pointers nor GUIDs. Except I extract the GUID eagerly.
            guid: get_media_track_guid(media_track),
        }
    }

    pub fn get_name(&self) -> CString {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.convenient_get_media_track_info_string(self.get_media_track(), c_str!("P_NAME"))
    }

    pub fn get_media_track(&self) -> *mut MediaTrack {
        self.load_if_necessary_or_complain();
        self.media_track.get()
    }

    // TODO Maybe return u32 and express master track index in other ways
    pub fn get_index(&self) -> i32 {
        self.load_and_check_if_necessary_or_complain();
        let ip_track_number = Reaper::instance().medium.convenient_get_media_track_info_i32(self.get_media_track(), c_str!("IP_TRACKNUMBER"));
        if ip_track_number == 0 {
            // Usually means that track doesn't exist. But this we already checked. This happens only if we query the
            // number of a track in another project tab. TODO Try to find a working solution. Till then, return 0.
            return 0;
        }
        if ip_track_number == -1 {
            // Master track indicator
            return -1;
        }
        // Must be > 0. Make it zero-rooted.
        ip_track_number - 1
    }

    fn load_and_check_if_necessary_or_complain(&self) {
        self.load_if_necessary_or_complain();
        self.complain_if_not_valid();
    }

    fn load_if_necessary_or_complain(&self) {
        if self.media_track.get().is_null() && !self.load_by_guid() {
            panic!("Track not loadable");
        }
    }

    fn complain_if_not_valid(&self) {
        if !self.is_valid() {
            panic!("Track not available");
        }
    }

    // Precondition: mediaTrack_ must be filled!
    fn is_valid(&self) -> bool {
        if self.media_track.get().is_null() {
            panic!("Track can not be validated if mediaTrack not available");
        }
        self.attempt_to_fill_project_if_necessary();
        if self.rea_project.get().is_null() {
            false
        } else {
            if Project::new(self.rea_project.get()).is_available() {
                Reaper::instance().medium.validate_ptr_2(self.rea_project.get(), self.media_track.get() as *mut c_void, c_str!("MediaTrack*"))
            } else {
                false
            }
        }
    }

    // Precondition: mediaTrack_ must be filled!
    fn attempt_to_fill_project_if_necessary(&self) {
        if self.rea_project.get().is_null() {
            self.rea_project.replace(self.find_containing_project());
        }
    }

    fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn load_by_guid(&self) -> bool {
        if self.rea_project.get().is_null() {
            panic!("For loading per GUID, a project must be given");
        }
        // TODO Don't save ReaProject but Project as member
        let guid = self.get_guid();
        let track = self.unchecked_project().get_tracks()
            .find(|t| t.get_guid() == guid);
        match track {
            Some(t) => {
                self.media_track.replace(t.get_media_track());
                true
            }
            None => {
                self.media_track.replace(null_mut());
                false
            }
        }
    }

    fn unchecked_project(&self) -> Project {
        self.attempt_to_fill_project_if_necessary();
        Project::new(self.rea_project.get())
    }

    // Precondition: mediaTrack_ must be filled!
    fn find_containing_project(&self) -> *mut ReaProject {
        if self.media_track.get().is_null() {
            panic!("Containing project cannot be found if mediaTrack not available");
        }
        // No ReaProject* available. Try current project first (most likely in everyday REAPER usage).
        let reaper = Reaper::instance();
        let current_project = reaper.get_current_project();
        // TODO Add convenience functions to medium API for checking various pointer types
        let is_valid_in_current_project = reaper.medium.validate_ptr_2(
            current_project.get_rea_project(),
            self.media_track.get() as *mut c_void,
            c_str!("MediaTrack*"),
        );
        if is_valid_in_current_project {
            return current_project.get_rea_project();
        }
        // Worst case. It could still be valid in another project. We have to check each project.
        let other_project = reaper.get_projects()
            // We already know it's invalid in current project
            .filter(|p| p != &current_project)
            .find(|p|
                reaper.medium.validate_ptr_2(
                    p.get_rea_project(),
                    self.media_track.get() as *mut c_void,
                    c_str!("MediaTrack*"),
                )
            );
        other_project.map(|p| p.get_rea_project()).unwrap_or(null_mut())
    }
}

pub fn get_media_track_guid(media_track: *mut MediaTrack) -> Guid {
    let internal = Reaper::instance().medium.convenient_get_media_track_info_guid(media_track, c_str!("GUID"));
    Guid::new(unsafe { *internal })
}

// In REAPER < 5.95 this returns nullptr. That means we might need to use findContainingProject logic at a later
// point.
fn get_media_track_rea_project(media_track: *mut MediaTrack) -> *mut ReaProject {
    Reaper::instance().medium.get_set_media_track_info(media_track, c_str!("P_PROJECT"), null_mut()) as *mut ReaProject
}