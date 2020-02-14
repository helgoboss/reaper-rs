use crate::high_level::{Track, Reaper, get_media_track_guid, ChunkRegion, MAX_TRACK_CHUNK_SIZE, Chunk};
use crate::high_level::fx::{Fx, get_fx_guid};
use crate::high_level::guid::Guid;
use std::ffi::CStr;
use c_str_macro::c_str;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxChain {
    track: Track,
    is_input_fx: bool,
}

impl FxChain {
    pub(super) fn new(track: Track, is_input_fx: bool) -> FxChain {
        FxChain {
            track,
            is_input_fx,
        }
    }

    pub fn get_fx_count(&self) -> u32 {
        let reaper = Reaper::instance();
        if self.is_input_fx {
            reaper.medium.track_fx_get_rec_count(self.track.get_media_track()) as u32
        } else {
            reaper.medium.track_fx_get_count(self.track.get_media_track()) as u32
        }
    }

    // Returned FX has GUIDs set
    pub fn get_fxs(&self) -> impl Iterator<Item=Fx> + '_ {
        (0..self.get_fx_count()).map(move |i| {
            Fx::from_guid_and_index(
                self.track.clone(),
                get_fx_guid(&self.track, i, self.is_input_fx).expect("Couldn't determine FX GUID"),
                i,
                self.is_input_fx,
            )
        })
    }

    // This returns a non-optional in order to support not-yet-loaded FX. GUID is a perfectly stable
    // identifier of an FX!
    pub fn get_fx_by_guid(&self, guid: &Guid) -> Fx {
        Fx::from_guid_lazy_index(self.track.clone(), *guid, self.is_input_fx)
    }

    // Like fxByGuid but if you already know the index
    pub fn get_fx_by_guid_and_index(&self, guid: &Guid, index: u32) -> Fx {
        Fx::from_guid_and_index(self.track.clone(), *guid, index, self.is_input_fx)
    }

    // TODO In Track this returns Chunk, here it returns ChunkRegion
    pub fn get_chunk(&self) -> Option<ChunkRegion> {
        self.find_chunk_region(self.track.get_chunk(MAX_TRACK_CHUNK_SIZE, false))
    }

    fn find_chunk_region(&self, track_chunk: Chunk) -> Option<ChunkRegion> {
        track_chunk.get_region().find_first_tag_named(0, self.get_chunk_tag_name())
    }

    fn get_chunk_tag_name(&self) -> &'static str {
        if self.is_input_fx {
            "FXCHAIN_REC"
        } else {
            "FXCHAIN"
        }
    }

    pub fn add_fx_by_original_name(&self, original_fx_name: &CStr) -> Option<Fx> {
        let fx_index = Reaper::instance().medium.track_fx_add_by_name(
            self.track.get_media_track(), original_fx_name, self.is_input_fx, -1);
        if fx_index == -1 {
            return None;
        }
        Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, fx_index as u32, self.is_input_fx).expect("Couldn't get GUID"),
            fx_index as u32,
            self.is_input_fx
        ).into()
    }

    pub fn get_first_fx_by_name(&self, name: &CStr) -> Option<Fx> {
        let fx_index = Reaper::instance().medium.track_fx_add_by_name(
            self.track.get_media_track(), name, self.is_input_fx, 0);
        if fx_index == -1 {
            return None;
        }
        Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, fx_index as u32, self.is_input_fx).expect("Couldn't get GUID"),
            fx_index as u32,
            self.is_input_fx,
        ).into()
    }

    // It's correct that this returns an optional because the index isn't a stable identifier of an FX.
    // The FX could move. So this should do a runtime lookup of the FX and return a stable GUID-backed Fx object if
    // an FX exists at that index.
    pub fn get_fx_by_index(&self, index: u32) -> Option<Fx> {
        if index >= self.get_fx_count() {
            return None;
        }
        Some(Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, index, self.is_input_fx).expect("Couldn't determine FX GUID"),
            index,
            self.is_input_fx,
        ))
    }

    pub fn get_first_fx(&self) -> Option<Fx> {
        self.get_fx_by_index(0)
    }

    pub fn get_last_fx(&self) -> Option<Fx> {
        let fx_count = self.get_fx_count();
        if fx_count == 0 {
            return None;
        }
        self.get_fx_by_index(fx_count - 1)
    }

    pub fn is_available(&self) -> bool {
        self.track.is_available()
    }
}
