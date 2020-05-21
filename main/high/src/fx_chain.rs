use crate::fx::{get_fx_guid, Fx};
use crate::guid::Guid;
use crate::{get_fx_query_index, Chunk, ChunkRegion, Reaper, Track, MAX_TRACK_CHUNK_SIZE};

use reaper_medium::{AddFxBehavior, ChunkCacheHint, TrackFxChainType, TransferBehavior};
use std::ffi::CStr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxChain {
    track: Track,
    is_input_fx: bool,
}

impl FxChain {
    pub(super) fn new(track: Track, is_input_fx: bool) -> FxChain {
        FxChain { track, is_input_fx }
    }

    pub fn fx_count(&self) -> u32 {
        let reaper = Reaper::get().medium_reaper();
        if self.is_input_fx {
            unsafe { reaper.track_fx_get_rec_count(self.track.raw()) as u32 }
        } else {
            unsafe { reaper.track_fx_get_count(self.track.raw()) as u32 }
        }
    }

    // Moves within this FX chain
    pub fn move_fx(&self, fx: &Fx, new_index: u32) {
        assert_eq!(fx.chain(), *self);
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().TrackFX_CopyToTrack.is_some() {
            unsafe {
                reaper.track_fx_copy_to_track(
                    (self.track.raw(), fx.query_index()),
                    (
                        self.track.raw(),
                        get_fx_query_index(new_index, self.is_input_fx),
                    ),
                    TransferBehavior::Move,
                );
            }
        } else {
            if !fx.is_available() || fx.index() == new_index {
                return;
            }
            let new_chunk = {
                let actual_new_index = new_index.min(self.fx_count() - 1);
                let original_fx_chunk_region = fx.chunk();
                let current_fx_at_new_index_chunk_region =
                    self.fx_by_index(actual_new_index).unwrap().chunk();
                let original_content = original_fx_chunk_region.content().to_owned();
                if fx.index() < actual_new_index {
                    // Moves down
                    original_fx_chunk_region
                        .parent_chunk()
                        .insert_after_region_as_block(
                            &current_fx_at_new_index_chunk_region,
                            original_content.as_str(),
                        );
                    original_fx_chunk_region
                        .parent_chunk()
                        .delete_region(&original_fx_chunk_region);
                } else {
                    // Moves up
                    original_fx_chunk_region
                        .parent_chunk()
                        .delete_region(&original_fx_chunk_region);
                    original_fx_chunk_region
                        .parent_chunk()
                        .insert_before_region_as_block(
                            &current_fx_at_new_index_chunk_region,
                            original_content.as_str(),
                        );
                };
                original_fx_chunk_region.parent_chunk()
            };
            self.track.set_chunk(new_chunk)
        }
    }

    pub fn remove_fx(&self, fx: &Fx) {
        assert_eq!(fx.chain(), *self);
        if !fx.is_available() {
            return;
        }
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().TrackFX_Delete.is_some() {
            unsafe {
                reaper
                    .track_fx_delete(self.track.raw(), fx.query_index())
                    .expect("couldn't delete track FX")
            };
        } else {
            let new_chunk = {
                let fx_chunk_region = fx.chunk();
                fx_chunk_region
                    .parent_chunk()
                    .delete_region(&fx_chunk_region);
                fx_chunk_region.parent_chunk()
            };
            self.track.set_chunk(new_chunk);
        }
    }

