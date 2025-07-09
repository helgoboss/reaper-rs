use crate::access::{ReadAccess, WriteAccess};
use crate::{FreeFn, Model};
use fragile::Fragile;
use reaper_low::firewall;
use reaper_medium::ReaperSession;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::OnceLock;

// TODO-high Take care of executing Drop. Important if deployed as part of VST plug-in that
//  gets unloaded on Windows. Maybe Fragile helps here? Debug Drop invocations on Windows!
static INSTANCE: OnceLock<Reaper> = OnceLock::new();

#[derive(Debug)]
pub struct Reaper {
    medium_reaper: reaper_medium::Reaper,
    medium_session: Fragile<RefCell<ReaperSession>>,
    model: Fragile<RefCell<Model<WriteAccess>>>,
}

impl Reaper {
    #[allow(clippy::result_large_err)]
    pub fn install_globally(medium_session: ReaperSession) -> Result<(), Self> {
        let reaper = Self {
            medium_reaper: medium_session.reaper().clone(),
            medium_session: Fragile::new(RefCell::new(medium_session)),
            model: Fragile::new(RefCell::new(Model(WriteAccess))),
        };
        INSTANCE.set(reaper)
    }

    pub fn get() -> &'static Self {
        INSTANCE
            .get()
            .expect("You must first call `Reaper::install_globally` in order to use this function.")
    }

    pub fn medium_session(&self) -> Ref<ReaperSession> {
        self.medium_session.get().borrow()
    }

    pub fn medium_session_mut(&self) -> RefMut<ReaperSession> {
        self.medium_session.get().borrow_mut()
    }

    pub fn medium_reaper(&self) -> &reaper_medium::Reaper {
        &self.medium_reaper
    }

    pub fn execute_later<F: FreeFn>(&self) {
        extern "C" fn call_and_unregister_timer<F: FreeFn>() {
            firewall(F::call);
            Reaper::get()
                .medium_session_mut()
                .plugin_register_remove_timer(call_and_unregister_timer::<F>);
        }
        let _ = self
            .medium_session_mut()
            .plugin_register_add_timer(call_and_unregister_timer::<F>);
    }

    pub fn model(&self) -> Ref<Model<ReadAccess>> {
        // self.model.get().borrow();
        todo!()
    }

    pub fn model_mut(&self) -> RefMut<Model<WriteAccess>> {
        self.model.get().borrow_mut()
    }
}
