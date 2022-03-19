use crate::{Pan, Reaper, Track, Volume};

use reaper_medium::{
    AutomationMode, EditMode, MediaTrack, ReaperFunctionError, ReaperString, TrackSendAttributeKey,
    TrackSendCategory, TrackSendDirection, TrackSendRef, VolumeAndPan,
};
use std::fmt;
use TrackSendDirection::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackRoute {
    direction: TrackSendDirection,
    track: Track,
    /// For send direction, the first indices are hardware outputs if there are any!
    index: u32,
}

impl TrackRoute {
    pub fn new(track: Track, direction: TrackSendDirection, index: u32) -> TrackRoute {
        TrackRoute {
            direction,
            track,
            index,
        }
    }

    pub fn direction(&self) -> TrackSendDirection {
        self.direction
    }

    pub fn is_available(&self) -> bool {
        self.index_is_in_range()
    }

    pub fn track(&self) -> &Track {
        &self.track
    }

    pub fn partner(&self) -> Option<TrackRoutePartner> {
        let partner = match self.direction {
            Receive => {
                let partner_track = get_partner_track(&self.track, self.direction, self.index)?;
                TrackRoutePartner::Track(partner_track)
            }
            Send => {
                let hw_output_count = self.track.typed_send_count(SendPartnerType::HardwareOutput);
                if self.index < hw_output_count {
                    TrackRoutePartner::HardwareOutput(self.index)
                } else {
                    let track_send_index = self.index - hw_output_count;
                    let partner_track =
                        get_partner_track(&self.track, self.direction, track_send_index)?;
                    TrackRoutePartner::Track(partner_track)
                }
            }
        };
        Some(partner)
    }

    /// If this is a send, it counts both hardware output sends and track sends!
    pub fn index(&self) -> u32 {
        self.index
    }

    /// This index only counts track routes. Returns None if it's a hardware output send.
    pub fn track_route_index(&self) -> Option<u32> {
        match self.direction {
            Receive => Some(self.index),
            Send => {
                let hw_output_count = self.track.typed_send_count(SendPartnerType::HardwareOutput);
                if self.index < hw_output_count {
                    None
                } else {
                    let track_send_index = self.index - hw_output_count;
                    Some(track_send_index)
                }
            }
        }
    }

    pub fn volume(&self) -> Result<Volume, ReaperFunctionError> {
        Ok(Volume::from_reaper_value(self.vol_pan()?.volume))
    }

