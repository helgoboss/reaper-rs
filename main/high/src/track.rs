use std::cell::Cell;

use crate::fx::{get_index_from_query_index, Fx};
use crate::fx_chain::FxChain;
use crate::guid::Guid;
use crate::track_route::TrackRoute;

use crate::{
    Chunk, ChunkRegion, Item, Pan, Project, Reaper, ReaperError, SendPartnerType,
    TrackRoutePartner, Width,
};

use crate::error::ReaperResult;
use either::Either;
use enumflags2::BitFlags;
use helgoboss_midi::Channel;
use reaper_medium::NotificationBehavior::NotifyAll;
use reaper_medium::ProjectContext::Proj;
use reaper_medium::SendTarget::OtherTrack;
use reaper_medium::TrackAttributeKey::{RecArm, RecInput, RecMon, Selected, Solo};
use reaper_medium::{
    AutomationMode, BeatAttachMode, ChunkCacheHint, GangBehavior, GlobalAutomationModeOverride,
    InputMonitoringMode, MediaTrack, NativeColorValue, NotificationBehavior, Progress, ReaProject,
    ReaperFunctionError, ReaperPanValue, ReaperString, ReaperStringArg, ReaperVolumeValue,
    ReaperWidthValue, RecordArmMode, RecordingInput, RecordingMode, RgbColor, SetTrackUiFlags,
    SoloMode, TrackArea, TrackAttributeKey, TrackLocation, TrackMuteOperation, TrackMuteState,
    TrackPolarity, TrackPolarityOperation, TrackRecArmOperation, TrackSendCategory,
    TrackSendDirection, TrackSoloOperation, ValueChange,
};
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::iter;

pub const MAX_TRACK_CHUNK_SIZE: u32 = 20_000_000;

#[derive(Clone, Debug, Eq)]
// TODO-low Reconsider design. Maybe don't do that interior mutability stuff. By moving from lazy to
//  eager (determining rea_project and media_track at construction time). This sounds good. We
//  should provide 2 types. A light-weight one which doesn't save the GUID and one that saves it
//  (for scenarios where we want to keep the object around). All the methods should be on the
//  light-weight one and the heavy-weight one should have a method to return the light-weight.
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

