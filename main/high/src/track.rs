use std::cell::Cell;

use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr::null_mut;

use c_str_macro::c_str;

use rxrust::prelude::PayloadCopy;

use crate::fx::{get_index_from_query_index, Fx};
use crate::fx_chain::FxChain;
use crate::guid::Guid;
use crate::track_send::TrackSend;

use crate::{get_target_track, Chunk, ChunkRegion, Pan, Project, Reaper, Volume};
use reaper_rs_low::get_control_surface_instance;
use reaper_rs_low::raw;
use reaper_rs_low::raw::{CSURF_EXT_SETINPUTMONITOR, GUID};

use reaper_rs_medium::TrackInfoKey::{
    B_MUTE, IP_TRACKNUMBER, I_RECARM, I_RECINPUT, I_RECMON, I_SELECTED, I_SOLO, P_NAME, P_PROJECT,
};
use reaper_rs_medium::ValueChange::Absolute;
use reaper_rs_medium::{
    AllowGang, AutomationMode, GlobalAutomationOverride, InputMonitoringMode, MediaTrack,
    ReaProject, ReaperPointer, RecArmState, RecordingInput, TrackInfoKey, TrackRef,
    TrackSendCategory, UndoHint, ValueChange,
};

pub const MAX_TRACK_CHUNK_SIZE: u32 = 1_000_000;

#[derive(Clone, Debug, Eq)]
// TODO-low Reconsider design. Maybe don't do that interior mutability stuff. By moving from lazy to
//  eager (determining rea_project and media_track at construction time).
pub struct Track {
    // Only filled if track loaded.
    media_track: Cell<Option<MediaTrack>>,
    // TODO-low Do we really need this pointer? Makes copying a tiny bit more expensive than just
    // copying a MediaTrack*.
    rea_project: Cell<Option<ReaProject>>,
    // Possible states:
    // a) guid, project, !mediaTrack (guid-based and not yet loaded)
    // b) guid, mediaTrack (guid-based and loaded)
    // TODO-low This is not super cheap to copy. Do we really need to initialize this eagerly?
    guid: Guid,
}

impl PayloadCopy for Track {}

impl Track {
    /// mediaTrack must not be null
    /// reaProject can be null but providing it can speed things up quite much for REAPER versions <
    /// 5.95
    pub fn new(media_track: MediaTrack, rea_project: Option<ReaProject>) -> Track {
        Track {
            media_track: Cell::new(Some(media_track)),
            rea_project: {
                let actual = rea_project.or_else(|| get_track_project_raw(media_track));
                Cell::new(actual)
            },
            // We load the GUID eagerly because we want to make comparability possible even in the
            // following case: Track A has been initialized with a GUID not been loaded
            // yet, track B has been initialized with a MediaTrack* (this constructor)
            // but has rendered invalid in the meantime. Now there would not be any way to compare
            // them because I can neither compare MediaTrack* pointers nor GUIDs. Except
            // I extract the GUID eagerly.
            guid: get_media_track_guid(media_track),
        }
    }

    pub(super) fn from_guid(project: Project, guid: Guid) -> Track {
        Track {
            media_track: Cell::new(None),
            rea_project: Cell::new(Some(project.get_raw())),
            guid: guid,
        }
    }

