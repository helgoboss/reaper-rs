use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::sync::Once;
use std::convert::TryFrom;

use c_str_macro::c_str;

use crate::high_level::{Project, Reaper, InputMonitoringMode, RecordingInput, MidiRecordingInput, Volume, Pan, ChunkRegion, Chunk, get_target_track};
use crate::high_level::ActionKind::Toggleable;
use crate::high_level::guid::Guid;
use crate::low_level::{MediaTrack, ReaProject, get_control_surface_instance, CSURF_EXT_SETINPUTMONITOR};
use crate::medium_level;
use crate::high_level::automation_mode::AutomationMode;
use crate::high_level::fx_chain::FxChain;
use crate::high_level::fx::{Fx, get_index_from_query_index};
use crate::high_level::track_send::TrackSend;
use slog::debug;

/// The difference to Track is that this implements Copy (not just Clone)
// TODO Maybe it's more efficient to use a moving or copying pointer for track Observables? Anyway,
//  this would require rxRust subjects to work with elements that are not copyable (because Rc,
//  RefCell, Box, Arc and all that stuff are never copyable) but just cloneable
#[derive(Clone, Copy, Debug, Eq)]
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

impl PartialEq for LightTrack {
    fn eq(&self, other: &Self) -> bool {
        Track::from(self.clone()) == Track::from(other.clone())
    }
}

const MAX_CHUNK_SIZE: u32 = 1_000_000;

// TODO Think hard about what equality means here!
#[derive(Clone, Debug, Eq)]
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
            guid: light.guid,
        }
    }
}

impl From<Track> for LightTrack {
    fn from(heavy: Track) -> Self {
        LightTrack {
            media_track: heavy.media_track.get(),
            rea_project: heavy.rea_project.get(),
            guid: heavy.guid,
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

    pub(super) fn from_guid(project: Project, guid: Guid) -> Track {
        Track {
            media_track: Cell::new(null_mut()),
            rea_project: Cell::new(project.get_rea_project()),
            guid: guid,
        }
    }

    pub fn set_name(&self, name: &CStr) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.get_set_media_track_info(
            self.get_media_track(),
            c_str!("P_NAME"),
            name.as_ptr() as *mut c_void,
        );
    }

    pub fn get_name(&self) -> CString {
        self.load_and_check_if_necessary_or_complain();
        if self.is_master_track() {
            c_str!("<Master track>").to_owned()
        } else {
            Reaper::instance().medium.convenient_get_media_track_info_string(self.get_media_track(), c_str!("P_NAME"))
        }
    }

    pub fn get_input_monitoring_mode(&self) -> InputMonitoringMode {
        self.load_and_check_if_necessary_or_complain();
        let irecmon = Reaper::instance().medium.convenient_get_media_track_info_i32_ptr(self.get_media_track(), c_str!("I_RECMON"));
        InputMonitoringMode::try_from(irecmon).expect("Unknown input monitoring mode")
    }

    pub fn set_input_monitoring_mode(&self, mode: InputMonitoringMode) {
        self.load_and_check_if_necessary_or_complain();
        let irecmon: i32 = mode.into();
        Reaper::instance().medium.csurf_on_input_monitoring_change_ex(self.get_media_track(), irecmon, false);
    }

    pub fn get_recording_input(&self) -> RecordingInput {
        self.load_and_check_if_necessary_or_complain();
        let rec_input_index = Reaper::instance().medium.convenient_get_media_track_info_i32_ptr(self.get_media_track(), c_str!("I_RECINPUT"));
        RecordingInput::from_rec_input_index(rec_input_index)
    }

    // TODO Support setting other kinds of inputs
    pub fn set_recording_input(&self, input: MidiRecordingInput) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::instance();
        reaper.medium.set_media_track_info_value(self.get_media_track(), c_str!("I_RECINPUT"), input.get_rec_input_index() as f64);
        // Only for triggering notification (as manual setting the rec input would also trigger it)
        // This doesn't work for other surfaces but they are also not interested in record input changes.
        let mut rec_mon = reaper.medium.get_media_track_info_value(self.get_media_track(), c_str!("I_RECMON"));
        // TODO This is ugly. Solve in other ways.
        let control_surface = get_control_surface_instance();
        control_surface.Extended(CSURF_EXT_SETINPUTMONITOR as i32, self.get_media_track() as *mut c_void, &mut rec_mon as *mut f64 as *mut c_void, null_mut());
    }

