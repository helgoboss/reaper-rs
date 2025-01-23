use super::raw::{reaper_plugin_info_t, HINSTANCE};
use super::PluginContext;
use crate::StaticPluginContext;
use fragile::Fragile;
use std::cell::RefCell;
use std::error::Error;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

/// This function catches panics before they reach REAPER.
///
/// This function is supposed to be wrapped around all Rust code that is called directly by REAPER,
/// e.g. control surface callbacks or command hooks. Its purpose it to establish a fault barrier in
/// order to prevent REAPER from crashing if a non-recoverable error occurs in the plug-in (a
/// panic).
///
/// Right now this doesn't do anything else than calling `catch_unwind()` but it might do more in
/// the future. Please note that logging is *not* supposed to be done here. It should be done in the
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
    static_context: StaticPluginContext,
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

/// Returns whether the shared library module is still attached. If this was an executable,
/// it would be analogous to still being inside the main function
/// (see https://github.com/rust-lang/rust/issues/110708).
///
/// If this returns `false`, no code should call [`std::thread::current`] anymore, because it will
/// panic, at least on Windows. This can have severe consequences because code that is executed
/// when this returns `false` is usually destructor code. And panicking in a destructor will abort
/// the application. This leads to a crash. This caused regular crashes on REAPER exit:
/// (https://github.com/helgoboss/helgobox/issues/1423)
pub fn module_is_attached() -> bool {
    MODULE_IS_ATTACHED.load(Ordering::Relaxed)
}

static MODULE_IS_ATTACHED: AtomicBool = AtomicBool::new(true);

/// This function executes all registered plug-in destroy hooks.
///
/// It's supposed to be called latest when the plug-in is unloaded. This is taken care of
/// automatically by macros in the `reaper-macros` crate. But you can also call it by yourself
/// **before** the plug-in gets detached. On Windows, this has the advantage that you can arbitrary
/// code. If you wait for DLL detachment, you can't execute certain code (see
/// [`PluginDestroyHook::callback`]).
///
/// Extension plug-in unloading happens when exiting REAPER but can also happen when the plug-in
/// can't be loaded completely. Then it's important to clean up static variables.
///
/// If we don't do this and `ReaperPluginEntry` returns 0 (triggers plug-in unload) at
/// a time when some static variables (e.g. high-level Reaper) are already
/// initialized and hooks are registered, we get an access violation because they
/// are not automatically freed on unload and destructors (drop functions) are not run!
///
/// # Panics
///
/// Panics if not called in the main thread.
pub fn execute_plugin_destroy_hooks() {
    // Indicate that we are not within "main" anymore. Plug-in destroy hooks must not call
    // std::thread::current anymore.
    MODULE_IS_ATTACHED.store(false, Ordering::Relaxed);
    // Run destruction in reverse order (recently constructed things will be destroyed first)
    for hook in PLUGIN_DESTROY_HOOKS.get().borrow_mut().drain(..).rev() {
        // We use println instead of tracing because tracing might not work anymore at this point
        println!("Executing plug-in destroy hook {}", hook.name);
        (hook.callback)();
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
pub unsafe fn register_plugin_destroy_hook(hook: PluginDestroyHook) {
    tracing::debug!(msg = "Registering plug-in destroy hook", %hook.name);
    PLUGIN_DESTROY_HOOKS.get().borrow_mut().push(hook);
}

static PLUGIN_DESTROY_HOOKS: LazyLock<Fragile<RefCell<Vec<PluginDestroyHook>>>> =
    LazyLock::new(Fragile::default);

/// A plug-in destroy hook. See [`register_plugin_destroy_hook`].
pub struct PluginDestroyHook {
    /// Descriptive name. Useful for debugging.
    pub name: &'static str,
    /// Callback that will be invoked when the plug-in module gets unloaded.
    ///
    /// If you are not calling [`execute_plugin_destroy_hooks`] explicitly before the DLL is
    /// attached, the function provided here must not call `std::thread::current` or access
    /// thread-locals. This would cause panics on Windows. Most likely it would also cause a real
    /// crash, because the panic probably occurs while executing `Drop` (destructor). Such a crash
    /// might not always get visible because it probably happens when exiting REAPER. But it's
    /// definitely visible in the event viewer.
    ///
    /// In practice, it can sometimes be hard to fulfill above requirement because destructor code
    /// runs automatically. Some value in the destroyed struct might have complex disposal
    /// logic that you can't change (e.g. Sentry client). In such a case, it's probably better
    /// to not register the destroy hook. Then exiting REAPER won't crash. The only downside is
    /// that, if you write a VST, REAPER for Windows preference "VST => Allow complete unload of
    /// VST plug-ins" should not be ticked. Otherwise, the unload will fail. But ticking that
    /// option is not the best idea anyway.
    pub callback: fn(),
}