    pub fn set_name(&self, name: &CStr) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get().medium.get_set_media_track_info(
                self.get_raw(),
                P_NAME,
                name.as_ptr() as *mut c_void,
            );
        }
    }

    // TODO-low Maybe return borrowed string instead!
    pub fn get_name(&self) -> CString {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_name(self.get_raw(), |n| n.into())
        }
        .unwrap_or_else(|| c_str!("<Master track>").to_owned())
    }

    pub fn get_input_monitoring_mode(&self) -> InputMonitoringMode {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_recmon(self.get_raw())
        }
    }

    pub fn set_input_monitoring_mode(&self, mode: InputMonitoringMode) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get().medium.csurf_on_input_monitoring_change_ex(
                self.get_raw(),
                mode,
                AllowGang::No,
            );
        }
    }

    pub fn get_recording_input(&self) -> Option<RecordingInput> {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_recinput(self.get_raw())
        }
    }

    pub fn set_recording_input(&self, input: Option<RecordingInput>) {
        self.load_and_check_if_necessary_or_complain();
        let rec_input_index = match input {
            None => -1,
            Some(ri) => u32::from(ri) as i32,
        };
        let reaper = Reaper::get();
        let _ = unsafe {
            reaper.medium.set_media_track_info_value(
                self.get_raw(),
                I_RECINPUT,
                rec_input_index as f64,
            )
        };
        // Only for triggering notification (as manual setting the rec input would also trigger it)
        // This doesn't work for other surfaces but they are also not interested in record input
        // changes.
        let mut rec_mon = unsafe {
            reaper
                .medium
                .get_media_track_info_value(self.get_raw(), I_RECMON)
        };
        // TODO-low This is ugly. Solve in other ways.
        let control_surface = get_control_surface_instance();
        let super_raw: *mut raw::MediaTrack = self.get_raw().as_ptr();
        control_surface.Extended(
            CSURF_EXT_SETINPUTMONITOR as i32,
            super_raw as *mut c_void,
            &mut rec_mon as *mut f64 as *mut c_void,
            null_mut(),
        );
    }

    pub fn get_raw(&self) -> MediaTrack {
        self.load_if_necessary_or_complain();
        self.media_track.get().unwrap()
    }

    pub fn get_pan(&self) -> Pan {
        self.load_and_check_if_necessary_or_complain();
        // It's important that we don't query D_PAN because that returns the wrong value in case an
        // envelope is written
        let result = unsafe { Reaper::get().medium.get_track_ui_vol_pan(self.get_raw()) }
            .expect("Couldn't get vol/pan");
        Pan::from_reaper_value(result.pan)
    }

    pub fn set_pan(&self, pan: Pan) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = pan.get_reaper_value();
        let reaper = Reaper::get();
        unsafe {
            reaper.medium.csurf_on_pan_change_ex(
                self.get_raw(),
                Absolute(reaper_value),
                AllowGang::No,
            );
        }
        // Setting the pan programmatically doesn't trigger SetSurfacePan in HelperControlSurface so
        // we need to notify manually
        unsafe {
            reaper
                .medium
                .csurf_set_surface_pan(self.get_raw(), reaper_value, None);
        }
    }

    pub fn get_volume(&self) -> Volume {
        // It's important that we don't query D_VOL because that returns the wrong value in case an
        // envelope is written
        let result = unsafe { Reaper::get().medium.get_track_ui_vol_pan(self.get_raw()) }
            .expect("Couldn't get vol/pan");
        Volume::from_reaper_value(result.volume)
    }

    pub fn set_volume(&self, volume: Volume) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = volume.get_reaper_value();
        let reaper = Reaper::get();
        // CSurf_OnVolumeChangeEx has a slightly lower precision than setting D_VOL directly. The
        // return value reflects the cropped value. The precision became much better with
        // REAPER 5.28.
        unsafe {
            reaper.medium.csurf_on_volume_change_ex(
                self.get_raw(),
                Absolute(reaper_value),
                AllowGang::No,
            );
        }
        // Setting the volume programmatically doesn't trigger SetSurfaceVolume in
        // HelperControlSurface so we need to notify manually
        unsafe {
            reaper
                .medium
                .csurf_set_surface_volume(self.get_raw(), reaper_value, None);
        }
    }

    pub fn get_index(&self) -> Option<u32> {
        self.load_and_check_if_necessary_or_complain();
        // TODO-low The following returns None if we query the number of a track in another project
        //  Try to find a working solution!
        let result = unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_tracknumber(self.get_raw())
        }?;
        use TrackRef::*;
        match result {
            MasterTrack => None,
            TrackIndex(idx) => Some(idx),
        }
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
            let recarm = unsafe {
                Reaper::get()
                    .medium
                    .get_media_track_info_value(self.get_raw(), I_RECARM)
            };
            recarm == 1.0
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn arm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.select();
        } else {
            let reaper = Reaper::get();
            unsafe {
                reaper.medium.csurf_on_rec_arm_change_ex(
                    self.get_raw(),
                    RecArmState::Armed,
                    AllowGang::No,
                );
            }
            // If track was auto-armed before, this would just have switched off the auto-arm but
            // not actually armed the track. Therefore we check if it's really armed and
            // if not we do it again.
            let recarm = unsafe {
                reaper
                    .medium
                    .get_media_track_info_value(self.get_raw(), I_RECARM)
            };
            if recarm != 1.0 {
                unsafe {
                    reaper.medium.csurf_on_rec_arm_change_ex(
                        self.get_raw(),
                        RecArmState::Armed,
                        AllowGang::No,
                    );
                }
            }
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn disarm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.unselect();
        } else {
            unsafe {
                Reaper::get().medium.csurf_on_rec_arm_change_ex(
                    self.get_raw(),
                    RecArmState::Unarmed,
                    AllowGang::No,
                );
            }
        }
    }

    pub fn enable_auto_arm(&self) {
        let mut chunk = self.get_chunk(MAX_TRACK_CHUNK_SIZE, UndoHint::UndoIsRequired);
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
        let mute = unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_value(self.get_raw(), B_MUTE)
        };
        mute == 1.0
    }

    pub fn mute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        let _ = unsafe {
            reaper
                .medium
                .set_media_track_info_value(self.get_raw(), B_MUTE, 1.0)
        };
        unsafe {
            reaper
                .medium
                .csurf_set_surface_mute(self.get_raw(), true, None);
        }
    }

    pub fn unmute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        let _ = unsafe {
            reaper
                .medium
                .set_media_track_info_value(self.get_raw(), B_MUTE, 0.0)
        };
        unsafe {
            reaper
                .medium
                .csurf_set_surface_mute(self.get_raw(), false, None);
        }
    }

    pub fn is_solo(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let solo = unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_value(self.get_raw(), I_SOLO)
        };
        solo > 0.0
    }

    pub fn solo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        let _ = unsafe {
            reaper
                .medium
                .set_media_track_info_value(self.get_raw(), I_SOLO, 1.0)
        };
        unsafe {
            reaper
                .medium
                .csurf_set_surface_solo(self.get_raw(), true, None);
        }
    }

    pub fn unsolo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get();
        let _ = unsafe {
            reaper
                .medium
                .set_media_track_info_value(self.get_raw(), I_SOLO, 0.0)
        };
        unsafe {
            reaper
                .medium
                .csurf_set_surface_solo(self.get_raw(), false, None);
        }
    }

    fn get_auto_arm_chunk_line(&self) -> Option<ChunkRegion> {
        get_auto_arm_chunk_line(&self.get_chunk(MAX_TRACK_CHUNK_SIZE, UndoHint::UndoIsOptional))
    }

    // Attention! If you pass undoIsOptional = true it's faster but it returns a chunk that contains
    // weird FXID_NEXT (in front of FX tag) instead of FXID (behind FX tag). So FX chunk code
    // should be double checked then.
    pub fn get_chunk(&self, max_chunk_size: u32, undo_is_optional: UndoHint) -> Chunk {
        let chunk_content = unsafe {
            Reaper::get().medium.get_track_state_chunk(
                self.get_raw(),
                max_chunk_size,
                undo_is_optional,
            )
        }
        .expect("Couldn't load track chunk");
        chunk_content.into()
    }

    // TODO-low Report possible error
    pub fn set_chunk(&self, chunk: Chunk) {
        let c_string: CString = chunk.into();
        let _ = unsafe {
            Reaper::get().medium.set_track_state_chunk(
                self.get_raw(),
                c_string.as_c_str(),
                UndoHint::UndoIsOptional,
            )
        };
    }

    pub fn is_selected(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let selected = unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_value(self.get_raw(), I_SELECTED)
        };
        selected == 1.0
    }

    pub fn select(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .set_track_selected(self.get_raw(), true);
        }
    }

    pub fn select_exclusively(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .set_only_track_selected(Some(self.get_raw()));
        }
    }

    pub fn unselect(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .set_track_selected(self.get_raw(), false);
        }
    }

    pub fn get_send_count(&self) -> u32 {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .get_track_num_sends(self.get_raw(), TrackSendCategory::Send)
        }
    }

    pub fn add_send_to(&self, target_track: Track) -> TrackSend {
        // TODO-low Check how this behaves if send already exists
        let send_index = unsafe {
            Reaper::get()
                .medium
                .create_track_send(self.get_raw(), Some(target_track.get_raw()))
        };
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

    // It's correct that this returns an optional because the index isn't a stable identifier of an
    // FX. The FX could move. So this should do a runtime lookup of the FX and return a stable
    // GUID-backed Fx object if an FX exists at that query index.
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
        if self.media_track.get().is_none() && !self.load_by_guid() {
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
        let media_track = match self.media_track.get() {
            None => panic!("Track can not be validated if mediaTrack not available"),
            Some(t) => t,
        };
        self.attempt_to_fill_project_if_necessary();
        match self.rea_project.get() {
            None => false,
            Some(rea_project) => {
                if Project::new(rea_project).is_available() {
                    Reaper::get()
                        .medium
                        .validate_ptr_2(Some(rea_project), media_track)
                } else {
                    false
                }
            }
        }
    }

    // Precondition: mediaTrack_ must be filled!
    fn attempt_to_fill_project_if_necessary(&self) {
        if self.rea_project.get().is_none() {
            self.rea_project.replace(self.find_containing_project_raw());
        }
    }

    pub fn get_guid(&self) -> &Guid {
        &self.guid
    }

    fn load_by_guid(&self) -> bool {
        if self.rea_project.get().is_none() {
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
                self.media_track.replace(Some(t.get_raw()));
                true
            }
            None => {
                self.media_track.replace(None);
                false
            }
        }
    }

    pub fn is_available(&self) -> bool {
        if self.media_track.get().is_none() {
            // Not yet loaded
            self.load_by_guid()
        } else {
            // Loaded
            self.is_valid()
        }
    }

    fn get_project_unchecked(&self) -> Project {
        self.attempt_to_fill_project_if_necessary();
        Project::new(self.rea_project.get().unwrap())
    }

    // Precondition: mediaTrack_ must be filled!
    fn find_containing_project_raw(&self) -> Option<ReaProject> {
        let media_track = match self.media_track.get() {
            None => panic!("Containing project cannot be found if mediaTrack not available"),
            Some(t) => t,
        };
        // No ReaProject* available. Try current project first (most likely in everyday REAPER
        // usage).
        let reaper = Reaper::get();
        let current_project = reaper.get_current_project();
        let is_valid_in_current_project = reaper
            .medium
            .validate_ptr_2(Some(current_project.get_raw()), media_track);
        if is_valid_in_current_project {
            return Some(current_project.get_raw());
        }
        // Worst case. It could still be valid in another project. We have to check each project.
        let other_project = reaper
            .get_projects()
            // We already know it's invalid in current project
            .filter(|p| p != &current_project)
            .find(|p| reaper.medium.validate_ptr_2(Some(p.get_raw()), media_track));
        other_project.map(|p| p.get_raw())
    }

    pub fn get_automation_mode(&self) -> AutomationMode {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium
                .get_track_automation_mode(self.media_track.get().unwrap())
        }
    }

    // None means Bypass
    pub fn get_effective_automation_mode(&self) -> Option<AutomationMode> {
        use GlobalAutomationOverride::*;
        match Reaper::get().get_global_automation_override() {
            None => Some(self.get_automation_mode()),
            Some(Bypass) => None,
            Some(Mode(am)) => Some(am),
        }
    }

    pub fn get_normal_fx_chain(&self) -> FxChain {
        FxChain::new(self.clone(), false)
    }

    pub fn get_input_fx_chain(&self) -> FxChain {
        FxChain::new(self.clone(), true)
    }

    pub fn is_master_track(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let t = unsafe {
            Reaper::get()
                .medium
                .get_media_track_info_tracknumber(self.get_raw())
        };
        t == Some(TrackRef::MasterTrack)
    }

    pub fn get_project(&self) -> Project {
        if self.rea_project.get().is_none() {
            self.load_if_necessary_or_complain();
        }
        self.get_project_unchecked()
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        match (&self.media_track.get(), &other.media_track.get()) {
            (Some(self_media_track), Some(other_media_track)) => {
                self_media_track == other_media_track
            }
            _ => self.get_guid() == other.get_guid(),
        }
    }
}

pub fn get_media_track_guid(media_track: MediaTrack) -> Guid {
    let internal = unsafe { Reaper::get().medium.get_media_track_info_guid(media_track) };
    Guid::new(internal)
}

// In REAPER < 5.95 this returns nullptr. That means we might need to use findContainingProject
// logic at a later point.
fn get_track_project_raw(media_track: MediaTrack) -> Option<ReaProject> {
    unsafe {
        Reaper::get()
            .medium
            .get_media_track_info_project(media_track)
    }
}

fn get_auto_arm_chunk_line(chunk: &Chunk) -> Option<ChunkRegion> {
    chunk.get_region().find_line_starting_with("AUTO_RECARM 1")
}
