use crate::{Reaper, ReaperFunctionPointers, ReaperPluginContext};

impl Reaper {
    /// Gives access to the REAPER function pointers.
    pub fn pointers(&self) -> &ReaperFunctionPointers {
        &self.pointers
    }

    /// Returns the plug-in context.
    pub fn plugin_context(&self) -> &ReaperPluginContext {
        self.plugin_context
            .as_ref()
            .expect("plug-in context not available on demo instances")
    }
}
