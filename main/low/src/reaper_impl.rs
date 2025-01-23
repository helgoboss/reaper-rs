use crate::raw::preview_register_t;
use crate::{PluginContext, Reaper, ReaperFunctionPointers};
use std::sync::OnceLock;

static INSTANCE: OnceLock<Reaper> = OnceLock::new();

impl Reaper {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    pub fn make_available_globally(functions: Reaper) -> Result<(), Reaper> {
        INSTANCE.set(functions)
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Reaper {
        INSTANCE
            .get()
            .expect("call `make_available_globally()` before using `get()`")
    }

    /// Gives access to the REAPER function pointers.
    pub fn pointers(&self) -> &ReaperFunctionPointers {
        &self.pointers
    }

    /// Returns the plug-in context.
    pub fn plugin_context(&self) -> &PluginContext {
        self.plugin_context
            .as_ref()
            .expect("plug-in context not available on demo instances")
    }
}

impl std::fmt::Debug for ReaperFunctionPointers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaperFunctionPointers")
            .field("loaded_count", &self.loaded_count)
            .field("total_count", &Self::TOTAL_COUNT)
            .finish()
    }
}

impl std::fmt::Debug for preview_register_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("preview_register_t").finish()
    }
}
