use crate::high_level::{Track, LightTrack, Reaper};
use std::cell::Cell;
use crate::low_level::MediaTrack;
use std::ptr::null_mut;
use c_str_macro::c_str;

/// The difference to TrackSend is that this implements Copy (not just Clone). See LightTrack for explanation.
#[derive(Clone, Copy, Debug)]
pub struct LightTrackSend {
    source_track: LightTrack,
    target_track: Option<LightTrack>,
    index: Option<u32>,
}

impl From<LightTrackSend> for TrackSend {
    fn from(light: LightTrackSend) -> Self {
        TrackSend {
            source_track: light.source_track.into(),
            target_track: light.target_track.map(|t| t.into()),
            index: Cell::new(light.index),
        }
    }
}

impl From<TrackSend> for LightTrackSend {
    fn from(heavy: TrackSend) -> Self {
        LightTrackSend {
            source_track: heavy.source_track.into(),
            target_track: heavy.target_track.map(|t| t.into()),
            index: heavy.index.get(),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackSend {
    source_track: Track,
    target_track: Option<Track>,
    index: Cell<Option<u32>>,
}

impl TrackSend {
    // Use this if you want to create an index-based send.
    pub fn index_based(source_track: Track, index: u32) -> TrackSend {
        TrackSend {
            source_track,
            target_track: None,
            index: Cell::new(Some(index)),
        }
    }

    // Use this if you want to create a target-track based send (more stable but sometimes not desired -
    // just think of presets that should work in other projects as well).
    // If you know the index, provide it as well!
    pub fn target_based(source_track: Track, target_track: Track, index: Option<u32>) -> TrackSend {
        TrackSend {
            source_track,
            target_track: Some(target_track),
            index: Cell::new(index),
        }
    }

    pub fn is_available(&self) -> bool {
        if self.is_index_based() {
            self.index_is_in_range()
        } else {
            if self.is_loaded_and_at_correct_index() {
                true
            } else {
                // Not yet loaded or at wrong index
                self.load_by_target_track()
            }
        }
    }

    pub fn get_target_track(&self) -> Track {
        if self.is_index_based() {
            get_target_track(&self.source_track, self.get_index())
        } else {
            self.target_track.clone().expect("No target track set")
        }
    }

    pub fn get_index(&self) -> u32 {
        self.check_or_load_if_necessary_or_complain();
        self.index.get().expect("Index not set")
    }

    fn load_by_target_track(&self) -> bool {
        let target_track = match &self.target_track {
            None => return false,
            Some(t) => t
        };
        if !self.source_track.is_available() {
            return false;
        }
        match self.source_track.get_sends()
            .find(|s| s.get_target_track() == *target_track) {
            None => false,
            Some(found_track_send) => {
                self.index.replace(Some(found_track_send.get_index()));
                true
            }
        }
    }

    // Precondition: is target track based
    fn is_loaded_and_at_correct_index(&self) -> bool {
        if self.index.get().is_some() {
            self.is_at_correct_index()
        } else {
            // Not loaded
            false
        }
    }

    // Precondition: is target track based
    fn is_at_correct_index(&self) -> bool {
        self.source_track.is_available() && self.get_target_track_by_index() == self.target_track
    }

    // Precondition: index set
    fn get_target_track_by_index(&self) -> Option<Track> {
        let target_media_track = get_target_media_track(&self.source_track, self.index.get().expect("Index not set"));
        if target_media_track.is_null() {
            return None;
        }
        Some(Track::new(target_media_track, self.source_track.get_project().get_rea_project()))
    }

    fn is_index_based(&self) -> bool {
        self.target_track.is_none()
    }

    fn index_is_in_range(&self) -> bool {
        self.source_track.is_available()
            && self.index.get().expect("No index") < self.source_track.get_send_count()
    }

    fn check_or_load_if_necessary_or_complain(&self) {
        if self.is_index_based() {
            if !self.index_is_in_range() {
                panic!("Index based send not loadable")
            }
        } else {
            // Target track based
            if !self.is_loaded_and_at_correct_index() && !self.load_by_target_track() {
                panic!("Target track based send not loadable")
            }
        }
    }
}

pub(super) fn get_target_track(source_track: &Track, send_index: u32) -> Track {
    Track::new(get_target_media_track(source_track, send_index), source_track.get_project().get_rea_project())
}

fn get_target_media_track(source_track: &Track, send_index: u32) -> *mut MediaTrack {
    Reaper::instance().medium.get_set_track_send_info(
        source_track.get_media_track(),
        0,
        send_index,
        c_str!("P_DESTTRACK"),
        null_mut(),
    ) as *mut MediaTrack
}