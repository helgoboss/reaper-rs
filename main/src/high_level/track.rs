use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::Once;

use c_str_macro::c_str;
use slog::debug;

use rxrust::prelude::PayloadCopy;

use crate::high_level::automation_mode::AutomationMode;
use crate::high_level::fx::{get_index_from_query_index, Fx};
use crate::high_level::fx_chain::FxChain;
use crate::high_level::guid::Guid;
use crate::high_level::track_send::TrackSend;
use crate::high_level::ActionKind::Toggleable;
use crate::high_level::{
    get_target_track, Chunk, ChunkRegion, InputMonitoringMode, MidiRecordingInput, Pan, Project,
    Reaper, RecordingInput, Volume,
};
use crate::low_level::{
    get_control_surface_instance, MediaTrack, ReaProject, CSURF_EXT_SETINPUTMONITOR, GUID,
};
use crate::medium_level;

pub const MAX_TRACK_CHUNK_SIZE: u32 = 1_000_000;

#[derive(Clone, Debug, Eq)]
// TODO-low Reconsider design. Maybe don't do that interior mutability stuff. By moving from lazy to
//  eager (determining rea_project and media_track at construction time).
pub struct Track {
    // Only filled if track loaded.
    media_track: Cell<*mut MediaTrack>,
    // TODO-low Do we really need this pointer? Makes copying a tiny bit more expensive than just copying a MediaTrack*.
    rea_project: Cell<*mut ReaProject>,
    // Possible states:
    // a) guid, project, !mediaTrack (guid-based and not yet loaded)
    // b) guid, mediaTrack (guid-based and loaded)
    // TODO-low This is not super cheap to copy. Do we really need to initialize this eagerly?
    guid: Guid,
}

impl PayloadCopy for Track {}

