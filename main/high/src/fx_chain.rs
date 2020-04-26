use crate::fx::{get_fx_guid, Fx};
use crate::guid::Guid;
use crate::{get_fx_query_index, Chunk, ChunkRegion, Reaper, Track, MAX_TRACK_CHUNK_SIZE};

use reaper_rs_medium::{TrackFxChainType, TransferBehavior, UndoHint};
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

    pub fn get_fx_count(&self) -> u32 {
        let reaper = Reaper::get();
        if self.is_input_fx {
            unsafe { reaper.medium.track_fx_get_rec_count(self.track.get_raw()) as u32 }
        } else {
            unsafe { reaper.medium.track_fx_get_count(self.track.get_raw()) as u32 }
        }
    }

    // Moves within this FX chain
    pub fn move_fx(&self, fx: &Fx, new_index: u32) {
        assert_eq!(fx.get_chain(), *self);
        unsafe {
            Reaper::get().medium.track_fx_copy_to_track(
                self.track.get_raw(),
                fx.get_query_index(),
                self.track.get_raw(),
                get_fx_query_index(new_index, self.is_input_fx),
                TransferBehavior::Move,
            );
        }
    }

    pub fn remove_fx(&self, fx: &Fx) {
        assert_eq!(fx.get_chain(), *self);
        if !fx.is_available() {
            return;
        }
        let reaper = Reaper::get();
        if reaper.medium.low.pointers.TrackFX_Delete.is_some() {
            unsafe {
                reaper
                    .medium
                    .track_fx_delete(self.track.get_raw(), fx.get_query_index())
            };
        } else {
            let new_chunk = {
                let fx_chunk_region = fx.get_chunk();
                fx_chunk_region
                    .get_parent_chunk()
                    .delete_region(&fx_chunk_region);
                fx_chunk_region.get_parent_chunk()
            };
            self.track.set_chunk(new_chunk);
        }
    }

    pub fn add_fx_from_chunk(&self, chunk: &str) -> Option<Fx> {
        let mut track_chunk = self.track.get_chunk(MAX_TRACK_CHUNK_SIZE, UndoHint::Normal);
        let chain_tag = self.find_chunk_region(track_chunk.clone());
        match chain_tag {
            Some(tag) => {
                // There's an FX chain already. Add after last FX.
                track_chunk.insert_before_region_as_block(&tag.get_last_line(), chunk);
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
                    &track_chunk.get_region().get_first_line(),
                    chain_chunk_string.as_str(),
                );
            }
        }
        self.track.set_chunk(track_chunk);
        return self.get_last_fx();
    }

    // Returned FX has GUIDs set
    pub fn get_fxs(&self) -> impl Iterator<Item = Fx> + '_ {
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

    // In Track this returns Chunk, here it returns ChunkRegion. Because REAPER always returns
    // the chunk of the complete track, not just of the FX chain.
    pub fn get_chunk(&self) -> Option<ChunkRegion> {
        self.find_chunk_region(self.track.get_chunk(MAX_TRACK_CHUNK_SIZE, UndoHint::Normal))
    }

    pub fn set_chunk(&self, chunk: &str) {
        let mut track_chunk = self.track.get_chunk(MAX_TRACK_CHUNK_SIZE, UndoHint::Normal);
        let chain_tag = self.find_chunk_region(track_chunk.clone());
        match chain_tag {
            Some(r) => {
                // There's an FX chain already. Replace it.
                track_chunk.replace_region(&r, chunk);
            }
            None => {
                // There's no FX chain yet. Insert it.
                track_chunk.insert_after_region_as_block(
                    &track_chunk.get_region().get_first_line(),
                    chunk,
                );
            }
        }
        self.track.set_chunk(track_chunk);
    }

    fn find_chunk_region(&self, track_chunk: Chunk) -> Option<ChunkRegion> {
        track_chunk
            .get_region()
            .find_first_tag_named(0, self.get_chunk_tag_name())
    }

    fn get_chunk_tag_name(&self) -> &'static str {
        if self.is_input_fx {
            "FXCHAIN_REC"
        } else {
            "FXCHAIN"
        }
    }

    pub fn get_first_instrument_fx(&self) -> Option<Fx> {
        if self.is_input_fx {
            return None;
        }
        unsafe {
            Reaper::get()
                .medium
                .track_fx_get_instrument(self.track.get_raw())
        }
        .and_then(|fx_index| self.get_fx_by_index(fx_index))
    }

    pub fn add_fx_by_original_name(&self, original_fx_name: &CStr) -> Option<Fx> {
        let fx_index = unsafe {
            Reaper::get().medium.track_fx_add_by_name_add(
                self.track.get_raw(),
                original_fx_name,
                if self.is_input_fx {
                    TrackFxChainType::InputFxChain
                } else {
                    TrackFxChainType::NormalFxChain
                },
                true,
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

    pub fn get_track(&self) -> Track {
        self.track.clone()
    }

    pub fn is_input_fx(&self) -> bool {
        self.is_input_fx
    }

    pub fn get_first_fx_by_name(&self, name: &CStr) -> Option<Fx> {
        let fx_index = unsafe {
            Reaper::get().medium.track_fx_add_by_name_query(
                self.track.get_raw(),
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
