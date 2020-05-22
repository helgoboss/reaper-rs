use super::raw::{reaper_plugin_info_t, HINSTANCE};
use super::PluginContext;
use crate::StaticExtensionPluginContext;
use std::error::Error;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// This function catches panics before they reach REAPER.
///
/// This function is supposed to be wrapped around all Rust code that is called directly by REAPER,
/// e.g. control surface callbacks or command hooks. Its purpose it to establish a fault barrier in
/// order to prevent REAPER from crashing if a non-recoverable error occurs in the plug-in (a
/// panic).
///
/// Right now this doesn't do anything else than calling `catch_unwind()` but it might do more in
/// future. Please note that logging is *not* supposed to be done here. It should be done in the
/// panic hook instead.
pub fn firewall<F: FnOnce() -> R, R>(f: F) -> Option<R> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

/// This is a convenience function for bootstrapping extension plug-ins.
///
/// This function basically translates the REAPER extension plug-in main entry point signature
/// (`ReaperPluginEntry()`) to a typical Rust main entry point signature (`main()`). It is
/// intended to be used by macros in the `reaper-macros` crate.
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer.
pub unsafe fn bootstrap_extension_plugin(
    h_instance: HINSTANCE,
    rec: *mut reaper_plugin_info_t,
    static_context: StaticExtensionPluginContext,
    init: fn(PluginContext) -> Result<(), Box<dyn Error>>,
) -> i32 {
    // TODO-low Log early errors
    firewall(|| {
        if rec.is_null() {
            return 0;
        }
        let rec = *rec;
        let context = match PluginContext::from_extension_plugin(h_instance, rec, static_context) {
            Ok(c) => c,
            Err(_) => return 0,
        };
        match init(context) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
    .unwrap_or(0)
}