impl Track {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions < 5.95
    pub fn new(media_track: *mut MediaTrack, rea_project: *mut ReaProject) -> Track {
        Track {
            media_track: Cell::new(media_track),
            rea_project: {
                let actual = if rea_project.is_null() {
                    get_track_project_raw(media_track)
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

    pub(super) fn from_guid(project: Project, guid: Guid) -> Track {
        Track {
            media_track: Cell::new(null_mut()),
            rea_project: Cell::new(project.get_raw()),
            guid: guid,
        }
    }

    pub fn set_name(&self, name: &CStr) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get().medium.get_set_media_track_info(
            self.get_raw(),
            c_str!("P_NAME"),
            name.as_ptr() as *mut c_void,
        );
    }

    // TODO-medium Maybe return borrowed string instead!
    pub fn get_name(&self) -> CString {
        self.load_and_check_if_necessary_or_complain();
        if self.is_master_track() {
            c_str!("<Master track>").to_owned()
        } else {
            let ptr = Reaper::get().medium.get_set_media_track_info(
                self.get_raw(),
                c_str!("P_NAME"),
                null_mut(),
            );
            unsafe { ptr.into_c_str() }.unwrap().to_owned()
        }
    }

    pub fn get_input_monitoring_mode(&self) -> InputMonitoringMode {
        self.load_and_check_if_necessary_or_complain();
        let ptr = Reaper::get().medium.get_set_media_track_info(
            self.get_raw(),
            c_str!("I_RECMON"),
            null_mut(),
        );
        let irecmon = unsafe { ptr.to::<i32>() }.unwrap() as u32;
        InputMonitoringMode::try_from(irecmon).expect("Unknown input monitoring mode")
    }

    pub fn set_input_monitoring_mode(&self, mode: InputMonitoringMode) {
        self.load_and_check_if_necessary_or_complain();
        let irecmon: u32 = mode.into();
        Reaper::get()
            .medium
            .csurf_on_input_monitoring_change_ex(self.get_raw(), irecmon, false);
    }

    pub fn get_recording_input(&self) -> RecordingInput {
        self.load_and_check_if_necessary_or_complain();
        let ptr = Reaper::get().medium.get_set_media_track_info(
            self.get_raw(),
            c_str!("I_RECINPUT"),
            null_mut(),
        );
        let rec_input_index = unsafe { ptr.to::<i32>() }.unwrap();
        RecordingInput::from_rec_input_index(rec_input_index)
    }

    // TODO-low Support setting other kinds of inputs
    pub fn set_recording_input(&self, input: MidiRecordingInput) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        reaper.medium.set_media_track_info_value(
            self.get_raw(),
            c_str!("I_RECINPUT"),
            input.get_rec_input_index() as f64,
        );
        // Only for triggering notification (as manual setting the rec input would also trigger it)
        // This doesn't work for other surfaces but they are also not interested in record input changes.
        let mut rec_mon = reaper
            .medium
            .get_media_track_info_value(self.get_raw(), c_str!("I_RECMON"));
        // TODO-low This is ugly. Solve in other ways.
        let control_surface = get_control_surface_instance();
        control_surface.Extended(
            CSURF_EXT_SETINPUTMONITOR as i32,
            self.get_raw() as *mut c_void,
            &mut rec_mon as *mut f64 as *mut c_void,
            null_mut(),
        );
    }

    pub fn get_raw(&self) -> *mut MediaTrack {
        self.load_if_necessary_or_complain();
        self.media_track.get()
    }

    pub fn get_pan(&self) -> Pan {
        self.load_and_check_if_necessary_or_complain();
        // It's important that we don't query D_PAN because that returns the wrong value in case an envelope is written
        let (_, pan) = Reaper::get()
            .medium
            .get_track_ui_vol_pan(self.get_raw())
            .expect("Couldn't get vol/pan");
        Pan::from_reaper_value(pan)
    }

    pub fn set_pan(&self, pan: Pan) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = pan.get_reaper_value();
        let reaper = Reaper::get();
        reaper
            .medium
            .csurf_on_pan_change_ex(self.get_raw(), reaper_value, false, false);
        // Setting the pan programmatically doesn't trigger SetSurfacePan in HelperControlSurface so we need
        // to notify manually
        reaper
            .medium
            .csurf_set_surface_pan(self.get_raw(), reaper_value, null_mut());
    }

    pub fn get_volume(&self) -> Volume {
        // It's important that we don't query D_VOL because that returns the wrong value in case an envelope is written
        let (volume, _) = Reaper::get()
            .medium
            .get_track_ui_vol_pan(self.get_raw())
            .expect("Couldn't get vol/pan");
        Volume::from_reaper_value(volume)
    }

    pub fn set_volume(&self, volume: Volume) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = volume.get_reaper_value();
        let reaper = Reaper::get();
        // CSurf_OnVolumeChangeEx has a slightly lower precision than setting D_VOL directly. The return value
        // reflects the cropped value. The precision became much better with REAPER 5.28.
        reaper
            .medium
            .csurf_on_volume_change_ex(self.get_raw(), reaper_value, false, false);
        // Setting the volume programmatically doesn't trigger SetSurfaceVolume in HelperControlSurface so we need
        // to notify manually
        reaper
            .medium
            .csurf_set_surface_volume(self.get_raw(), reaper_value, null_mut());
    }

    // TODO-medium Maybe return u32 and express master track index in other ways
    pub fn get_index(&self) -> i32 {
        self.load_and_check_if_necessary_or_complain();
        let ip_track_number = Reaper::get()
            .medium
            .get_set_media_track_info(self.get_raw(), c_str!("IP_TRACKNUMBER"), null_mut())
            .0 as i32;
        if ip_track_number == 0 {
            // Usually means that track doesn't exist. But this we already checked. This happens only if we query the
            // number of a track in another project tab. TODO-low Try to find a working solution. Till then, return 0.
            return 0;
        }
        if ip_track_number == -1 {
            // Master track indicator
            return -1;
        }
        // Must be > 0. Make it zero-rooted.
        ip_track_number - 1
    }

