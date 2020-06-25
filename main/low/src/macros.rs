/// Macro which gathers things that go into the static REAPER VST plug-in context.
///
/// This macro provides module entry points which gather some handles for creating
/// a REAPER VST plug-in context. The gathered handles are exposed via the function
/// [`static_vst_plugin_context()`] and are intended to be passed to
/// [`PluginContext::from_vst_plugin()`].
///
/// # Example
///
/// ```
/// use reaper_low::{reaper_vst_plugin, static_vst_plugin_context, StaticVstPluginContext};
///
/// reaper_vst_plugin!();
///
/// let static_context: StaticVstPluginContext = static_vst_plugin_context();
/// ```
///
/// [`PluginContext::from_vst_plugin()`]:
/// struct.PluginContext.html#method.from_vst_plugin
/// [`static_vst_plugin_context()`]: fn.static_vst_plugin_context.html
#[macro_export]
macro_rules! reaper_vst_plugin {
    () => {
        mod reaper_vst_plugin {
            /// Windows entry point for getting hold of the module handle (HINSTANCE).
            ///
            /// This is called by REAPER for Windows at startup time.
            #[cfg(target_family = "windows")]
            #[allow(non_snake_case)]
            #[no_mangle]
            extern "system" fn DllMain(
                hinstance: reaper_low::raw::HINSTANCE,
                reason: u32,
                _: *const u8,
            ) -> u32 {
                if (reason == reaper_low::raw::DLL_PROCESS_ATTACH) {
                    reaper_low::register_hinstance(hinstance);
                }
                1
            }

            /// Linux entry point for getting hold of the SWELL function provider.
            ///
            /// This is called by REAPER for Linux at startup time.
            #[cfg(target_os = "linux")]
            #[allow(non_snake_case)]
            #[no_mangle]
            extern "C" fn SWELL_dllMain(
                hinstance: reaper_low::raw::HINSTANCE,
                reason: u32,
                get_func: Option<
                    unsafe extern "C" fn(
                        name: *const std::os::raw::c_char,
                    ) -> *mut std::os::raw::c_void,
                >,
            ) -> std::os::raw::c_int {
                reaper_low::register_swell_function_provider(get_func);
                1
            }
        }
    };
}
