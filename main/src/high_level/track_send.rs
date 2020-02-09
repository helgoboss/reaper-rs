use crate::high_level::{Track, LightTrack};
use std::cell::Cell;

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
}