use crate::{Model, ProjectDesc, TrackDesc};
use fragile::Fragile;
use reaper_medium::{ProjectContext, ProjectRef, ReaperSession, TrackDefaultsBehavior};
use std::cell::{Ref, RefCell, RefMut};
use std::sync::OnceLock;

// TODO-high Take care of executing Drop. Important if deployed as part of VST plug-in that
//  gets unloaded on Windows. Maybe Fragile helps here? Debug Drop invocations on Windows!
static INSTANCE: OnceLock<Reaper> = OnceLock::new();

#[derive(Debug)]
pub struct Reaper {
    medium_session: Fragile<ReaperSession>,
    model: Fragile<RefCell<Model>>,
}

impl Reaper {
    pub fn install_globally(medium_session: ReaperSession) -> Result<(), Self> {
        let reaper = Self {
            medium_session: Fragile::new(medium_session),
            model: Fragile::new(RefCell::new(Model(()))),
        };
        INSTANCE.set(reaper)
    }

    pub fn get() -> &'static Self {
        INSTANCE
            .get()
            .expect("You must first call `Reaper::install_globally` in order to use this function.")
    }

    pub fn medium_session(&self) -> &ReaperSession {
        self.medium_session.get()
    }

    pub fn medium_reaper(&self) -> &reaper_medium::Reaper {
        self.medium_session.get().reaper()
    }

    pub fn model(&self) -> Ref<Model> {
        self.model.get().borrow()
    }

    pub fn model_mut(&self) -> RefMut<Model> {
        self.model.get().borrow_mut()
    }
}
