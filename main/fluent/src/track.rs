use crate::access::{ReadAccess, WriteAccess};
use crate::{FxChain, Project, ProjectDesc, Reaper};
use reaper_low::raw::GUID;
use reaper_medium::{MediaTrack, ReaperStringArg, TrackFxChainType};
use std::marker::PhantomData;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TrackDesc {
    project_desc: ProjectDesc,
    guid: GUID,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Track<'a, A> {
    raw: MediaTrack,
    _p: PhantomData<&'a A>,
}

impl TrackDesc {
    pub fn new(project_desc: ProjectDesc, guid: GUID) -> Self {
        Self { project_desc, guid }
    }

    // pub fn ptr(&self) -> Option<TrackPtr> {
    //     let project = self.project_desc.get()?;
    //     let track = project.tracks().find(|t| t.guid() == self.guid);
    //     track
    // }
    //
    // pub fn normal_fx_chain(&self) -> FxChainDesc {
    //     FxChainDesc::new(*self, TrackFxChainType::NormalFxChain)
    // }
}

impl<A> Track<'_, A> {
    pub(crate) fn new(raw: MediaTrack) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }

    pub fn desc(&self) -> TrackDesc {
        // TrackDesc::new(self.project(), self.guid())
        todo!()
    }

    pub fn normal_fx_chain(&self) -> FxChain<ReadAccess> {
        self.normal_fx_chain_internal()
    }

    pub fn set_name<'b>(&mut self, name: impl Into<ReaperStringArg<'b>>) {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_set_name(self.raw, name);
        }
    }

    pub fn normal_fx_chain_mut(&self) -> FxChain<WriteAccess> {
        self.normal_fx_chain_internal()
    }

    fn normal_fx_chain_internal<B>(&self) -> FxChain<B> {
        FxChain::new(Track::new(self.raw), TrackFxChainType::NormalFxChain)
    }

    pub fn project(&self) -> Project<ReadAccess> {
        let raw = unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_project(self.raw)
                .expect("REAPER >= 5.95 required for this operation")
        };
        Project::new(raw)
    }

    pub fn raw(&self) -> MediaTrack {
        self.raw
    }

    pub fn guid(&self) -> GUID {
        unsafe {
            Reaper::get()
                .medium_reaper()
                .get_set_media_track_info_get_guid(self.raw)
        }
    }
}