    fn vol_pan(&self) -> Result<VolumeAndPan, ReaperFunctionError> {
        // It's important that we don't use GetTrackSendInfo_Value with D_VOL because it returns the
        // wrong value if an envelope is written.
        match self.direction {
            Send => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_send_ui_vol_pan(self.track().raw(), self.index)
            },
            Receive => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_receive_ui_vol_pan(self.track().raw(), self.index)
            },
        }
    }

    pub fn set_volume(&self, volume: Volume) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper().set_track_send_ui_vol(
                self.track().raw(),
                self.send_ref(),
                volume.reaper_value(),
                EditMode::NormalTweak,
            )
        }
    }

    fn category_with_index(&self) -> (TrackSendCategory, u32) {
        match self.direction {
            Receive => (TrackSendCategory::Receive, self.index),
            Send => {
                let hw_output_count = self.track.typed_send_count(SendPartnerType::HardwareOutput);
                if self.index < hw_output_count {
                    (TrackSendCategory::HardwareOutput, self.index)
                } else {
                    (TrackSendCategory::Send, self.index - hw_output_count)
                }
            }
        }
    }

    fn send_ref(&self) -> TrackSendRef {
        match self.direction {
            Send => TrackSendRef::Send(self.index),
            Receive => TrackSendRef::Receive(self.index),
        }
    }

    pub fn name(&self) -> ReaperString {
        const BUFFER_SIZE: u32 = 256;
        match self.direction {
            Send => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_send_name(self.track().raw(), self.index, BUFFER_SIZE)
                    .expect("send doesn't exist")
            },
            Receive => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_receive_name(self.track().raw(), self.index, BUFFER_SIZE)
                    .expect("receive doesn't exist")
            },
        }
    }

    pub fn pan(&self) -> Result<Pan, ReaperFunctionError> {
        Ok(Pan::from_reaper_value(self.vol_pan()?.pan))
    }

    pub fn set_pan(&self, pan: Pan) -> Result<(), ReaperFunctionError> {
        unsafe {
            Reaper::get().medium_reaper().set_track_send_ui_pan(
                self.track().raw(),
                self.send_ref(),
                pan.reaper_value(),
                EditMode::NormalTweak,
            )
        }
    }

    pub fn is_muted(&self) -> bool {
        let res = match self.direction {
            Send => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_send_ui_mute(self.track().raw(), self.index())
            },
            Receive => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .get_track_receive_ui_mute(self.track().raw(), self.index())
            },
        };
        res.expect("couldn't get send mute")
    }

    pub fn mute(&self) {
        self.set_muted(true);
    }

    pub fn unmute(&self) {
        self.set_muted(false);
    }

    fn set_muted(&self, muted: bool) {
        if self.is_muted() != muted {
            unsafe {
                let _ = Reaper::get()
                    .medium_reaper
                    .toggle_track_send_ui_mute(self.track().raw(), self.send_ref());
            }
        }
    }

    pub fn is_mono(&self) -> bool {
        self.prop_is_enabled(TrackSendAttributeKey::Mono)
    }

    pub fn set_mono(&self, mono: bool) {
        self.set_prop_enabled(TrackSendAttributeKey::Mono, mono);
    }

    pub fn phase_is_inverted(&self) -> bool {
        self.prop_is_enabled(TrackSendAttributeKey::Phase)
    }

    pub fn set_phase_inverted(&self, inverted: bool) {
        self.set_prop_enabled(TrackSendAttributeKey::Phase, inverted);
    }

    pub fn set_automation_mode(&self, mode: AutomationMode) {
        self.set_prop_numeric_value(TrackSendAttributeKey::AutoMode, mode.to_raw() as _);
    }

    pub fn automation_mode(&self) -> AutomationMode {
        let raw_mode = self.prop_numeric_value(TrackSendAttributeKey::AutoMode) as i32;
        AutomationMode::from_raw(raw_mode)
    }

    fn set_prop_enabled(&self, key: TrackSendAttributeKey, enabled: bool) {
        self.set_prop_numeric_value(key, if enabled { 1.0 } else { 0.0 });
    }

    fn prop_is_enabled(&self, key: TrackSendAttributeKey) -> bool {
        self.prop_numeric_value(key) > 0.0
    }

    fn set_prop_numeric_value(&self, key: TrackSendAttributeKey, value: f64) {
        let (category, index) = self.category_with_index();
        unsafe {
            let _ = Reaper::get().medium_reaper().set_track_send_info_value(
                self.track().raw(),
                category,
                index,
                key,
                value,
            );
        }
    }

    fn prop_numeric_value(&self, key: TrackSendAttributeKey) -> f64 {
        let (category, index) = self.category_with_index();
        unsafe {
            Reaper::get().medium_reaper().get_track_send_info_value(
                self.track().raw(),
                category,
                index,
                key,
            )
        }
    }

    pub fn delete(&self) -> Result<(), ReaperFunctionError> {
        let (category, index) = self.category_with_index();
        unsafe {
            Reaper::get()
                .medium_reaper()
                .remove_track_send(self.track().raw(), category, index)
        }
    }

    fn index_is_in_range(&self) -> bool {
        if !self.track.is_available() {
            return false;
        }
        let count = match self.direction {
            Receive => self.track.receive_count(),
            Send => self.track.send_count(),
        };
        self.index < count
    }
}

impl fmt::Display for TrackRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().to_str())
    }
}

pub(super) fn get_partner_track(
    track: &Track,
    direction: TrackSendDirection,
    index: u32,
) -> Option<Track> {
    let raw = get_partner_track_raw(track, direction, index)?;
    let track = Track::new(raw, Some(track.project().raw()));
    Some(track)
}

fn get_partner_track_raw(
    track: &Track,
    direction: TrackSendDirection,
    index: u32,
) -> Option<MediaTrack> {
    let res = match direction {
        Receive => unsafe {
            Reaper::get().medium_reaper().get_track_send_info_srctrack(
                track.raw(),
                TrackSendDirection::Receive,
                index,
            )
        },
        Send => unsafe {
            Reaper::get().medium_reaper().get_track_send_info_desttrack(
                track.raw(),
                TrackSendDirection::Send,
                index,
            )
        },
    };
    res.ok()
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TrackRoutePartner {
    Track(Track),
    HardwareOutput(u32),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SendPartnerType {
    Track,
    HardwareOutput,
}

impl SendPartnerType {
    pub fn to_category(self) -> TrackSendCategory {
        use SendPartnerType::*;
        match self {
            Track => TrackSendCategory::Send,
            HardwareOutput => TrackSendCategory::HardwareOutput,
        }
    }
}
