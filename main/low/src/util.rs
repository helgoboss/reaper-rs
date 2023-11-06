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

/// This function executes all registered plug-in destroy hooks.
///
/// It's supposed to be called when the extension plug-in is unloaded. This is taken care of
/// automatically by macros in the `reaper-macros` crate.
///
/// Extension plug-in unloading happens when exiting REAPER but can also happen when the plug-in
/// can't be loaded completely. Then it's important to clean up static variables.
///
/// If we don't do this and `ReaperPluginEntry` returns 0 (triggers plug-in unload) at
/// a time when some static variables (e.g. high-level Reaper) are already
/// initialized and hooks are registered, we get an access violation because they
/// are not automatically freed on unload and destructors (drop functions) are not run!
///
/// # Safety
///
/// Must only be called in main thread.
pub unsafe fn execute_plugin_destroy_hooks() {
    // Run destruction in reverse order (recently constructed things will be destroyed first)
    for f in PLUGIN_DESTROY_HOOKS.drain(..).rev() {
        f();
    }
}

/// Registers a function that will be executed when the plug-in module gets unloaded.
///
/// This is supposed to be used from the *reaper-rs* low-level API but also higher-level APIs
/// whenever they register static variables that require manual cleanup on plug-in unload.
///
/// # Safety
///
/// Must only be called in main thread.
pub unsafe fn register_plugin_destroy_hook(f: fn()) {
    PLUGIN_DESTROY_HOOKS.push(f);
}

static mut PLUGIN_DESTROY_HOOKS: Vec<fn()> = Vec::new();