    pub fn add_fx_from_chunk(&self, chunk: &str) -> Option<Fx> {
        let mut track_chunk = self
            .track
            .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode);
        let chain_tag = self.find_chunk_region(track_chunk.clone());
        match chain_tag {
            Some(tag) => {
                // There's an FX chain already. Add after last FX.
                track_chunk.insert_before_region_as_block(&tag.last_line(), chunk);
            }
            None => {
                // There's no FX chain yet. Insert it with FX.
                let mut chain_chunk_string = String::from(
                    r#"
<FXCHAIN
WNDRECT 0 144 1082 736
SHOW 0
LASTSEL 1
DOCKED 0
"#,
                );
                chain_chunk_string.push_str(chunk);
                chain_chunk_string.push_str("\n>");
                track_chunk.insert_after_region_as_block(
                    &track_chunk.region().first_line(),
                    chain_chunk_string.as_str(),
                );
            }
        }
        self.track.set_chunk(track_chunk);
        self.last_fx()
    }

    // Returned FX has GUIDs set
    pub fn fxs(&self) -> impl Iterator<Item = Fx> + '_ {
        (0..self.fx_count()).map(move |i| {
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
    pub fn fx_by_guid(&self, guid: &Guid) -> Fx {
        Fx::from_guid_lazy_index(self.track.clone(), *guid, self.is_input_fx)
    }

    // Like fxByGuid but if you already know the index
    pub fn fx_by_guid_and_index(&self, guid: &Guid, index: u32) -> Fx {
        Fx::from_guid_and_index(self.track.clone(), *guid, index, self.is_input_fx)
    }

    // In Track this returns Chunk, here it returns ChunkRegion. Because REAPER always returns
    // the chunk of the complete track, not just of the FX chain.
    pub fn chunk(&self) -> Option<ChunkRegion> {
        self.find_chunk_region(
            self.track
                .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode),
        )
    }

    pub fn set_chunk(&self, chunk: &str) {
        let mut track_chunk = self
            .track
            .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode);
        let chain_tag = self.find_chunk_region(track_chunk.clone());
        match chain_tag {
            Some(r) => {
                // There's an FX chain already. Replace it.
                track_chunk.replace_region(&r, chunk);
            }
            None => {
                // There's no FX chain yet. Insert it.
                track_chunk.insert_after_region_as_block(&track_chunk.region().first_line(), chunk);
            }
        }
        self.track.set_chunk(track_chunk);
    }

    fn find_chunk_region(&self, track_chunk: Chunk) -> Option<ChunkRegion> {
        track_chunk
            .region()
            .find_first_tag_named(0, self.chunk_tag_name())
    }

    fn chunk_tag_name(&self) -> &'static str {
        if self.is_input_fx {
            "FXCHAIN_REC"
        } else {
            "FXCHAIN"
        }
    }

    pub fn first_instrument_fx(&self) -> Option<Fx> {
        if self.is_input_fx {
            return None;
        }
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_instrument(self.track.raw())
        }
        .and_then(|fx_index| self.fx_by_index(fx_index))
    }

    pub fn add_fx_by_original_name(&self, original_fx_name: &CStr) -> Option<Fx> {
        let fx_index = unsafe {
            Reaper::get().medium_reaper().track_fx_add_by_name_add(
                self.track.raw(),
                original_fx_name,
                if self.is_input_fx {
                    TrackFxChainType::InputFxChain
                } else {
                    TrackFxChainType::NormalFxChain
                },
                AddFxBehavior::AlwaysAdd,
            )
        }
        .ok()?;
        Some(Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, fx_index, self.is_input_fx).expect("Couldn't get GUID"),
            fx_index,
            self.is_input_fx,
        ))
    }

    pub fn track(&self) -> Track {
        self.track.clone()
    }

    pub fn is_input_fx(&self) -> bool {
        self.is_input_fx
    }

    pub fn first_fx_by_name(&self, name: &CStr) -> Option<Fx> {
        let fx_index = unsafe {
            Reaper::get().medium_reaper().track_fx_add_by_name_query(
                self.track.raw(),
                name,
                if self.is_input_fx {
                    TrackFxChainType::InputFxChain
                } else {
                    TrackFxChainType::NormalFxChain
                },
            )
        }?;
        Some(Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, fx_index, self.is_input_fx).expect("Couldn't get GUID"),
            fx_index,
            self.is_input_fx,
        ))
    }

    // It's correct that this returns an optional because the index isn't a stable identifier of an
    // FX. The FX could move. So this should do a runtime lookup of the FX and return a stable
    // GUID-backed Fx object if an FX exists at that index.
    pub fn fx_by_index(&self, index: u32) -> Option<Fx> {
        if index >= self.fx_count() {
            return None;
        }
        Some(Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, index, self.is_input_fx).expect("Couldn't determine FX GUID"),
            index,
            self.is_input_fx,
        ))
    }

    // This returns a purely index-based FX that doesn't keep track of FX GUID, doesn't follow
    // reorderings and so on.
    pub fn fx_by_index_untracked(&self, index: u32) -> Fx {
        Fx::from_index_untracked(self.track.clone(), index, self.is_input_fx)
    }

    pub fn first_fx(&self) -> Option<Fx> {
        self.fx_by_index(0)
    }

    pub fn last_fx(&self) -> Option<Fx> {
        let fx_count = self.fx_count();
        if fx_count == 0 {
            return None;
        }
        self.fx_by_index(fx_count - 1)
    }

    pub fn is_available(&self) -> bool {
        self.track.is_available()
    }
}
