use crate::{FxChainDesc, Project, ProjectDesc, Reaper};
use reaper_low::raw::GUID;
use reaper_medium::{MediaTrack, TrackFxChainType};
use std::marker::PhantomData;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TrackDesc {
    project_desc: ProjectDesc,
    guid: GUID,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Track<'a> {
    project: &'a Project<'a>,
    raw: MediaTrack,
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

impl<'a> Track<'a> {
    pub(crate) fn new(project: &'a Project<'a>, raw: MediaTrack) -> Self {
        Self { project, raw }
    }

    pub fn desc(&self) -> TrackDesc {
        TrackDesc::new(self.project.desc(), self.guid())
    }

    pub fn project(&self) -> &Project {
        &self.project
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