unsafe impl Send for Track {}

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
                .get_set_media_track_info_set_name(self.raw_internal(), name);
        }
    }

    pub fn item_count(&self) -> u32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        self.item_count_internal()
    }

    pub fn delete_all_items(&self) -> Result<(), ReaperFunctionError> {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get().medium_reaper();
        let raw = self.raw_internal();
        for i in (0..self.item_count_internal()).rev() {
            unsafe {
                let Some(item) = reaper.get_track_media_item(raw, i) else {
                    continue;
                };
                reaper.delete_track_media_item(raw, item)?;
            }
        }
        Ok(())
    }

    fn item_count_internal(&self) -> u32 {
        unsafe {
            Reaper::get()
                .medium_reaper
                .count_track_media_items(self.raw_internal())
        }
    }

    pub fn items(&self) -> impl ExactSizeIterator<Item = Item> + 'static {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Either::Left(iter::empty());
        }
        let raw = self.raw_internal();
        let iter = (0..self.item_count_internal()).map(move |i| {
            let media_item = unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_media_item(raw, i as _)
                    .unwrap()
            };
            Item::new(media_item)
        });
        Either::Right(iter)
    }

    pub fn add_item(&self) -> Result<Item, ReaperFunctionError> {
        self.load_and_check_if_necessary_or_complain();
        let raw_item = unsafe {
            Reaper::get()
                .medium_reaper()
                .add_media_item_to_track(self.raw_unchecked())?
        };
        Ok(Item::new(raw_item))
    }

    // TODO-low It's really annoying to always have to unwrap an option even if we know this is not
    //  a master track. Maybe we should have different types: Track, MasterTrack, NormalTrack
    pub fn name(&self) -> Option<ReaperString> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_name(self.raw_internal(), |n| n.to_owned())
        }
    }

    pub fn custom_color(&self) -> Option<RgbColor> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        let reaper = Reaper::get().medium_reaper();
        let res = unsafe { reaper.get_set_media_track_info_get_custom_color(self.raw_internal()) };
        if !res.is_used {
            return None;
        }
        Some(reaper.color_from_native(res.color))
    }

    pub fn set_custom_color(&self, color: Option<RgbColor>) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get().medium_reaper();
        let value = match color {
            None => NativeColorValue {
                color: Default::default(),
                is_used: false,
            },
            Some(c) => NativeColorValue {
                color: reaper.color_to_native(c),
                is_used: true,
            },
        };
        unsafe { reaper.get_set_media_track_info_set_custom_color(self.raw_internal(), value) };
    }

    pub fn set_anticipative_fx_enabled(&self, value: bool) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        let perf_flags = self.perf_flags_internal();
        let new_perf_flags = if value {
            perf_flags | 2
        } else {
            perf_flags & !2
        };
        self.set_perf_flags_internal(new_perf_flags)
    }

    fn perf_flags_internal(&self) -> u32 {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_media_track_info_value(self.raw_internal(), TrackAttributeKey::PerfFlags)
                as u32
        }
    }

    fn set_perf_flags_internal(&self, value: u32) -> ReaperResult<()> {
        unsafe {
            Reaper::get().medium_reaper.set_media_track_info_value(
                self.raw_internal(),
                TrackAttributeKey::PerfFlags,
                value as f64,
            )?;
        }
        Ok(())
    }

    pub fn input_monitoring_mode(&self) -> InputMonitoringMode {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return InputMonitoringMode::Normal;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_rec_mon(self.raw_internal())
        }
    }

    pub fn set_input_monitoring_mode(
        &self,
        mode: InputMonitoringMode,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        self.load_and_check_if_necessary_or_complain();
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().SetTrackUIInputMonitor.is_some() {
            unsafe {
                reaper.set_track_ui_input_monitor(
                    self.raw_unchecked(),
                    mode,
                    build_track_ui_flags(gang_behavior, grouping_behavior),
                )
            };
        } else {
            unsafe {
                reaper.csurf_on_input_monitoring_change_ex(
                    self.raw_unchecked(),
                    mode,
                    gang_behavior,
                );
            }
        }
    }

    pub fn recording_input(&self) -> Option<RecordingInput> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_rec_input(self.raw_internal())
        }
    }

    pub fn midi_input_channel_mapping(&self) -> Option<Channel> {
        self.load_and_check_if_necessary_or_err().ok()?;
        let val = unsafe {
            Reaper::get().medium_reaper().get_media_track_info_value(
                self.raw_internal(),
                TrackAttributeKey::MidiInputChanMap,
            )
        };
        if val < 0.0 {
            None
        } else {
            Some(Channel::new(val as u8))
        }
    }

    pub fn set_midi_input_channel_mapping(&self, channel: Option<Channel>) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        unsafe {
            Reaper::get().medium_reaper().set_media_track_info_value(
                self.raw_internal(),
                TrackAttributeKey::MidiInputChanMap,
                channel.map(|ch| ch.get() as f64).unwrap_or(-1.0),
            )?;
        }
        Ok(())
    }

    pub fn set_recording_input(&self, input: Option<RecordingInput>) {
        self.load_and_check_if_necessary_or_complain();
        let rec_input_index = match input {
            None => -1,
            Some(ri) => ri.to_raw(),
        };
        let _ = unsafe {
            Reaper::get().medium_reaper().set_media_track_info_value(
                self.raw_unchecked(),
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
                .get_media_track_info_value(self.raw_unchecked(), RecMon)
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

    pub fn recording_mode(&self) -> RecordingMode {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_rec_mode(self.raw_internal())
        }
    }

    pub fn set_recording_mode(&self, value: RecordingMode) {
        self.load_and_check_if_necessary_or_complain();
        let _ = unsafe {
            Reaper::get().medium_reaper().set_media_track_info_value(
                self.raw_unchecked(),
                TrackAttributeKey::RecMode,
                value.to_raw() as f64,
            )
        };
        // Only for triggering notification (as manual setting the rec input would also trigger it)
        // This doesn't work for other surfaces but they are also not interested in record input
        // changes.
        let _rec_mon = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_unchecked(), RecMon)
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

    /// This one also ensures the track is valid.
    pub fn raw(&self) -> ReaperResult<MediaTrack> {
        self.load_and_check_if_necessary_or_err()?;
        Ok(self.raw_internal())
    }

    /// This one **doesn't** ensure the track is valid.
    pub(crate) fn raw_unchecked(&self) -> MediaTrack {
        unsafe {
            self.load_if_necessary_or_complain_unchecked();
        }
        self.raw_internal()
    }

    fn raw_internal(&self) -> MediaTrack {
        self.media_track.get().unwrap()
    }

    pub fn pan(&self) -> Pan {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Pan::from_reaper_value(ReaperPanValue::CENTER);
        }
        // It's important that we don't query D_PAN because that returns the wrong value in case an
        // envelope is written
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_vol_pan(self.raw_internal())
                .expect("couldn't get vol/pan")
        };
        Pan::from_reaper_value(result.pan)
    }

    /// Sets the pan using the best and most full-featured function available and informs control surfaces about it.
    pub fn set_pan_smart(
        &self,
        value: ReaperPanValue,
        opts: TrackSetSmartOpts,
    ) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        let track = self.raw_internal();
        let reaper = Reaper::get().medium_reaper();
        let resulting_value = if self.project() == Reaper::get().current_project() {
            if reaper.low().pointers().SetTrackUIPan.is_some() {
                unsafe {
                    reaper.set_track_ui_pan(
                        track,
                        ValueChange::Absolute(value),
                        opts.progress(),
                        build_track_ui_flags(opts.gang_behavior, opts.grouping_behavior),
                    )
                }
            } else {
                unsafe {
                    reaper.csurf_on_pan_change_ex(
                        track,
                        ValueChange::Absolute(value),
                        opts.gang_behavior,
                    )
                }
            }
        } else {
            // ReaLearn #283
            unsafe {
                let _ =
                    reaper.set_media_track_info_value(track, TrackAttributeKey::Pan, value.get());
            }
            value
        };
        // Setting the pan programmatically doesn't trigger SetSurfacePan for control surfaces so
        // we need to notify manually
        unsafe {
            reaper.csurf_set_surface_pan(track, resulting_value, NotifyAll);
        }
        Ok(())
    }

    /// Sets the given track's pan, also supports relative changes and gang.
    pub fn csurf_on_pan_change_ex(
        &self,
        value_change: ValueChange<ReaperPanValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperResult<ReaperPanValue> {
        self.load_and_check_if_necessary_or_err()?;
        let value = unsafe {
            Reaper::get().medium_reaper.csurf_on_pan_change_ex(
                self.raw_internal(),
                value_change,
                gang_behavior,
            )
        };
        Ok(value)
    }

    /// Informs control surfaces that the given track's pan has changed.
    pub fn csurf_set_surface_pan(
        &self,
        pan: ReaperPanValue,
        notification_behavior: NotificationBehavior,
    ) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        unsafe {
            Reaper::get().medium_reaper.csurf_set_surface_pan(
                self.raw_internal(),
                pan,
                notification_behavior,
            )
        };
        Ok(())
    }

    pub fn width(&self) -> Width {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Width::from_reaper_value(ReaperWidthValue::CENTER);
        }
        // It's important that we don't query D_WIDTH because that returns the wrong value in case
        // an envelope is written
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_pan(self.raw_internal())
                .expect("couldn't get pan/width")
        };
        Width::from_reaper_value(result.pan_2.as_width_value())
    }

    /// Sets the width using the best and most full-featured function available and informs control surfaces about it.
    pub fn set_width_smart(
        &self,
        value: ReaperWidthValue,
        opts: TrackSetSmartOpts,
    ) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        let track = self.raw_internal();
        let reaper = Reaper::get().medium_reaper();
        if self.project() == Reaper::get().current_project() {
            if reaper.low().pointers().SetTrackUIWidth.is_some() {
                unsafe {
                    reaper.set_track_ui_width(
                        track,
                        ValueChange::Absolute(value),
                        opts.progress(),
                        build_track_ui_flags(opts.gang_behavior, opts.grouping_behavior),
                    );
                }
            } else {
                unsafe {
                    reaper.csurf_on_width_change_ex(
                        track,
                        ValueChange::Absolute(value),
                        opts.gang_behavior,
                    );
                }
            }
        } else {
            // ReaLearn #283
            let _ = unsafe {
                reaper.set_media_track_info_value(track, TrackAttributeKey::Width, value.get())
            };
        }
        // Setting the width programmatically doesn't trigger SetSurfacePan for control surfaces
        // so we need to notify manually. There's no CSurf_SetSurfaceWidth, so we just retrigger
        // CSurf_SetSurfacePan.
        unsafe {
            let vol_pan = reaper.get_track_ui_vol_pan(track)?;
            Reaper::get()
                .medium_reaper()
                .csurf_set_surface_pan(track, vol_pan.pan, NotifyAll);
        }
        Ok(())
    }

    /// Sets the given track's width, also supports relative changes and gang.
    pub fn csurf_on_width_change_ex(
        &self,
        value_change: ValueChange<ReaperWidthValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperResult<ReaperWidthValue> {
        self.load_and_check_if_necessary_or_err()?;
        let value = unsafe {
            Reaper::get().medium_reaper.csurf_on_width_change_ex(
                self.raw_internal(),
                value_change,
                gang_behavior,
            )
        };
        Ok(value)
    }

    pub fn folder_depth_change(&self) -> i32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_internal(), TrackAttributeKey::FolderDepth)
                as i32
        }
    }

    pub fn channel_count(&self) -> u32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_internal(), TrackAttributeKey::Nchan)
        };
        result as _
    }

    pub fn volume(&self) -> ReaperVolumeValue {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return ReaperVolumeValue::MIN;
        }
        // It's important that we don't query D_VOL because that returns the wrong value in case an
        // envelope is written
        let result = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_vol_pan(self.raw_internal())
                .expect("Couldn't get vol/pan")
        };
        result.volume
    }

    /// Sets the volume using the best and most full-featured function available and informs control surfaces about it.
    pub fn set_volume_smart(
        &self,
        value: ReaperVolumeValue,
        opts: TrackSetSmartOpts,
    ) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        let track = self.raw_internal();
        let reaper = Reaper::get().medium_reaper();
        let resulting_value = if self.project() == Reaper::get().current_project() {
            // Why we use SetTrackUIVolume or CSurf_OnVolumeChangeEx and not the others:
            //
            // - Setting D_VOL directly via `SetMediaTrackInfo_Value` will not work for writing
            //   automation.
            // - `CSurf_SetSurfaceVolume` seems to only inform control surfaces, doesn't actually
            //   set the volume.
            if reaper.low().pointers().SetTrackUIVolume.is_some() {
                unsafe {
                    reaper.set_track_ui_volume(
                        track,
                        ValueChange::Absolute(value),
                        opts.progress(),
                        build_track_ui_flags(opts.gang_behavior, opts.grouping_behavior),
                    )
                }
            } else {
                // Downsides of using this function:
                //
                // - CSurf_OnVolumeChangeEx has a slightly lower precision than setting D_VOL directly.
                //   The return value reflects the cropped value. However, the precision became much
                //   better with REAPER 5.28.
                // - In automation mode "Touch" this leads to jumps.
                // - Doesn't support grouping
                unsafe {
                    reaper.csurf_on_volume_change_ex(
                        track,
                        ValueChange::Absolute(value),
                        opts.gang_behavior,
                    )
                }
            }
        } else {
            // ReaLearn #283
            unsafe {
                let _ =
                    reaper.set_media_track_info_value(track, TrackAttributeKey::Vol, value.get());
            }
            value
        };
        // Setting the volume programmatically doesn't inform control surfaces - including our own
        // surfaces which are important for feedback. So use the following to notify manually.
        unsafe {
            reaper.csurf_set_surface_volume(track, resulting_value, NotifyAll);
        }
        Ok(())
    }

    pub fn set_track_ui_volume(
        &self,
        value_change: ValueChange<ReaperVolumeValue>,
        progress: Progress,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) -> ReaperResult<ReaperVolumeValue> {
        let reaper = &Reaper::get().medium_reaper;
        reaper
            .low()
            .pointers()
            .SetTrackUIVolume
            .ok_or("REAPER version too old to support SetTrackUIVolume")?;
        self.load_and_check_if_necessary_or_err()?;
        let value = unsafe {
            reaper.set_track_ui_volume(
                self.raw_internal(),
                value_change,
                progress,
                build_track_ui_flags(gang_behavior, grouping_behavior),
            )
        };
        Ok(value)
    }

    /// Sets the given track's volume, also supports relative changes and gang.
    pub fn csurf_on_volume_change_ex(
        &self,
        value_change: ValueChange<ReaperVolumeValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperResult<ReaperVolumeValue> {
        self.load_and_check_if_necessary_or_err()?;
        let value = unsafe {
            Reaper::get().medium_reaper.csurf_on_volume_change_ex(
                self.raw_internal(),
                value_change,
                gang_behavior,
            )
        };
        Ok(value)
    }

    /// Informs control surfaces that the given track's volume has changed.
    pub fn csurf_set_surface_volume(
        &self,
        volume: ReaperVolumeValue,
        notification_behavior: NotificationBehavior,
    ) -> ReaperResult<()> {
        self.load_and_check_if_necessary_or_err()?;
        unsafe {
            Reaper::get().medium_reaper.csurf_set_surface_volume(
                self.raw_internal(),
                volume,
                notification_behavior,
            )
        };
        Ok(())
    }

    pub fn scroll_mixer(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_mixer_scroll(self.raw_unchecked());
        }
    }

    pub fn location(&self) -> TrackLocation {
        self.load_and_check_if_necessary_or_complain();
        self.location_internal()
    }

    fn location_internal(&self) -> TrackLocation {
        // TODO-low The following returns None if we query the number of a track in another project
        //  Try to find a working solution!
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_track_number(self.raw_internal())
                .unwrap()
        }
    }

    pub fn index(&self) -> Option<u32> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        use TrackLocation::*;
        match self.location_internal() {
            MasterTrack => None,
            NormalTrack(idx) => Some(idx),
        }
    }

    pub fn has_auto_arm_enabled(&self) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        self.has_auto_arm_enabled_internal()
    }

    fn has_auto_arm_enabled_internal(&self) -> bool {
        if let Ok(line) = self.auto_arm_chunk_line() {
            line.is_some()
        } else {
            false
        }
    }

    #[allow(clippy::float_cmp)]
    pub fn is_armed(&self, support_auto_arm: bool) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        if support_auto_arm && self.has_auto_arm_enabled_internal() {
            self.is_selected_internal()
        } else {
            let recarm = unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_media_track_info_value(self.raw_internal(), RecArm)
            };
            recarm == 1.0
        }
    }

    pub fn beat_attach_mode(&self) -> Option<BeatAttachMode> {
        unsafe {
            Reaper::get()
                .medium_reaper
                .get_set_media_track_info_get_beat_attach_mode(self.raw_internal())
        }
    }

    pub fn parent_send_enabled(&self) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_internal(), TrackAttributeKey::MainSend)
                > 0.0
        }
    }

    pub fn set_parent_send_enabled(&self, parent_send: bool) {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_media_track_info_value(
                    self.raw_unchecked(),
                    TrackAttributeKey::MainSend,
                    if parent_send { 1.0 } else { 0.0 },
                )
                .unwrap();
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn arm(
        &self,
        support_auto_arm: bool,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.select();
        } else if self.project() == Reaper::get().current_project() {
            self.set_arm_state_internal(RecordArmMode::Armed, gang_behavior, grouping_behavior);
            // If track was auto-armed before, this would just have switched off the auto-arm
            // but not actually armed the track. Therefore we check if it's
            // really armed and if not we do it again.
            let recarm = unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_media_track_info_value(self.raw_unchecked(), RecArm)
            };
            #[allow(clippy::float_cmp)]
            {
                if recarm != 1.0 {
                    self.set_arm_state_internal(
                        RecordArmMode::Armed,
                        gang_behavior,
                        grouping_behavior,
                    );
                }
            }
        } else {
            // ReaLearn #283
            self.set_prop_enabled(TrackAttributeKey::RecArm, true);
        }
    }

    // If supportAutoArm is false, auto-arm mode is disabled if it has been enabled before
    pub fn disarm(
        &self,
        support_auto_arm: bool,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        if support_auto_arm && self.has_auto_arm_enabled() {
            self.unselect();
        } else if self.project() == Reaper::get().current_project() {
            self.set_arm_state_internal(RecordArmMode::Unarmed, gang_behavior, grouping_behavior);
        } else {
            // ReaLearn #283
            self.set_prop_enabled(TrackAttributeKey::RecArm, false);
        }
    }

    pub fn set_armed(
        &self,
        armed: bool,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        let arm_mode = if armed {
            RecordArmMode::Armed
        } else {
            RecordArmMode::Unarmed
        };
        self.set_arm_state_internal(arm_mode, gang_behavior, grouping_behavior);
    }

    fn set_arm_state_internal(
        &self,
        mode: RecordArmMode,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().SetTrackUIRecArm.is_some() {
            unsafe {
                reaper.set_track_ui_rec_arm(
                    self.raw_unchecked(),
                    TrackRecArmOperation::Set(mode),
                    build_track_ui_flags(gang_behavior, grouping_behavior),
                );
            }
        } else {
            unsafe {
                reaper.csurf_on_rec_arm_change_ex(self.raw_unchecked(), mode, gang_behavior);
            }
        }
    }

    pub fn enable_auto_arm(&self) -> ReaperResult<()> {
        let mut chunk = self.chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode)?;
        if get_auto_arm_chunk_line(&chunk).is_some() {
            return Ok(());
        }
        let was_armed_before = self.is_armed(true);
        chunk.insert_after_region_as_block(&chunk.region().first_line(), "AUTO_RECARM 1");
        self.set_chunk(chunk)?;
        if was_armed_before {
            self.arm(
                true,
                GangBehavior::DenyGang,
                GroupingBehavior::PreventGrouping,
            );
        } else {
            self.disarm(
                true,
                GangBehavior::DenyGang,
                GroupingBehavior::PreventGrouping,
            );
        }
        Ok(())
    }

    pub fn disable_auto_arm(&self) -> ReaperResult<()> {
        let chunk = {
            let auto_arm_chunk_line = match self.auto_arm_chunk_line()? {
                None => return Ok(()),
                Some(l) => l,
            };
            let mut chunk = auto_arm_chunk_line.parent_chunk();
            chunk.delete_region(&auto_arm_chunk_line);
            chunk
        };
        self.set_chunk(chunk)
    }

    pub fn phase_is_inverted(&self) -> bool {
        self.prop_is_enabled(TrackAttributeKey::Phase)
    }

    pub fn set_phase_inverted(
        &self,
        polarity: TrackPolarity,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().SetTrackUIPolarity.is_some() {
            unsafe {
                reaper.set_track_ui_polarity(
                    self.raw_unchecked(),
                    TrackPolarityOperation::Set(polarity),
                    build_track_ui_flags(gang_behavior, grouping_behavior),
                );
            }
        } else {
            self.set_prop_numeric_value(TrackAttributeKey::Phase, polarity.to_raw() as f64);
        }
    }

    fn set_prop_enabled(&self, key: TrackAttributeKey, enabled: bool) {
        self.set_prop_numeric_value(key, if enabled { 1.0 } else { 0.0 });
    }

    fn prop_is_enabled(&self, key: TrackAttributeKey) -> bool {
        self.prop_numeric_value(key) > 0.0
    }

    fn set_prop_numeric_value(&self, key: TrackAttributeKey, value: f64) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            let _ = Reaper::get().medium_reaper().set_media_track_info_value(
                self.raw_unchecked(),
                key,
                value,
            );
        }
    }

    fn prop_numeric_value(&self, key: TrackAttributeKey) -> f64 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0.0;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_internal(), key)
        }
    }

    pub fn set_shown(&self, area: TrackArea, value: bool) {
        self.set_shown_without_updating_ui(area, value);
        // The following is actually not necessary if this was the master track
        let reaper = &Reaper::get().medium_reaper;
        match area {
            TrackArea::Tcp => reaper.track_list_adjust_windows_minor(),
            TrackArea::Mcp => reaper.track_list_adjust_windows_major(),
        };
    }

    pub fn set_shown_without_updating_ui(&self, area: TrackArea, value: bool) {
        let reaper = &Reaper::get().medium_reaper;
        if self.is_master_track() {
            let mut flags = reaper.get_master_track_visibility();
            if (value && area == TrackArea::Tcp) || (!value && area == TrackArea::Mcp) {
                flags.insert(area);
            } else {
                flags.remove(area);
            };
            reaper.set_master_track_visibility(flags);
        } else {
            unsafe {
                let _ = reaper.set_media_track_info_value(
                    self.raw_unchecked(),
                    get_show_attribute_key(area),
                    if value { 1.0 } else { 0.0 },
                );
            }
        }
    }

    pub fn is_shown(&self, area: TrackArea) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        let reaper = &Reaper::get().medium_reaper;
        if self.is_master_track_internal() {
            let has_flag = reaper.get_master_track_visibility().contains(area);
            match area {
                TrackArea::Tcp => has_flag,
                TrackArea::Mcp => !has_flag,
            }
        } else {
            unsafe {
                reaper.get_media_track_info_value(self.raw_internal(), get_show_attribute_key(area))
                    > 0.0
            }
        }
    }

    #[allow(clippy::float_cmp)]
    pub fn is_muted(&self) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        let mute = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_ui_mute(self.raw_internal())
        };
        mute.unwrap_or(false)
    }

    pub fn mute(&self, gang_behavior: GangBehavior, grouping_behavior: GroupingBehavior) {
        self.set_mute_internal(TrackMuteState::Mute, gang_behavior, grouping_behavior);
    }

    pub fn unmute(&self, gang_behavior: GangBehavior, grouping_behavior: GroupingBehavior) {
        self.set_mute_internal(TrackMuteState::Unmute, gang_behavior, grouping_behavior);
    }

    pub fn set_mute(
        &self,
        mute: bool,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        let state = if mute {
            TrackMuteState::Mute
        } else {
            TrackMuteState::Unmute
        };
        self.set_mute_internal(state, gang_behavior, grouping_behavior);
    }

    fn set_mute_internal(
        &self,
        state: TrackMuteState,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        if self.project() == Reaper::get().current_project() {
            self.load_and_check_if_necessary_or_complain();
            let reaper = Reaper::get().medium_reaper();
            if reaper.low().pointers().SetTrackUIMute.is_some() {
                unsafe {
                    reaper.set_track_ui_mute(
                        self.raw_unchecked(),
                        TrackMuteOperation::Set(state),
                        build_track_ui_flags(gang_behavior, grouping_behavior),
                    )
                };
            } else {
                unsafe {
                    reaper.csurf_on_mute_change_ex(
                        self.raw_unchecked(),
                        state == TrackMuteState::Mute,
                        gang_behavior,
                    )
                };
            }
        } else {
            // ReaLearn #283
            self.set_prop_numeric_value(TrackAttributeKey::Mute, state.to_raw() as f64);
        }
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_mute(
                self.raw_unchecked(),
                state == TrackMuteState::Mute,
                NotifyAll,
            );
        }
    }

    pub fn is_solo(&self) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        let solo = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_internal(), Solo)
        };
        solo > 0.0
    }

    pub fn solo(&self, gang_behavior: GangBehavior, grouping_behavior: GroupingBehavior) {
        self.set_solo(true, gang_behavior, grouping_behavior);
    }

    pub fn unsolo(&self, gang_behavior: GangBehavior, grouping_behavior: GroupingBehavior) {
        self.set_solo(false, gang_behavior, grouping_behavior);
    }

    pub fn solo_mode(&self) -> SoloMode {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return SoloMode::Off;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_solo(self.raw_internal())
        }
    }

    pub fn set_solo_mode(&self, mode: SoloMode) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_set_solo(self.raw_unchecked(), mode);
        }
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_solo(
                self.raw_unchecked(),
                mode.to_raw() > 0,
                NotifyAll,
            );
        }
    }

    fn set_solo(
        &self,
        solo: bool,
        gang_behavior: GangBehavior,
        grouping_behavior: GroupingBehavior,
    ) {
        if self.project() == Reaper::get().current_project() {
            self.load_and_check_if_necessary_or_complain();
            let reaper = Reaper::get().medium_reaper();
            if reaper.low().pointers().SetTrackUIMute.is_some() {
                let operation = if solo {
                    TrackSoloOperation::SetSolo
                } else {
                    TrackSoloOperation::UnsetSolo
                };
                unsafe {
                    reaper.set_track_ui_solo(
                        self.raw_unchecked(),
                        operation,
                        build_track_ui_flags(gang_behavior, grouping_behavior),
                    )
                };
            } else {
                unsafe {
                    reaper.csurf_on_solo_change_ex(self.raw_unchecked(), solo, gang_behavior)
                };
            }
        } else {
            // ReaLearn #283
            self.set_prop_enabled(TrackAttributeKey::Solo, solo);
        }
        unsafe {
            Reaper::get().medium_reaper().csurf_set_surface_solo(
                self.raw_unchecked(),
                solo,
                NotifyAll,
            );
        }
    }

    pub fn fx_is_enabled(&self) -> bool {
        self.prop_is_enabled(TrackAttributeKey::FxEn)
    }

    pub fn enable_fx(&self) {
        self.set_fx_is_enabled(true);
    }

    pub fn disable_fx(&self) {
        self.set_fx_is_enabled(false);
    }

    fn set_fx_is_enabled(&self, enabled: bool) {
        self.set_prop_enabled(TrackAttributeKey::FxEn, enabled);
    }

    fn auto_arm_chunk_line(&self) -> Result<Option<ChunkRegion>, &'static str> {
        let chunk = self.chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::UndoMode)?;
        Ok(get_auto_arm_chunk_line(&chunk))
    }

    // Attention! If you pass undoIsOptional = true it's faster but it returns a chunk that contains
    // weird FXID_NEXT (in front of FX tag) instead of FXID (behind FX tag). So FX chunk code
    // should be double checked then.
    pub fn chunk(
        &self,
        max_chunk_size: u32,
        undo_is_optional: ChunkCacheHint,
    ) -> Result<Chunk, &'static str> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Err("track not available, so chunk neither");
        }
        self.chunk_internal(max_chunk_size, undo_is_optional)
    }

    fn chunk_internal(
        &self,
        max_chunk_size: u32,
        undo_is_optional: ChunkCacheHint,
    ) -> Result<Chunk, &'static str> {
        let chunk_content = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_state_chunk(self.raw_internal(), max_chunk_size, undo_is_optional)
                .map_err(|_| "Couldn't load track chunk")?
        };
        Ok(chunk_content.into())
    }

    // TODO-low Report possible error
    pub fn set_chunk(&self, chunk: Chunk) -> ReaperResult<()> {
        let string: String = chunk
            .try_into()
            .map_err(|_| ReaperError::new("unfortunate"))?;
        let _ = unsafe {
            Reaper::get().medium_reaper().set_track_state_chunk(
                self.raw_unchecked(),
                string,
                ChunkCacheHint::UndoMode,
            )
        };
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    pub fn is_selected(&self) -> bool {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        self.is_selected_internal()
    }

    fn is_selected_internal(&self) -> bool {
        let selected = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_media_track_info_value(self.raw_unchecked(), Selected)
        };
        selected == 1.0
    }

    pub fn select(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_track_selected(self.raw_unchecked(), true);
        }
    }

    pub fn select_exclusively(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_only_track_selected(Some(self.raw_unchecked()));
        }
    }

    pub fn unselect(&self) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_track_selected(self.raw_unchecked(), false);
        }
    }

    pub fn receive_count(&self) -> u32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        self.receive_count_internal()
    }

    fn receive_count_internal(&self) -> u32 {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_num_sends(self.raw_internal(), TrackSendCategory::Receive)
        }
    }

    pub fn send_count(&self) -> u32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        self.send_count_internal()
    }

    fn send_count_internal(&self) -> u32 {
        self.hw_send_count_internal() + self.typed_send_count_internal(SendPartnerType::Track)
    }

    pub fn typed_send_count(&self, partner_type: SendPartnerType) -> u32 {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return 0;
        }
        self.typed_send_count_internal(partner_type)
    }

    fn typed_send_count_internal(&self, partner_type: SendPartnerType) -> u32 {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_num_sends(self.raw_internal(), partner_type.to_category())
        }
    }

    pub fn add_send_to(&self, destination_track: &Track) -> TrackRoute {
        // TODO-low Check how this behaves if send already exists
        let send_index = unsafe {
            Reaper::get().medium_reaper().create_track_send(
                self.raw_unchecked(),
                OtherTrack(destination_track.raw_unchecked()),
            )
        }
        .unwrap();
        let hw_send_count = self.hw_send_count_internal();
        TrackRoute::new(
            self.clone(),
            TrackSendDirection::Send,
            hw_send_count + send_index,
        )
    }

    pub fn receives(&self) -> impl ExactSizeIterator<Item = TrackRoute> + '_ {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Either::Left(iter::empty());
        }
        let iter = (0..self.receive_count_internal())
            .map(move |i| TrackRoute::new(self.clone(), TrackSendDirection::Receive, i));
        Either::Right(iter)
    }

    pub fn sends(&self) -> impl ExactSizeIterator<Item = TrackRoute> + '_ {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Either::Left(iter::empty());
        }
        let iter = (0..self.send_count_internal())
            .map(move |i| TrackRoute::new(self.clone(), TrackSendDirection::Send, i));
        Either::Right(iter)
    }

    pub fn typed_sends(
        &self,
        partner_type: SendPartnerType,
    ) -> impl ExactSizeIterator<Item = TrackRoute> + '_ {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return Either::Left(iter::empty());
        }
        Either::Right(self.typed_sends_internal(partner_type))
    }

    fn typed_sends_internal(
        &self,
        partner_type: SendPartnerType,
    ) -> impl ExactSizeIterator<Item = TrackRoute> + '_ {
        let hw_send_count = self.hw_send_count_internal();
        let (from, count) = match partner_type {
            SendPartnerType::Track => (
                hw_send_count,
                self.typed_send_count_internal(SendPartnerType::Track),
            ),
            SendPartnerType::HardwareOutput => (0, hw_send_count),
        };
        let until = from + count;
        (from..until).map(move |i| TrackRoute::new(self.clone(), TrackSendDirection::Send, i))
    }

    fn hw_send_count_internal(&self) -> u32 {
        self.typed_send_count_internal(SendPartnerType::HardwareOutput)
    }

    pub fn receive_by_index(&self, index: u32) -> Option<TrackRoute> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        if index >= self.receive_count_internal() {
            return None;
        }
        let route = TrackRoute::new(self.clone(), TrackSendDirection::Receive, index);
        Some(route)
    }

    pub fn send_by_index(&self, index: u32) -> Option<TrackRoute> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        if index >= self.send_count_internal() {
            return None;
        }
        let route = TrackRoute::new(self.clone(), TrackSendDirection::Send, index);
        Some(route)
    }

    pub fn typed_send_by_index(
        &self,
        partner_type: SendPartnerType,
        index: u32,
    ) -> Option<TrackRoute> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        if index >= self.typed_send_count_internal(partner_type) {
            return None;
        }
        let actual_index = match partner_type {
            SendPartnerType::Track => self.hw_send_count_internal() + index,
            SendPartnerType::HardwareOutput => index,
        };
        let route = TrackRoute::new(self.clone(), TrackSendDirection::Send, actual_index);
        Some(route)
    }

    pub fn find_receive_by_source_track(&self, source_track: &Track) -> Option<TrackRoute> {
        self.receives().find(|s| match s.partner() {
            Some(TrackRoutePartner::Track(t)) => t == *source_track,
            _ => false,
        })
    }

    pub fn find_send_by_destination_track(&self, destination_track: &Track) -> Option<TrackRoute> {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return None;
        }
        self.typed_sends_internal(SendPartnerType::Track)
            .find(|s| match s.partner() {
                Some(TrackRoutePartner::Track(t)) => t == *destination_track,
                _ => false,
            })
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
        unsafe {
            self.load_if_necessary_or_complain_unchecked();
        }
        self.complain_if_not_valid();
    }

    pub(crate) fn load_and_check_if_necessary_or_err(&self) -> ReaperResult<()> {
        unsafe {
            self.load_if_necessary_or_err_unchecked()?;
        }
        self.err_if_not_valid()?;
        Ok(())
    }

    /// # Safety
    ///
    /// This is technically safe but I want other methods to really think twice before using this method,
    /// because unlike `load_and_check_if_necessary_or_err`, this one doesn't check the validity of methods.
    /// Subsequently, this can crash REAPER. This confusion was leading to
    /// https://github.com/helgoboss/helgobox/issues/1304, for example.
    unsafe fn load_if_necessary_or_complain_unchecked(&self) {
        self.load_if_necessary_or_err_unchecked().unwrap();
    }

    /// # Safety
    ///
    /// This is technically safe but I want other methods to really think twice before using this method,
    /// because unlike `load_and_check_if_necessary_or_complain`, this one doesn't check the validity of methods.
    /// Subsequently, this can crash REAPER. This confusion was leading to
    /// https://github.com/helgoboss/helgobox/issues/1304, for example.
    unsafe fn load_if_necessary_or_err_unchecked(&self) -> Result<(), &'static str> {
        if self.media_track.get().is_none() && !self.load_by_guid() {
            Err("Track not loadable")
        } else {
            Ok(())
        }
    }

    fn complain_if_not_valid(&self) {
        self.err_if_not_valid().unwrap();
    }

    fn err_if_not_valid(&self) -> Result<(), &'static str> {
        if self.is_valid() {
            Ok(())
        } else {
            Err("Track not available")
        }
    }

    /// Precondition: mediaTrack_ must be filled!
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

    /// Precondition: mediaTrack_ must be filled!
    fn attempt_to_fill_project_if_necessary(&self) {
        if self.rea_project.get().is_none() {
            self.rea_project.replace(self.find_containing_project_raw());
        }
    }

    pub fn guid(&self) -> &Guid {
        &self.guid
    }

    pub fn set_guid(&mut self, guid: Guid) {
        self.load_and_check_if_necessary_or_complain();
        self.guid = guid;
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_set_guid(self.raw_internal(), &guid.to_raw());
        }
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
                self.media_track.replace(Some(t.raw_unchecked()));
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

    /// Precondition: mediaTrack_ must be filled!
    ///
    /// Should be rather cheap on the happy path because we try with the current project first!
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

    pub fn set_automation_mode(&self, mode: AutomationMode) {
        self.load_and_check_if_necessary_or_complain();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .set_track_automation_mode(self.raw_unchecked(), mode);
        }
    }

    pub fn automation_mode(&self) -> AutomationMode {
        if self.load_and_check_if_necessary_or_err().is_err() {
            return AutomationMode::Read;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_track_automation_mode(self.raw_internal())
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
        if self.load_and_check_if_necessary_or_err().is_err() {
            return false;
        }
        self.is_master_track_internal()
    }

    fn is_master_track_internal(&self) -> bool {
        let t = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_track_number(self.raw_unchecked())
        };
        t == Some(TrackLocation::MasterTrack)
    }

    pub fn project(&self) -> Project {
        if self.rea_project.get().is_none() {
            unsafe {
                self.load_if_necessary_or_complain_unchecked();
            }
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

fn get_show_attribute_key(track_area: TrackArea) -> TrackAttributeKey<'static> {
    use TrackArea::*;
    match track_area {
        Tcp => TrackAttributeKey::ShowInTcp,
        Mcp => TrackAttributeKey::ShowInMixer,
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GroupingBehavior {
    PreventGrouping,
    UseGrouping,
}

pub struct TrackSetSmartOpts {
    pub gang_behavior: GangBehavior,
    pub grouping_behavior: GroupingBehavior,
    /// This affects undo history and touch-mode automation writing.
    ///
    /// - When `true`, an undo point will be created for this change.
    /// - When `true`, existing automation data will not be overwritten anymore.
    pub done: bool,
}

impl TrackSetSmartOpts {
    fn progress(&self) -> Progress {
        if self.done {
            Progress::Done
        } else {
            Progress::NotDone
        }
    }
}

impl Default for TrackSetSmartOpts {
    fn default() -> Self {
        Self {
            gang_behavior: GangBehavior::DenyGang,
            grouping_behavior: GroupingBehavior::PreventGrouping,
            // Important to be false! We don't want to create undo points by default!
            done: false,
        }
    }
}

fn build_track_ui_flags(
    gang_behavior: GangBehavior,
    grouping_behavior: GroupingBehavior,
) -> BitFlags<SetTrackUiFlags> {
    let mut flags = BitFlags::empty();
    if gang_behavior == GangBehavior::DenyGang {
        flags |= SetTrackUiFlags::PreventSelectionGanging;
    }
    if grouping_behavior == GroupingBehavior::PreventGrouping {
        flags |= SetTrackUiFlags::PreventTrackGrouping;
    }
    flags
}
