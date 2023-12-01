use crate::access::ReadAccess;
use crate::{FxChain, FxChainDesc, Reaper};
use reaper_low::raw::GUID;
use reaper_medium::{
    FxShowInstruction, MediaTrack, ReaperString, ReaperStringArg, TrackFxChainType, TrackFxLocation,
};
use std::marker::PhantomData;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FxDesc {
    chain_desc: FxChainDesc,
    id: GUID,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Fx<'a, A> {
    fx_chain: FxChain<'a, ReadAccess>,
    index: u32,
    _p: PhantomData<&'a A>,
}

impl FxDesc {
    pub fn new(chain_desc: FxChainDesc, id: GUID) -> Self {
        Self { chain_desc, id }
    }

    // pub fn resolve(&self) -> Option<Fx> {
    //     let fx_chain = self.chain_desc.resolve()?;
    //     let index = fx_chain.fxs().position(|fx| fx.guid() == self.id)?;
    //     let fx = Fx {
    //         fx_chain,
    //         index: index as u32,
    //     };
    //     Some(fx)
    // }
}

impl<'a, A> Fx<'a, A> {
    pub(crate) fn new(fx_chain: FxChain<'a, ReadAccess>, index: u32) -> Self {
        Self {
            fx_chain,
            index,
            _p: PhantomData,
        }
    }

    pub fn guid(&self) -> GUID {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_fx_guid(self.raw_track(), self.location())
                .expect("must exist")
        }
    }

    pub fn fx_chain(&self) -> FxChain<ReadAccess> {
        self.fx_chain
    }

    pub fn hide_window(&mut self) {
        unsafe {
            Reaper::get().medium_reaper().track_fx_show(
                self.raw_track(),
                FxShowInstruction::HideFloatingWindow(self.location()),
            );
            self.fx_chain.hide_window();
        }
    }

    pub fn get_named_config_param_as_string<'b>(
        &self,
        param_name: impl Into<ReaperStringArg<'b>>,
        buffer_size: u32,
    ) -> Option<ReaperString> {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .track_fx_get_named_config_parm_as_string(
                    self.raw_track(),
                    self.location(),
                    param_name,
                    buffer_size,
                )
                .ok()
        }
    }

    fn raw_track(&self) -> MediaTrack {
        self.fx_chain.track().raw()
    }

    fn location(&self) -> TrackFxLocation {
        match self.fx_chain.kind() {
            TrackFxChainType::NormalFxChain => TrackFxLocation::NormalFxChain(self.index),
            TrackFxChainType::InputFxChain => TrackFxLocation::InputFxChain(self.index),
        }
    }
}
