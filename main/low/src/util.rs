use super::raw::{reaper_plugin_info_t, HINSTANCE};
use super::ReaperPluginContext;
use std::error::Error;
use std::panic::{catch_unwind, UnwindSafe};

/// This function is supposed to be wrapped around all Rust code that is called directly by REAPER,
/// e.g. control surface callbacks or command hooks. Its purpose it to establish a fault barrier in
/// order to prevent REAPER from crashing if a non-recoverable error occurs in the plug-in (a
/// panic).
///
/// Right now this doesn't do anything else than calling `catch_unwind` but it might do more in
/// future. Please note that logging is **not** supposed to be done here. It should be done in the
/// panic hook instead.
pub fn firewall<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Option<R> {
    catch_unwind(f).ok()
}

/// This function basically translates the REAPER extension plug-in main entry point signature
/// (`ReaperPluginEntry`) to a typical Rust main entry point signature (`main`). It's primarily
/// intended to be used by macros in the `reaper-rs-macros` crate.
pub fn bootstrap_extension_plugin(
    _h_instance: HINSTANCE,
    rec: *mut reaper_plugin_info_t,
    init: fn(&ReaperPluginContext) -> Result<(), Box<dyn Error>>,
) -> i32 {
    // TODO-low Log early errors
    firewall(|| {
        let context = match ReaperPluginContext::from_extension_plugin(rec) {
            Err(_) => return 0,
            Ok(c) => c,
        };
        match init(&context) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
    .unwrap_or(0)
}
