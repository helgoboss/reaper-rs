/// Macro which gathers end exposes the static REAPER VST plug-in context.
///
/// This macro provides module entry points which gather some static data for creating
/// a REAPER VST plug-in context. The gathered data is exposed via the function
/// `reaper_vst_plugin::static_context()` and is intended to be passed to
/// [`ReaperPluginContext::from_extension_plugin()`].
///
/// # Example
///
/// ```
/// use reaper_low::reaper_vst_plugin;
///
/// reaper_vst_plugin!();
///
/// let static_context = reaper_vst_plugin::static_context();
/// ```
///
/// [`ReaperPluginContext::from_extension_plugin()`]:
/// struct.ReaperPluginContext.html#method.from_extension_plugin
#[macro_export]
macro_rules! reaper_vst_plugin {
    () => {
        mod reaper_vst_plugin {
            static mut REAPER_VST_PLUGIN_HINSTANCE: $crate::raw::HINSTANCE = std::ptr::null_mut();
            static INIT_REAPER_VST_PLUGIN_HINSTANCE: std::sync::Once = std::sync::Once::new();

            #[cfg(target_os = "windows")]
            #[allow(non_snake_case)]
            #[no_mangle]
            extern "C" fn DllMain(
                hinstance: $crate::raw::HINSTANCE,
                reason: u32,
                _: *const u8,
            ) -> u32 {
                if (reason == $crate::raw::DLL_PROCESS_ATTACH) {
                    INIT_REAPER_VST_PLUGIN_HINSTANCE.call_once(|| {
                        unsafe { REAPER_VST_PLUGIN_HINSTANCE = hinstance };
                    });
                }
                1
            }

            static mut REAPER_VST_PLUGIN_SWELL_GET_FUNC: Option<
                unsafe extern "C" fn(
                    name: *const std::os::raw::c_char,
                ) -> *mut std::os::raw::c_void,
            > = None;
            static INIT_REAPER_VST_PLUGIN_SWELL_GET_FUNC: std::sync::Once = std::sync::Once::new();

            #[allow(non_snake_case)]
            #[no_mangle]
            extern "C" fn SWELL_dllMain(
                hinstance: $crate::raw::HINSTANCE,
                reason: u32,
                get_func: Option<
                    unsafe extern "C" fn(
                        name: *const std::os::raw::c_char,
                    ) -> *mut std::os::raw::c_void,
                >,
            ) -> std::os::raw::c_int {
                if (reason == $crate::raw::DLL_PROCESS_ATTACH) {
                    INIT_REAPER_VST_PLUGIN_SWELL_GET_FUNC.call_once(|| {
                        unsafe { REAPER_VST_PLUGIN_SWELL_GET_FUNC = get_func };
                    });
                }
                1
            }

            pub fn static_context() -> $crate::StaticReaperVstPluginContext {
                $crate::StaticReaperVstPluginContext {
                    h_instance: unsafe { REAPER_VST_PLUGIN_HINSTANCE },
                    get_swell_func: unsafe { REAPER_VST_PLUGIN_SWELL_GET_FUNC },
                    ..Default::default()
                }
            }
        }
    };
}