    pub fn has_auto_arm_enabled(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        self.get_auto_arm_chunk_line().is_some()
    }

    pub fn is_armed(&self, support_auto_arm: bool) -> bool {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.is_selected()
        } else {
            self.load_and_check_if_necessary_or_complain();
            Reaper::get()
                .medium
                .get_media_track_info_value(self.get_raw(), c_str!("I_RECARM"))
                == 1.0
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn arm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.select();
        } else {
            let reaper = Reaper::get();
            reaper
                .medium
                .csurf_on_rec_arm_change_ex(self.get_raw(), 1, false);
            // If track was auto-armed before, this would just have switched off the auto-arm but not actually armed
            // the track. Therefore we check if it's really armed and if not we do it again.
            if reaper
                .medium
                .get_media_track_info_value(self.get_raw(), c_str!("I_RECARM"))
                != 1.0
            {
                reaper
                    .medium
                    .csurf_on_rec_arm_change_ex(self.get_raw(), 1, false);
            }
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn disarm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.unselect();
        } else {
            Reaper::get()
                .medium
                .csurf_on_rec_arm_change_ex(self.get_raw(), 0, false);
        }
    }

    pub fn enable_auto_arm(&self) {
        let mut chunk = self.get_chunk(MAX_TRACK_CHUNK_SIZE, false);
        if get_auto_arm_chunk_line(&chunk).is_some() {
            return;
        }
        let was_armed_before = self.is_armed(true);
        chunk.insert_after_region_as_block(&chunk.get_region().get_first_line(), "AUTO_RECARM 1");
        self.set_chunk(chunk);
        if was_armed_before {
            self.arm(true);
        } else {
            self.disarm(true);
        }
    }

    pub fn disable_auto_arm(&self) {
        let chunk = {
            let auto_arm_chunk_line = match self.get_auto_arm_chunk_line() {
                None => return,
                Some(l) => l,
            };
            let mut chunk = auto_arm_chunk_line.get_parent_chunk();
            chunk.delete_region(&auto_arm_chunk_line);
            chunk
        };
        self.set_chunk(chunk);
    }

    pub fn is_muted(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get()
            .medium
            .get_media_track_info_value(self.get_raw(), c_str!("B_MUTE"))
            == 1.0
    }

    pub fn mute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        reaper
            .medium
            .set_media_track_info_value(self.get_raw(), c_str!("B_MUTE"), 1.0);
        reaper
            .medium
            .csurf_set_surface_mute(self.get_raw(), true, null_mut());
    }

    pub fn unmute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        reaper
            .medium
            .set_media_track_info_value(self.get_raw(), c_str!("B_MUTE"), 0.0);
        reaper
            .medium
            .csurf_set_surface_mute(self.get_raw(), false, null_mut());
    }

    pub fn is_solo(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get()
            .medium
            .get_media_track_info_value(self.get_raw(), c_str!("I_SOLO"))
            > 0.0
    }

    pub fn solo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        reaper
            .medium
            .set_media_track_info_value(self.get_raw(), c_str!("I_SOLO"), 1.0);
        reaper
            .medium
            .csurf_set_surface_solo(self.get_raw(), true, null_mut());
    }

    pub fn unsolo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        reaper
            .medium
            .set_media_track_info_value(self.get_raw(), c_str!("I_SOLO"), 0.0);
        reaper
            .medium
            .csurf_set_surface_solo(self.get_raw(), false, null_mut());
    }

    fn get_auto_arm_chunk_line(&self) -> Option<ChunkRegion> {
        get_auto_arm_chunk_line(&self.get_chunk(MAX_TRACK_CHUNK_SIZE, true))
    }

    // Attention! If you pass undoIsOptional = true it's faster but it returns a chunk that contains weird
    // FXID_NEXT (in front of FX tag) instead of FXID (behind FX tag). So FX chunk code should be double checked then.
    pub fn get_chunk(&self, max_chunk_size: u32, undo_is_optional: bool) -> Chunk {
        let chunk_content = Reaper::get()
            .medium
            .get_track_state_chunk(self.get_raw(), max_chunk_size, undo_is_optional)
            .expect("Couldn't load track chunk");
        chunk_content.into()
    }

    pub fn set_chunk(&self, chunk: Chunk) {
        let c_string: CString = chunk.into();
        Reaper::get()
            .medium
            .set_track_state_chunk(self.get_raw(), c_string.as_c_str(), true);
    }

    pub fn is_selected(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get()
            .medium
            .get_media_track_info_value(self.get_raw(), c_str!("I_SELECTED"))
            == 1.0
    }

    pub fn select(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get()
            .medium
            .set_track_selected(self.get_raw(), true);
    }

    pub fn select_exclusively(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get().medium.set_only_track_selected(self.get_raw());
    }

    pub fn unselect(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get()
            .medium
            .set_track_selected(self.get_raw(), false);
    }

    pub fn get_send_count(&self) -> u32 {
        self.load_and_check_if_necessary_or_complain();
        Reaper::get().medium.get_track_num_sends(self.get_raw(), 0)
    }

    pub fn add_send_to(&self, target_track: Track) -> TrackSend {
        // TODO-low Check how this behaves if send already exists
        let send_index = Reaper::get()
            .medium
            .create_track_send(self.get_raw(), target_track.get_raw());
        TrackSend::target_based(self.clone(), target_track, Some(send_index))
    }

    // Returns target-track based sends
    pub fn get_sends(&self) -> impl Iterator<Item = TrackSend> + '_ {
        self.load_and_check_if_necessary_or_complain();
        (0..self.get_send_count()).map(move |i| {
            // Create a stable send (based on target track)
            TrackSend::target_based(self.clone(), get_target_track(self, i), Some(i))
        })
    }

    pub fn get_send_by_index(&self, index: u32) -> Option<TrackSend> {
        if index >= self.get_send_count() {
            return None;
        }
        Some(TrackSend::target_based(
            self.clone(),
            get_target_track(self, index),
            Some(index),
        ))
    }

    pub fn get_send_by_target_track(&self, target_track: Track) -> TrackSend {
        TrackSend::target_based(self.clone(), target_track, None)
    }

    // Non-Optional. Even the index is not a stable identifier, we need a way to create
    // sends just by an index, not to target tracks. Think of ReaLearn for example and saving
    // a preset for a future project which doesn't have the same target track like in the
    // example project.
    pub fn get_index_based_send_by_index(&self, index: u32) -> TrackSend {
        TrackSend::index_based(self.clone(), index)
    }

    // It's correct that this returns an optional because the index isn't a stable identifier of an FX.
    // The FX could move. So this should do a runtime lookup of the FX and return a stable GUID-backed Fx object if
    // an FX exists at that query index.
    pub fn get_fx_by_query_index(&self, query_index: i32) -> Option<Fx> {
        let (index, is_input_fx) = get_index_from_query_index(query_index);
        let fx_chain = if is_input_fx {
            self.get_input_fx_chain()
        } else {
            self.get_normal_fx_chain()
        };
        fx_chain.get_fx_by_index(index)
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
                Reaper::get().medium.validate_ptr_2(
                    self.rea_project.get(),
                    self.media_track.get() as *mut c_void,
                    c_str!("MediaTrack*"),
                )
            } else {
                false
            }
        }
    }

    // Precondition: mediaTrack_ must be filled!
    fn attempt_to_fill_project_if_necessary(&self) {
        if self.rea_project.get().is_null() {
            self.rea_project.replace(self.find_containing_project_raw());
        }
    }

    // TODO-medium Maybe return by value instead
    pub fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn load_by_guid(&self) -> bool {
        if self.rea_project.get().is_null() {
            panic!("For loading per GUID, a project must be given");
        }
        // TODO-low Don't save ReaProject but Project as member
        let guid = self.get_guid();
        let track = self
            .get_project_unchecked()
            .get_tracks()
            .find(|t| t.get_guid() == guid);
        match track {
            Some(t) => {
                self.media_track.replace(t.get_raw());
                true
            }
            None => {
                self.media_track.replace(null_mut());
                false
            }
        }
    }

    pub fn is_available(&self) -> bool {
        if self.media_track.get().is_null() {
            // Not yet loaded
            self.load_by_guid()
        } else {
            // Loaded
            self.is_valid()
        }
    }

    fn get_project_unchecked(&self) -> Project {
        self.attempt_to_fill_project_if_necessary();
        Project::new(self.rea_project.get())
    }

    // Precondition: mediaTrack_ must be filled!
    fn find_containing_project_raw(&self) -> *mut ReaProject {
        if self.media_track.get().is_null() {
            panic!("Containing project cannot be found if mediaTrack not available");
        }
        // No ReaProject* available. Try current project first (most likely in everyday REAPER usage).
        let reaper = Reaper::get();
        let current_project = reaper.get_current_project();
        // TODO-medium Add convenience functions to medium API for checking various pointer types
        let is_valid_in_current_project = reaper.medium.validate_ptr_2(
            current_project.get_raw(),
            self.media_track.get() as *mut c_void,
            c_str!("MediaTrack*"),
        );
        if is_valid_in_current_project {
            return current_project.get_raw();
        }
        // Worst case. It could still be valid in another project. We have to check each project.
        let other_project = reaper
            .get_projects()
            // We already know it's invalid in current project
            .filter(|p| p != &current_project)
            .find(|p| {
                reaper.medium.validate_ptr_2(
                    p.get_raw(),
                    self.media_track.get() as *mut c_void,
                    c_str!("MediaTrack*"),
                )
            });
        other_project.map(|p| p.get_raw()).unwrap_or(null_mut())
    }

    pub fn get_automation_mode(&self) -> AutomationMode {
        self.load_and_check_if_necessary_or_complain();
        let am = Reaper::get()
            .medium
            .get_track_automation_mode(self.media_track.get());
        AutomationMode::try_from(am as i32).expect("Unknown automation mode")
    }

    pub fn get_effective_automation_mode(&self) -> AutomationMode {
        let automation_override = Reaper::get().get_global_automation_override();
        if automation_override == AutomationMode::NoOverride {
            self.get_automation_mode()
        } else {
            automation_override
        }
    }

    pub fn get_normal_fx_chain(&self) -> FxChain {
        FxChain::new(self.clone(), false)
    }

    pub fn get_input_fx_chain(&self) -> FxChain {
        FxChain::new(self.clone(), true)
    }

    pub fn is_master_track(&self) -> bool {
        self.get_index() == -1
    }

    pub fn get_project(&self) -> Project {
        if self.rea_project.get().is_null() {
            self.load_if_necessary_or_complain();
        }
        self.get_project_unchecked()
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        if self.media_track.get().is_null() || other.media_track.get().is_null() {
            self.get_guid() == other.get_guid()
        } else {
            self.media_track == other.media_track
        }
    }
}

pub fn get_media_track_guid(media_track: *mut MediaTrack) -> Guid {
    let internal = Reaper::get()
        .medium
        .get_set_media_track_info(media_track, c_str!("GUID"), null_mut())
        .0 as *mut GUID;
    Guid::new(unsafe { *internal })
}

// In REAPER < 5.95 this returns nullptr. That means we might need to use findContainingProject logic at a later
// point.
fn get_track_project_raw(media_track: *mut MediaTrack) -> *mut ReaProject {
    Reaper::get()
        .medium
        .get_set_media_track_info(media_track, c_str!("P_PROJECT"), null_mut())
        .0 as *mut ReaProject
}

fn get_auto_arm_chunk_line(chunk: &Chunk) -> Option<ChunkRegion> {
    chunk.get_region().find_line_starting_with("AUTO_RECARM 1")
}