    pub fn get_media_track(&self) -> *mut MediaTrack {
        self.load_if_necessary_or_complain();
        self.media_track.get()
    }

    pub fn get_pan(&self) -> Pan {
        self.load_and_check_if_necessary_or_complain();
        // It's important that we don't query D_PAN because that returns the wrong value in case an envelope is written
        let (_, pan) = Reaper::instance().medium.get_track_ui_vol_pan(self.get_media_track())
            .expect("Couldn't get vol/pan");
        Pan::of_reaper_value(pan)
    }

    pub fn set_pan(&self, pan: Pan) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = pan.get_reaper_value();
        let reaper = Reaper::instance();
        reaper.medium.csurf_on_pan_change_ex(self.get_media_track(), reaper_value, false, false);
        // Setting the pan programmatically doesn't trigger SetSurfacePan in HelperControlSurface so we need
        // to notify manually
        reaper.medium.csurf_set_surface_pan(self.get_media_track(), reaper_value, null_mut());
    }

    pub fn get_volume(&self) -> Volume {
        // It's important that we don't query D_VOL because that returns the wrong value in case an envelope is written
        let (volume, _) = Reaper::instance().medium.get_track_ui_vol_pan(self.get_media_track())
            .expect("Couldn't get vol/pan");
        Volume::of_reaper_value(volume)
    }

    pub fn set_volume(&self, volume: Volume) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = volume.get_reaper_value();
        let reaper = Reaper::instance();
        // CSurf_OnVolumeChangeEx has a slightly lower precision than setting D_VOL directly. The return value
        // reflects the cropped value. The precision became much better with REAPER 5.28.
        reaper.medium.csurf_on_volume_change_ex(self.get_media_track(), reaper_value, false, false);
        // Setting the volume programmatically doesn't trigger SetSurfaceVolume in HelperControlSurface so we need
        // to notify manually
        reaper.medium.csurf_set_surface_volume(self.get_media_track(), reaper_value, null_mut());
    }

    // TODO Maybe return u32 and express master track index in other ways
    pub fn get_index(&self) -> i32 {
        self.load_and_check_if_necessary_or_complain();
        let ip_track_number = Reaper::instance().medium.convenient_get_media_track_info_i32_value(self.get_media_track(), c_str!("IP_TRACKNUMBER"));
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

    pub fn has_auto_arm_enabled(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        self.get_auto_arm_chunk_line().is_some()
    }

    pub fn is_armed(&self, support_auto_arm: bool) -> bool {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.is_selected()
        } else {
            self.load_and_check_if_necessary_or_complain();
            Reaper::instance().medium.get_media_track_info_value(self.get_media_track(), c_str!("I_RECARM")) == 1.0
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn arm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.select();
        } else {
            let reaper = Reaper::instance();
            reaper.medium.csurf_on_rec_arm_change_ex(self.get_media_track(), 1, false);
            // If track was auto-armed before, this would just have switched off the auto-arm but not actually armed
            // the track. Therefore we check if it's really armed and if not we do it again.
            if reaper.medium.get_media_track_info_value(self.get_media_track(), c_str!("I_RECARM")) != 1.0 {
                reaper.medium.csurf_on_rec_arm_change_ex(self.get_media_track(), 1, false);
            }
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn disarm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.unselect();
        } else {
            Reaper::instance().medium.csurf_on_rec_arm_change_ex(self.get_media_track(), 0, false);
        }
    }

    pub fn enable_auto_arm(&self) {
        let mut chunk = self.get_chunk(MAX_CHUNK_SIZE, false);
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
                Some(l) => l
            };
            let mut chunk = auto_arm_chunk_line.get_parent_chunk();
            chunk.delete_region(&auto_arm_chunk_line);
            chunk
        };
        self.set_chunk(chunk);
    }

    pub fn is_muted(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.get_media_track_info_value(self.get_media_track(), c_str!("B_MUTE")) == 1.0
    }

    fn get_auto_arm_chunk_line(&self) -> Option<ChunkRegion> {
        get_auto_arm_chunk_line(&self.get_chunk(MAX_CHUNK_SIZE, true))
    }

    // Attention! If you pass undoIsOptional = true it's faster but it returns a chunk that contains weird
    // FXID_NEXT (in front of FX tag) instead of FXID (behind FX tag). So FX chunk code should be double checked then.
    pub fn get_chunk(&self, max_chunk_size: u32, undo_is_optional: bool) -> Chunk {
        let chunk_content = Reaper::instance().medium
            .get_track_state_chunk(self.get_media_track(), max_chunk_size, undo_is_optional)
            .expect("Couldn't load track chunk");
        chunk_content.into()
    }

    pub fn set_chunk(&self, chunk: Chunk) {
        let c_string: CString = chunk.into();
        Reaper::instance().medium.set_track_state_chunk(self.get_media_track(), c_string.as_c_str(), true);
    }

    pub fn is_selected(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.get_media_track_info_value(self.get_media_track(), c_str!("I_SELECTED")) == 1.0
    }

    pub fn select(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.set_track_selected(self.get_media_track(), true);
    }

    pub fn select_exclusively(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.set_only_track_selected(self.get_media_track());
    }

    pub fn unselect(&self) {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.set_track_selected(self.get_media_track(), false);
    }

    pub fn get_send_count(&self) -> u32 {
        self.load_and_check_if_necessary_or_complain();
        Reaper::instance().medium.get_track_num_sends(self.get_media_track(), 0)
    }

    pub fn add_send_to(&self, target_track: Track) -> TrackSend {
        // TODO Check how this behaves if send already exists
        let send_index = Reaper::instance().medium.create_track_send(self.get_media_track(), target_track.get_media_track());
        TrackSend::target_based(self.clone(), target_track, Some(send_index))
    }

    // Returns target-track based sends
    pub fn get_sends(&self) -> impl Iterator<Item=TrackSend> + '_ {
        self.load_and_check_if_necessary_or_complain();
        (0..self.get_send_count())
            .map(move |i| {
                // Create a stable send (based on target track)
                TrackSend::target_based(self.clone(), get_target_track(self, i), Some(i))
            })
    }

    pub fn get_send_by_index(&self, index: u32) -> Option<TrackSend> {
        if index >= self.get_send_count() {
            return None;
        }
        Some(TrackSend::target_based(self.clone(), get_target_track(self, index), Some(index)))
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

    // TODO Maybe return by value instead
    pub fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn load_by_guid(&self) -> bool {
        if self.rea_project.get().is_null() {
            panic!("For loading per GUID, a project must be given");
        }
        // TODO Don't save ReaProject but Project as member
        let guid = self.get_guid();
        let track = self.get_project_unchecked().get_tracks()
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

    pub fn get_automation_mode(&self) -> AutomationMode {
        self.load_and_check_if_necessary_or_complain();
        let am = Reaper::instance().medium.get_track_automation_mode(self.media_track.get());
        AutomationMode::try_from(am).expect("Unknown automation mode")
    }

    pub fn get_effective_automation_mode(&self) -> AutomationMode {
        let automation_override = Reaper::instance().get_global_automation_override();
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
    let internal = Reaper::instance().medium.convenient_get_media_track_info_guid(media_track, c_str!("GUID"));
    Guid::new(unsafe { *internal })
}

// In REAPER < 5.95 this returns nullptr. That means we might need to use findContainingProject logic at a later
// point.
fn get_media_track_rea_project(media_track: *mut MediaTrack) -> *mut ReaProject {
    Reaper::instance().medium.get_set_media_track_info(media_track, c_str!("P_PROJECT"), null_mut()) as *mut ReaProject
}

fn get_auto_arm_chunk_line(chunk: &Chunk) -> Option<ChunkRegion> {
    chunk.get_region().find_line_starting_with("AUTO_RECARM 1")
}