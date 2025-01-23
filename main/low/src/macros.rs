/// Macro which gathers things that go into the static REAPER VST plug-in context.
///
/// This macro provides module entry points which gather some handles for creating
/// a REAPER VST plug-in context. The gathered handles are exposed via the function
/// [`static_plugin_context()`] and are intended to be passed to
/// [`PluginContext::from_vst_plugin()`].
///
/// # Example
///
/// ```
/// use reaper_low::{reaper_vst_plugin, static_plugin_context, StaticPluginContext};
///
/// reaper_vst_plugin!();
///
/// let static_context: StaticPluginContext = static_plugin_context();
/// ```
///
/// [`PluginContext::from_vst_plugin()`]:
/// struct.PluginContext.html#method.from_vst_plugin
/// [`static_plugin_context()`]: fn.static_plugin_context.html
#[macro_export]
macro_rules! reaper_vst_plugin {
    () => {
        mod reaper_vst_plugin {
            reaper_low::dll_main!();
            reaper_low::swell_dll_main!();
        }
    };
}

/// Macro which generates and exports the `DllMain` function on Windows.
#[macro_export]
macro_rules! dll_main {
    () => {
        /// Windows entry and exit point for getting hold of the module handle (HINSTANCE) and
        /// clean-up.
        ///
        /// Called by REAPER for Windows once at startup time with DLL_PROCESS_ATTACH and once
        /// at exit time or manual unload time (after initial scan, whenever plug-in
        /// initialization failed or if "Allow complete unload of VST plug-ins" is enabled and
        /// last instance gone) with DLL_PROCESS_DETACH.
        #[cfg(target_family = "windows")]
        #[allow(non_snake_case)]
        #[no_mangle]
        extern "system" fn DllMain(
            hinstance: reaper_low::raw::HINSTANCE,
            reason: u32,
            _: *const u8,
        ) -> u32 {
            if reason == reaper_low::raw::DLL_PROCESS_ATTACH {
                let _ = reaper_low::register_hinstance(hinstance);
            } else if reason == reaper_low::raw::DLL_PROCESS_DETACH {
                reaper_low::execute_plugin_destroy_hooks();
            }
            1
        }
    };
}

/// Macro which generates and exports the `SWELL_dllMain` function on Linux.
#[macro_export]
macro_rules! swell_dll_main {
    () => {
        /// Linux entry and exit point for getting hold of the SWELL function provider.
        ///
        /// Clean-up is neither necessary nor desired on Linux at the moment because even if
        /// "Allow complete unload of VST plug-ins" is enabled in REAPER, the module somehow
        /// seems to stick around or at least the statics don't get dropped. Dropping them
        /// manually via `execute_plugin_destroy_hooks()` as we do on Windows - and thereby also
        /// removing any globally set up `Reaper` struct - would cause harm. Because as soon as
        /// we add an instance of the plug-in again, the important `call_once()` things
        /// wouldn't be executed anymore and thus the global `Reaper` struct wouldn't be
        /// available. Fortunately, the issue why we introduced proper clean-up in the first
        /// place (non-freed TCP ports) doesn't even exist on Linux. So everything is fine
        /// apart from non-freed memory, which we can't do anything about because "Allow
        /// complete unload of VST plug-ins" doesn't seem to be properly supported on REAPER
        /// for Linux at the time of this writing. If it will be one day, I would hope that this
        /// function will be invoked with DLL_PROCESS_DETACH really only on complete unload,
        /// as on Windows! Then we could use the same code as on Windows. Now it's called even
        /// if the module is not completely unloaded.
        ///
        /// Called by REAPER for Linux once at startup time with DLL_PROCESS_ATTACH and once
        /// at exit time or manual unload time (after initial scan, whenever plug-in
        /// initialization failed or if "Allow complete unload of VST plug-ins" is enabled and
        /// last instance gone) with DLL_PROCESS_DETACH.
        ///
        /// In case anybody wonders where's the SWELL entry point for macOS:
        /// `swell-modstub-custom.mm`.
        #[cfg(target_os = "linux")]
        #[allow(non_snake_case)]
        #[no_mangle]
        extern "C" fn SWELL_dllMain(
            hinstance: reaper_low::raw::HINSTANCE,
            reason: u32,
            get_func: Option<GetSwellFunc>,
        ) -> std::os::raw::c_int {
            if reason == reaper_low::raw::DLL_PROCESS_ATTACH {
                if let Some(get_func) = get_func {
                    let _ = reaper_low::register_swell_function_provider(get_func);
                }
            }
            1
        }
    };
}
