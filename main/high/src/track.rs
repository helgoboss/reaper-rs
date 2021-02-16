use std::cell::Cell;

use crate::fx::{get_index_from_query_index, Fx};
use crate::fx_chain::FxChain;
use crate::guid::Guid;
use crate::track_send::TrackSend;

use crate::{get_target_track, Chunk, ChunkRegion, Pan, Project, Reaper, Volume, Width};

use reaper_medium::NotificationBehavior::NotifyAll;
use reaper_medium::ProjectContext::Proj;
use reaper_medium::SendTarget::OtherTrack;
use reaper_medium::TrackAttributeKey::{Mute, RecArm, RecInput, RecMon, Selected, Solo};
use reaper_medium::ValueChange::Absolute;
use reaper_medium::{
    AutomationMode, ChunkCacheHint, GangBehavior, GlobalAutomationModeOverride,
    InputMonitoringMode, MediaTrack, ReaProject, ReaperString, ReaperStringArg, ReaperWidthValue,
    RecordArmMode, RecordingInput, TrackAttributeKey, TrackLocation, TrackSendCategory,
};
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

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
            rea_project: Cell::new(Some(project.raw())),
            guid,
        }
    }

    pub fn set_name<'a>(&self, name: impl Into<ReaperStringArg<'a>>) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_set_name(self.raw(), name);
        }
    }

    // TODO-low It's really annoying to always have to unwrap an option even if we know this is not
    //  a master track. Maybe we should have different types: Track, MasterTrack, NormalTrack
    pub fn name(&self) -> Option<ReaperString> {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_name(self.raw(), |n| n.to_owned())
        }
    }

    pub fn input_monitoring_mode(&self) -> InputMonitoringMode {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_rec_mon(self.raw())
        }
    }

    pub fn set_input_monitoring_mode(&self, mode: InputMonitoringMode) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .csurf_on_input_monitoring_change_ex(self.raw(), mode, GangBehavior::DenyGang);
        }
    }

    pub fn recording_input(&self) -> Option<RecordingInput> {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_rec_input(self.raw())
        }
    }

    pub fn set_recording_input(&self, input: Option<RecordingInput>) {
        self.load_and_check_if_necessary_or_complain();
        let rec_input_index = match input {
            None => -1,
            Some(ri) => ri.to_raw(),
        };
        let _ = unsafe {
            Reaper::get().medium_reaper().set_media_track_info_value(
                self.raw(),
                RecInput,
                rec_input_index as f64,
            )
        };
        // Only for triggering notification (as manual setting the rec input would also trigger it)
        // This doesn't work for other surfaces but they are also not interested in record input
        // changes.
        let _rec_mon = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), RecMon)
        };
        // TODO-low 5198273 This is ugly. Solve in other ways.
        // let control_surface = get_control_surface_instance();
        // let super_raw: *mut raw::MediaTrack = self.raw().as_ptr();
        // control_surface.Extended(
        //     CSURF_EXT_SETINPUTMONITOR as i32,
        //     super_raw as *mut c_void,
        //     &mut rec_mon as *mut f64 as *mut c_void,
        //     null_mut(),
        // );
    }

    pub fn raw(&self) -> MediaTrack {
        self.load_if_necessary_or_complain();
        self.media_track.get().unwrap()
    }

    pub fn pan(&self) -> Pan {
        self.load_and_check_if_necessary_or_complain();
        // It's important that we don't query D_PAN because that returns the wrong value in case an
        // envelope is written
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_vol_pan(self.raw())
        }
        .expect("Couldn't get vol/pan");
        Pan::from_reaper_value(result.pan)
    }

    pub fn set_pan(&self, pan: Pan) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = pan.reaper_value();
        unsafe {
            Reaper::get().medium_reaper().csurf_on_pan_change_ex(
                self.raw(),
                Absolute(reaper_value),
                GangBehavior::DenyGang,
            );
        }
        // Setting the pan programmatically doesn't trigger SetSurfacePan for control surfaces so
        // we need to notify manually
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_pan(
                self.raw(),
                reaper_value,
                NotifyAll,
            );
        }
    }

    pub fn width(&self) -> Width {
        self.load_and_check_if_necessary_or_complain();
        // TODO-high For Pan, we don't use D_PAN because that returns the wrong value in case an
        // envelope is written. There's no such thing for D_WIDTH until now. Maybe request it.
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), TrackAttributeKey::Width)
        };
        Width::from_reaper_value(ReaperWidthValue::new(result))
    }

    pub fn set_width(&self, width: Width) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = width.reaper_value();
        unsafe {
            Reaper::get().medium_reaper().csurf_on_width_change_ex(
                self.raw(),
                Absolute(reaper_value),
                GangBehavior::DenyGang,
            );
        }
        // Setting the width programmatically doesn't trigger SetSurfacePan for control surfaces
        // so we need to notify manually. There's no CSurf_SetSurfaceWidth, so we just retrigger
        // CSurf_SetSurfacePan.
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_pan(
                self.raw(),
                self.pan().reaper_value(),
                NotifyAll,
            );
        }
    }

    pub fn folder_depth_change(&self) -> i32 {
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), TrackAttributeKey::FolderDepth)
        };
        result as _
    }

    pub fn volume(&self) -> Volume {
        // It's important that we don't query D_VOL because that returns the wrong value in case an
        // envelope is written
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_vol_pan(self.raw())
        }
        .expect("Couldn't get vol/pan");
        Volume::from_reaper_value(result.volume)
    }

    pub fn set_volume(&self, volume: Volume) {
        self.load_and_check_if_necessary_or_complain();
        let reaper_value = volume.reaper_value();
        // Why we use this function and not the others:
        //
        // - Setting D_VOL directly via `set_media_track_info_value` will not work for writing
        //   automation.
        // - csurf_set_surface_volume seems to only inform control surfaces, doesn't actually set
        //   the volume.
        //
        // Downsides of using this function:
        //
        // - CSurf_OnVolumeChangeEx has a slightly lower precision than setting D_VOL directly. The
        //   return value reflects the cropped value. However, the precision became much better with
        //   REAPER 5.28.
        // - In automation mode "Touch" this leads to jumps.
        unsafe {
            Reaper::get().medium_reaper().csurf_on_volume_change_ex(
                self.raw(),
                Absolute(reaper_value),
                GangBehavior::DenyGang,
            );
        }
        // Setting the volume programmatically doesn't inform control surfaces - including our own
        // surfaces which are important for feedback. So use the following to notify manually.
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_volume(
                self.raw(),
                reaper_value,
                NotifyAll,
            );
        }
    }

    pub fn scroll_mixer(&self) {
        self.load_if_necessary_or_complain();
        unsafe {
            Reaper::get().medium_reaper().set_mixer_scroll(self.raw());
        }
    }

    pub fn location(&self) -> TrackLocation {
        self.load_and_check_if_necessary_or_complain();
        // TODO-low The following returns None if we query the number of a track in another project
        //  Try to find a working solution!
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_track_number(self.raw())
                .unwrap()
        }
    }

    pub fn index(&self) -> Option<u32> {
        use TrackLocation::*;
        match self.location() {
            MasterTrack => None,
            NormalTrack(idx) => Some(idx),
        }
    }

    pub fn has_auto_arm_enabled(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        self.auto_arm_chunk_line().is_some()
    }

    #[allow(clippy::float_cmp)]
    pub fn is_armed(&self, support_auto_arm: bool) -> bool {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.is_selected()
        } else {
            self.load_and_check_if_necessary_or_complain();
            let recarm = unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_media_track_info_value(self.raw(), RecArm)
            };
            recarm == 1.0
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn arm(&self, support_auto_arm: bool) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.select();
        } else {
            unsafe {
                Reaper::get().medium_reaper().csurf_on_rec_arm_change_ex(
                    self.raw(),
                    RecordArmMode::Armed,
                    GangBehavior::DenyGang,
                );
            }
            // If track was auto-armed before, this would just have switched off the auto-arm but
            // not actually armed the track. Therefore we check if it's really armed and
            // if not we do it again.
            let recarm = unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_media_track_info_value(self.raw(), RecArm)
            };
            #[allow(clippy::float_cmp)]
            {
                if recarm != 1.0 {
                    unsafe {
                        Reaper::get().medium_reaper().csurf_on_rec_arm_change_ex(
                            self.raw(),
                            RecordArmMode::Armed,
                            GangBehavior::DenyGang,
                        );
                    }
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
                Reaper::get().medium_reaper().csurf_on_rec_arm_change_ex(
                    self.raw(),
                    RecordArmMode::Unarmed,
                    GangBehavior::DenyGang,
                );
            }
        }
    }

    pub fn enable_auto_arm(&self) {
        let mut chunk = self.chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode);
        if get_auto_arm_chunk_line(&chunk).is_some() {
            return;
        }
        let was_armed_before = self.is_armed(true);
        chunk.insert_after_region_as_block(&chunk.region().first_line(), "AUTO_RECARM 1");
        self.set_chunk(chunk);
        if was_armed_before {
            self.arm(true);
        } else {
            self.disarm(true);
        }
    }

    pub fn disable_auto_arm(&self) {
        let chunk = {
            let auto_arm_chunk_line = match self.auto_arm_chunk_line() {
                None => return,
                Some(l) => l,
            };
            let mut chunk = auto_arm_chunk_line.parent_chunk();
            chunk.delete_region(&auto_arm_chunk_line);
            chunk
        };
        self.set_chunk(chunk);
    }

    #[allow(clippy::float_cmp)]
    pub fn is_muted(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let mute = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), Mute)
        };
        mute == 1.0
    }

    pub fn mute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let _ = unsafe {
            Reaper::get()
                .medium_reaper()
                .set_media_track_info_value(self.raw(), Mute, 1.0)
        };
        unsafe {
            Reaper::get()
                .medium_reaper()
                .csurf_set_surface_mute(self.raw(), true, NotifyAll);
        }
    }

    pub fn unmute(&self) {
        self.load_and_check_if_necessary_or_complain();
        let _ = unsafe {
            Reaper::get()
                .medium_reaper()
                .set_media_track_info_value(self.raw(), Mute, 0.0)
        };
        unsafe {
            Reaper::get()
                .medium_reaper()
                .csurf_set_surface_mute(self.raw(), false, NotifyAll);
        }
    }

    pub fn is_solo(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let solo = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), Solo)
        };
        solo > 0.0
    }

    pub fn solo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let _ = unsafe {
            Reaper::get()
                .medium_reaper()
                .set_media_track_info_value(self.raw(), Solo, 1.0)
        };
        unsafe {
            Reaper::get()
                .medium_reaper()
                .csurf_set_surface_solo(self.raw(), true, NotifyAll);
        }
    }

    pub fn unsolo(&self) {
        self.load_and_check_if_necessary_or_complain();
        let _ = unsafe {
            Reaper::get()
                .medium_reaper()
                .set_media_track_info_value(self.raw(), Solo, 0.0)
        };
        unsafe {
            Reaper::get()
                .medium_reaper()
                .csurf_set_surface_solo(self.raw(), false, NotifyAll);
        }
    }

    pub fn fx_is_enabled(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let result = unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_track_info_value(self.raw(), TrackAttributeKey::FxEn)
        };
        result > 0.0
    }

    pub fn enable_fx(&self) {
        self.set_fx_is_enabled(true);
    }

    pub fn disable_fx(&self) {
        self.set_fx_is_enabled(false);
    }

    fn set_fx_is_enabled(&self, enabled: bool) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            let _ = Reaper::get().medium_reaper.set_media_track_info_value(
                self.raw(),
                TrackAttributeKey::FxEn,
                if enabled { 1.0 } else { 0.0 },
            );
        }
    }

    fn auto_arm_chunk_line(&self) -> Option<ChunkRegion> {
        get_auto_arm_chunk_line(&self.chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::UndoMode))
    }

    // Attention! If you pass undoIsOptional = true it's faster but it returns a chunk that contains
    // weird FXID_NEXT (in front of FX tag) instead of FXID (behind FX tag). So FX chunk code
    // should be double checked then.
    pub fn chunk(&self, max_chunk_size: u32, undo_is_optional: ChunkCacheHint) -> Chunk {
        let chunk_content = unsafe {
            Reaper::get().medium_reaper().get_track_state_chunk(
                self.raw(),
                max_chunk_size,
                undo_is_optional,
            )
        }
        .expect("Couldn't load track chunk");
        chunk_content.into()
    }

    // TODO-low Report possible error
    pub fn set_chunk(&self, chunk: Chunk) {
        let string: String = chunk.try_into().expect("unfortunate");
        let _ = unsafe {
            Reaper::get().medium_reaper().set_track_state_chunk(
                self.raw(),
                string,
                ChunkCacheHint::UndoMode,
            )
        };
    }

    #[allow(clippy::float_cmp)]
    pub fn is_selected(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let selected = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw(), Selected)
        };
        selected == 1.0
    }

    pub fn select(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_track_selected(self.raw(), true);
        }
    }

    pub fn select_exclusively(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_only_track_selected(Some(self.raw()));
        }
    }

    pub fn unselect(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_track_selected(self.raw(), false);
        }
    }

    pub fn send_count(&self) -> u32 {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_num_sends(self.raw(), TrackSendCategory::Send)
        }
    }

    pub fn add_send_to(&self, target_track: Track) -> TrackSend {
        // TODO-low Check how this behaves if send already exists
        let send_index = unsafe {
            Reaper::get()
                .medium_reaper()
                .create_track_send(self.raw(), OtherTrack(target_track.raw()))
        }
        .unwrap();
        TrackSend::target_based(self.clone(), target_track, Some(send_index))
    }

    // Returns target-track based sends
    pub fn sends(&self) -> impl Iterator<Item = TrackSend> + '_ {
        self.load_and_check_if_necessary_or_complain();
        (0..self.send_count()).map(move |i| {
            // Create a stable send (based on target track)
            TrackSend::target_based(self.clone(), get_target_track(self, i), Some(i))
        })
    }

    pub fn send_by_index(&self, index: u32) -> Option<TrackSend> {
        if index >= self.send_count() {
            return None;
        }
        Some(TrackSend::target_based(
            self.clone(),
            get_target_track(self, index),
            Some(index),
        ))
    }

    pub fn send_by_target_track(&self, target_track: Track) -> TrackSend {
        TrackSend::target_based(self.clone(), target_track, None)
    }

    // Non-Optional. Even the index is not a stable identifier, we need a way to create
    // sends just by an index, not to target tracks. Think of ReaLearn for example and saving
    // a preset for a future project which doesn't have the same target track like in the
    // example project.
    pub fn index_based_send_by_index(&self, index: u32) -> TrackSend {
        TrackSend::index_based(self.clone(), index)
    }

    // It's correct that this returns an optional because the index isn't a stable identifier of an
    // FX. The FX could move. So this should do a runtime lookup of the FX and return a stable
    // GUID-backed Fx object if an FX exists at that query index.
    pub fn fx_by_query_index(&self, query_index: i32) -> Option<Fx> {
        let (index, is_input_fx) = get_index_from_query_index(query_index);
        let fx_chain = if is_input_fx {
            self.input_fx_chain()
        } else {
            self.normal_fx_chain()
        };
        fx_chain.fx_by_index(index)
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
                        .medium_reaper()
                        .validate_ptr_2(Proj(rea_project), media_track)
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

    pub fn guid(&self) -> &Guid {
        &self.guid
    }

    fn load_by_guid(&self) -> bool {
        if self.rea_project.get().is_none() {
            panic!("For loading per GUID, a project must be given");
        }
        // TODO-low Don't save ReaProject but Project as member
        let guid = self.guid();
        let track = self.project_unchecked().tracks().find(|t| t.guid() == guid);
        match track {
            Some(t) => {
                self.media_track.replace(Some(t.raw()));
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

    fn project_unchecked(&self) -> Project {
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
        let current_project = reaper.current_project();
        let is_valid_in_current_project = reaper
            .medium_reaper()
            .validate_ptr_2(Proj(current_project.raw()), media_track);
        if is_valid_in_current_project {
            return Some(current_project.raw());
        }
        // Worst case. It could still be valid in another project. We have to check each project.
        let other_project = reaper
            .projects()
            // We already know it's invalid in current project
            .filter(|p| p != &current_project)
            .find(|p| {
                reaper
                    .medium_reaper()
                    .validate_ptr_2(Proj(p.raw()), media_track)
            });
        other_project.map(|p| p.raw())
    }

    pub fn automation_mode(&self) -> AutomationMode {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_automation_mode(self.media_track.get().unwrap())
        }
    }

    // None means Bypass
    pub fn effective_automation_mode(&self) -> Option<AutomationMode> {
        use GlobalAutomationModeOverride::*;
        match Reaper::get()
            .medium_reaper()
            .get_global_automation_override()
        {
            None => Some(self.automation_mode()),
            Some(Bypass) => None,
            Some(Mode(am)) => Some(am),
        }
    }

    pub fn normal_fx_chain(&self) -> FxChain {
        FxChain::from_track(self.clone(), false)
    }

    pub fn input_fx_chain(&self) -> FxChain {
        FxChain::from_track(self.clone(), true)
    }

    pub fn is_master_track(&self) -> bool {
        self.load_and_check_if_necessary_or_complain();
        let t = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_track_number(self.raw())
        };
        t == Some(TrackLocation::MasterTrack)
    }

    pub fn project(&self) -> Project {
        if self.rea_project.get().is_none() {
            self.load_if_necessary_or_complain();
        }
        self.project_unchecked()
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        match (&self.media_track.get(), &other.media_track.get()) {
            (Some(self_media_track), Some(other_media_track)) => {
                self_media_track == other_media_track
            }
            _ => self.guid() == other.guid(),
        }
    }
}

impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(t) = self.media_track.get() {
            t.hash(state);
        } else {
            self.guid.hash(state);
        }
    }
}

pub fn get_media_track_guid(media_track: MediaTrack) -> Guid {
    let internal = unsafe {
        Reaper::get()
            .medium_reaper()
            .get_set_media_track_info_get_guid(media_track)
    };
    Guid::new(internal)
}

// In REAPER < 5.95 this returns nullptr. That means we might need to use findContainingProject
// logic at a later point.
fn get_track_project_raw(media_track: MediaTrack) -> Option<ReaProject> {
    unsafe {
        Reaper::get()
            .medium_reaper()
            .get_set_media_track_info_get_project(media_track)
    }
}

fn get_auto_arm_chunk_line(chunk: &Chunk) -> Option<ChunkRegion> {
    chunk.region().find_line_starting_with("AUTO_RECARM 1")
}
