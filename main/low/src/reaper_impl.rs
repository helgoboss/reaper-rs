use crate::raw::preview_register_t;
use crate::{register_plugin_destroy_hook, PluginContext, Reaper, ReaperFunctionPointers};

// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Reaper> = None;

impl Reaper {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    pub fn make_available_globally(functions: Reaper) {
        static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();
        unsafe {
            INIT_INSTANCE.call_once(|| {
                INSTANCE = Some(functions);
                register_plugin_destroy_hook(|| INSTANCE = None);
            });
        }
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Reaper {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("call `make_available_globally()` before using `get()`")
        }
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
