use crate::access::{Mut, ReadAccess, WriteAccess};
use crate::{Fx, Reaper, Track, TrackDesc};
use reaper_medium::{AddFxBehavior, ReaperFunctionError, ReaperStringArg, TrackFxChainType};
use std::iter::FusedIterator;
use std::marker::PhantomData;

// TODO-high Monitoring context
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FxChainDesc {
    track_desc: TrackDesc,
    kind: TrackFxChainType,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FxChain<'a, A> {
    track: Track<'a, ReadAccess>,
    kind: TrackFxChainType,
    _p: PhantomData<A>,
}

impl FxChainDesc {
    pub fn new(track_desc: TrackDesc, kind: TrackFxChainType) -> Self {
        Self { track_desc, kind }
    }

    // pub fn resolve(&self) -> Option<FxChain> {
    //     let fx_chain = FxChain {
    //         track: self.track_desc.ptr()?,
    //         kind: self.kind,
    //     };
    //     Some(fx_chain)
    // }
}

impl<'a, A> FxChain<'a, A> {
    pub(crate) fn new(track: Track<'a, ReadAccess>, kind: TrackFxChainType) -> Self {
        Self {
            track,
            kind,
            _p: PhantomData,
        }
    }

    pub fn track(&self) -> Track<ReadAccess> {
        self.track
    }

    pub fn kind(&self) -> TrackFxChainType {
        self.kind
    }

    pub fn add_fx_by_name<'b>(
        &mut self,
        name: impl Into<ReaperStringArg<'b>>,
        behavior: AddFxBehavior,
    ) -> Result<Fx<WriteAccess>, ReaperFunctionError>
    where
        A: Mut,
    {
        let r = Reaper::get().medium_reaper();
        let index =
            unsafe { r.track_fx_add_by_name_add(self.track.raw(), name, self.kind, behavior)? };
        Ok(Fx::new(FxChain::new(self.track, self.kind), index))
    }

    pub fn fxs(
        &self,
    ) -> impl Iterator<Item = Fx<ReadAccess>> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        (0..self.fx_count()).map(|i| Fx::new(FxChain::new(self.track, self.kind), i))
    }

    pub fn fx_count(&self) -> u32 {
        let r = Reaper::get().medium_reaper();
        match self.kind {
            TrackFxChainType::NormalFxChain => unsafe { r.track_fx_get_count(self.track.raw()) },
            TrackFxChainType::InputFxChain => unsafe { r.track_fx_get_rec_count(self.track.raw()) },
        }
    }

    pub fn desc(&self) -> FxChainDesc {
        FxChainDesc::new(self.track.desc(), self.kind)
    }
}
