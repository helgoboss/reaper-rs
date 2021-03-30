use crate::fx::{get_fx_guid, Fx};
use crate::guid::Guid;
use crate::{
    get_track_fx_location, Chunk, ChunkRegion, Project, Reaper, Take, Track, MAX_TRACK_CHUNK_SIZE,
};

use reaper_medium::{
    AddFxBehavior, ChunkCacheHint, FxChainVisibility, FxShowInstruction, ReaperStringArg,
    TrackFxChainType, TransferBehavior,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FxChainContext {
    // TODO-medium Deal with the following PartialEq issue.
    //  The combination "Master track + input FX chain" by convention represents the
    //  monitoring FX chain in REAPER. It's a bit unfortunate that we have 2 representations
    //  of the same thing: A special monitoring FX enum variant and this convention.
    //  E.g. it leads to the result that both representations are not equal from a reaper-rs
    //  perspective. We should enforce the enum variant whenever possible because the
    //  convention is somehow flawed. E.g. what if we have 2 master tracks of different
    //  projects? 2 FX chains won't equal if they both are master tracks and is_input_fx = true
    //  but master tracks from different projects!
    Monitoring,
    Track { track: Track, is_input_fx: bool },
    Take(Take),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FxChain {
    context: FxChainContext,
}

impl FxChain {
    pub(crate) fn from_track(track: Track, is_input_fx: bool) -> FxChain {
        FxChain {
            context: FxChainContext::Track { track, is_input_fx },
        }
    }

    pub(crate) fn from_take(take: Take) -> FxChain {
        FxChain {
            context: FxChainContext::Take(take),
        }
    }

    pub(crate) fn from_monitoring() -> FxChain {
        FxChain {
            context: FxChainContext::Monitoring,
        }
    }

    pub fn context(&self) -> &FxChainContext {
        &self.context
    }

    pub fn project(&self) -> Option<Project> {
        self.track().map(|t| t.project())
    }

    pub fn fx_count(&self) -> u32 {
        let reaper = Reaper::get().medium_reaper();
        match &self.context {
            FxChainContext::Track { track, is_input_fx } => {
                if *is_input_fx {
                    unsafe { reaper.track_fx_get_rec_count(track.raw()) }
                } else {
                    unsafe { reaper.track_fx_get_count(track.raw()) }
                }
            }
            FxChainContext::Monitoring => {
                let track = Reaper::get().current_project().master_track();
                unsafe { reaper.track_fx_get_rec_count(track.raw()) }
            }
            FxChainContext::Take(_) => todo!(),
        }
    }

    pub fn visibility(&self) -> FxChainVisibility {
        let reaper = Reaper::get().medium_reaper();
        match &self.context {
            FxChainContext::Track { track, is_input_fx } => {
                if *is_input_fx {
                    unsafe { reaper.track_fx_get_rec_chain_visible(track.raw()) }
                } else {
                    unsafe { reaper.track_fx_get_chain_visible(track.raw()) }
                }
            }
            FxChainContext::Monitoring => {
                let track = Reaper::get().current_project().master_track();
                unsafe { reaper.track_fx_get_rec_chain_visible(track.raw()) }
            }
            FxChainContext::Take(_) => todo!(),
        }
    }

    pub fn hide(&self) {
        match self.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let track = self.track_or_master_track();
                let instruction = FxShowInstruction::HideChain(if self.is_input_fx() {
                    TrackFxChainType::InputFxChain
                } else {
                    TrackFxChainType::NormalFxChain
                });
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_show(track.raw(), instruction);
                }
            }
        }
    }

    // Moves within this FX chain
    pub fn move_fx(&self, fx: &Fx, new_index: u32) -> Result<(), &'static str> {
        assert_eq!(fx.chain(), self);
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().TrackFX_CopyToTrack.is_some() {
            match self.context() {
                FxChainContext::Take(_) => todo!(),
                _ => {
                    let (track, location) = fx.track_and_location();
                    unsafe {
                        reaper.track_fx_copy_to_track(
                            (track.raw(), location),
                            (
                                track.raw(),
                                get_track_fx_location(new_index, self.is_input_fx()),
                            ),
                            TransferBehavior::Move,
                        );
                    }
                }
            };
        } else {
            if !fx.is_available() {
                return Err("FX not available");
            }
            if fx.index() == new_index {
                return Ok(());
            }
            let new_chunk = {
                let actual_new_index = new_index.min(self.fx_count() - 1);
                let original_fx_chunk_region = fx.chunk()?;
                let current_fx_at_new_index_chunk_region =
                    self.fx_by_index(actual_new_index).unwrap().chunk()?;
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
            self.track_fx_track()
                .ok_or("working on track FX only")?
                .set_chunk(new_chunk)?
        };
        Ok(())
    }

    fn track_fx_track(&self) -> Option<&Track> {
        match self.context() {
            FxChainContext::Track { track, .. } => Some(track),
            _ => None,
        }
    }

    pub fn remove_fx(&self, fx: &Fx) -> Result<(), &'static str> {
        assert_eq!(fx.chain(), self);
        if !fx.is_available() {
            return Err("FX not available");
        }
        let reaper = Reaper::get().medium_reaper();
        if reaper.low().pointers().TrackFX_Delete.is_some() {
            match self.context() {
                FxChainContext::Take(_) => todo!(),
                _ => {
                    let (track, location) = fx.track_and_location();
                    unsafe {
                        reaper
                            .track_fx_delete(track.raw(), location)
                            .map_err(|_| "couldn't delete track FX")?
                    };
                }
            };
        } else {
            let new_chunk = {
                let fx_chunk_region = fx.chunk()?;
                fx_chunk_region
                    .parent_chunk()
                    .delete_region(&fx_chunk_region);
                fx_chunk_region.parent_chunk()
            };
            self.track_fx_track()
                .ok_or("working on track FX only")?
                .set_chunk(new_chunk)?;
        }
        Ok(())
    }

    pub fn add_fx_from_chunk(&self, chunk: &str) -> Result<Fx, &'static str> {
        let mut track_chunk = self
            .track_fx_track()
            .ok_or("working on track FX only")?
            .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode)?;
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
        self.track_fx_track()
            .ok_or("working on track FX only")?
            .set_chunk(track_chunk)?;
        self.last_fx().ok_or("FX not added")
    }

    // Returned FX has GUIDs set
    pub fn fxs(&self) -> impl Iterator<Item = Fx> + ExactSizeIterator + '_ {
        (0..self.fx_count()).map(move |i| {
            Fx::from_guid_and_index(
                self.clone(),
                get_fx_guid(self, i).expect("Couldn't determine FX GUID"),
                i,
            )
        })
    }

    // Returned FX are light-weight and don't have GUID set.
    pub fn index_based_fxs(&self) -> impl Iterator<Item = Fx> + ExactSizeIterator + '_ {
        (0..self.fx_count()).map(move |i| Fx::from_index_untracked(self.clone(), i))
    }

    // This returns a non-optional in order to support not-yet-loaded FX. GUID is a perfectly stable
    // identifier of an FX!
    pub fn fx_by_guid(&self, guid: &Guid) -> Fx {
        Fx::from_guid_lazy_index(self.clone(), *guid)
    }

    // Like fxByGuid but if you already know the index
    pub fn fx_by_guid_and_index(&self, guid: &Guid, index: u32) -> Fx {
        Fx::from_guid_and_index(self.clone(), *guid, index)
    }

    // In Track this returns Chunk, here it returns ChunkRegion. Because REAPER always returns
    // the chunk of the complete track, not just of the FX chain.
    pub fn chunk(&self) -> Result<Option<ChunkRegion>, &'static str> {
        let res = self.find_chunk_region(
            self.track_fx_track()
                .ok_or("working on track FX only")?
                .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode)?,
        );
        Ok(res)
    }

    pub fn set_chunk(&self, chunk: &str) -> Result<(), &'static str> {
        let mut track_chunk = self
            .track_fx_track()
            .ok_or("works on track FX only")?
            .chunk(MAX_TRACK_CHUNK_SIZE, ChunkCacheHint::NormalMode)?;
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
        self.track_fx_track()
            .ok_or("works on track FX only")?
            .set_chunk(track_chunk)?;
        Ok(())
    }

    fn find_chunk_region(&self, track_chunk: Chunk) -> Option<ChunkRegion> {
        track_chunk
            .region()
            .find_first_tag_named(0, self.chunk_tag_name())
    }

    fn chunk_tag_name(&self) -> &'static str {
        if self.is_input_fx() {
            "FXCHAIN_REC"
        } else {
            "FXCHAIN"
        }
    }

    pub fn first_instrument_fx(&self) -> Option<Fx> {
        match self.context() {
            FxChainContext::Take(_) => todo!(),
            FxChainContext::Monitoring => None,
            FxChainContext::Track { track, is_input_fx } => {
                if *is_input_fx {
                    return None;
                }
                let fx_index = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_instrument(track.raw())
                };
                fx_index.and_then(|i| self.fx_by_index(i))
            }
        }
    }

    pub fn hide_all_floating_windows(&self) {
        for fx in self.fxs() {
            fx.hide_floating_window();
        }
    }

    pub fn add_fx_by_original_name<'a>(
        &self,
        original_fx_name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<Fx> {
        let fx_index = match self.context() {
            FxChainContext::Take(_) => todo!(),
            _ => unsafe {
                Reaper::get()
                    .medium_reaper()
                    .track_fx_add_by_name_add(
                        self.track_or_master_track().raw(),
                        original_fx_name,
                        if self.is_input_fx() {
                            TrackFxChainType::InputFxChain
                        } else {
                            TrackFxChainType::NormalFxChain
                        },
                        AddFxBehavior::AlwaysAdd,
                    )
                    .ok()?
            },
        };
        Some(Fx::from_guid_and_index(
            self.clone(),
            get_fx_guid(self, fx_index).expect("Couldn't get GUID"),
            fx_index,
        ))
    }

    /// For internal use only.
    ///
    /// We don't want to expose that monitoring FX is reachable via master track of current project
    /// - although it has nothing to do with the current project.
    fn track_or_master_track(&self) -> Track {
        match self.context() {
            FxChainContext::Monitoring => Reaper::get().current_project().master_track(),
            FxChainContext::Track { track, .. } => track.clone(),
            FxChainContext::Take(take) => take.track().clone(),
        }
    }

    pub fn track(&self) -> Option<&Track> {
        match &self.context {
            FxChainContext::Track { track, .. } => Some(track),
            // TODO-low This is dangerous. Some chunk functions which call track assume this is
            //  a track FX when this returns a track. Clean them up!
            FxChainContext::Take(take) => Some(take.track()),
            FxChainContext::Monitoring => None,
        }
    }

    pub fn is_input_fx(&self) -> bool {
        match &self.context {
            FxChainContext::Track { is_input_fx, .. } => *is_input_fx,
            // In REAPER, monitoring FX chain is usually referred to as input FX of the master
            // track, so it's just consequent to report it as input FX.
            FxChainContext::Monitoring => true,
            _ => false,
        }
    }

    pub fn first_fx_by_name<'a>(&self, name: impl Into<ReaperStringArg<'a>>) -> Option<Fx> {
        let fx_index = match self.context() {
            FxChainContext::Take(_) => todo!(),
            FxChainContext::Track { track, .. } => unsafe {
                Reaper::get().medium_reaper().track_fx_add_by_name_query(
                    track.raw(),
                    name,
                    if self.is_input_fx() {
                        TrackFxChainType::InputFxChain
                    } else {
                        TrackFxChainType::NormalFxChain
                    },
                )?
            },
            FxChainContext::Monitoring => unsafe {
                Reaper::get().medium_reaper().track_fx_add_by_name_query(
                    Reaper::get().current_project().master_track().raw(),
                    name,
                    TrackFxChainType::InputFxChain,
                )?
            },
        };
        Some(Fx::from_guid_and_index(
            self.clone(),
            get_fx_guid(self, fx_index).expect("Couldn't get GUID"),
            fx_index,
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
            self.clone(),
            get_fx_guid(self, index).expect("Couldn't determine FX GUID"),
            index,
        ))
    }

    // This returns a purely index-based FX that doesn't keep track of FX GUID, doesn't follow
    // reorderings and so on.
    pub fn fx_by_index_untracked(&self, index: u32) -> Fx {
        Fx::from_index_untracked(self.clone(), index)
    }

    pub fn index_based_fx_by_index(&self, index: u32) -> Option<Fx> {
        let untracked = Fx::from_index_untracked(self.clone(), index);
        if !untracked.is_available() {
            return None;
        }
        Some(untracked)
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
        match self.context() {
            FxChainContext::Take(_) => todo!(),
            FxChainContext::Monitoring => true,
            FxChainContext::Track { track, .. } => track.is_available(),
        }
    }
}
