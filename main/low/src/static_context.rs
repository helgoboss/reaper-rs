use crate::{
    raw, register_plugin_destroy_hook, GetSwellFunc, StaticExtensionPluginContext,
    StaticVstPluginContext,
};

/// Exposes the (hopefully) obtained static extension plug-in context.
///
/// This is typically called by one of the plugin macros.
pub fn static_extension_plugin_context() -> StaticExtensionPluginContext {
    StaticExtensionPluginContext {
        get_swell_func: unsafe { swell::GET_SWELL_FUNC },
    }
}

/// Exposes the (hopefully) obtained static VST plug-in context.
///
/// This is typically called by one of the plugin macros.
pub fn static_vst_plugin_context() -> StaticVstPluginContext {
    StaticVstPluginContext {
        h_instance: unsafe { hinstance::HINSTANCE },
        get_swell_func: unsafe { swell::GET_SWELL_FUNC },
    }
}

/// Registers the given SWELL function provider globally.
///
/// As a result it can later be picked up e.g. by using [`static_extension_plugin_context()`].
/// This is typically called by some SWELL entry point in one of the plugin macros.
///
/// [`static_extension_plugin_context()`]: fn.static_extension_plugin_context.html
pub fn register_swell_function_provider(get_func: Option<GetSwellFunc>) {
    // Save provider in static variable.
    swell::INIT_GET_SWELL_FUNC.call_once(|| unsafe {
        swell::GET_SWELL_FUNC = get_func;
        register_plugin_destroy_hook(|| swell::GET_SWELL_FUNC = None);
    });
    // On Linux Rust will get informed first about the SWELL function provider, so we need to pass
    // it on to the C++ side.
    #[cfg(target_os = "linux")]
    unsafe {
        swell::register_swell_function_provider_called_from_rust(get_func);
    }
}

/// Registers the module handle globally.
///
/// As a result it can later be picked up by using [`static_vst_plugin_context()`].
/// This is typically called on Windows only by some entry point in one of the plugin macros.
///
/// [`static_vst_plugin_context()`]: fn.static_vst_plugin_context.html
pub fn register_hinstance(hinstance: raw::HINSTANCE) {
    // Save handle in static variable.
    hinstance::INIT_HINSTANCE.call_once(|| unsafe {
        hinstance::HINSTANCE = hinstance;
        register_plugin_destroy_hook(|| hinstance::HINSTANCE = std::ptr::null_mut());
    });
}

mod swell {
    use std::os::raw::{c_char, c_void};
    use std::sync::Once;

    /// On Linux/macOS this will contain the SWELL function provider after REAPER start.
    pub static mut GET_SWELL_FUNC: Option<
        unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
    > = None;
    pub static INIT_GET_SWELL_FUNC: Once = Once::new();

    #[cfg(target_os = "linux")]
    extern "C" {
        /// On Linux, this function is implemented on C++ side (`swell-modstub-generic-custom.cpp`).
        ///
        /// This is supposed to be called by Rust as soon as it gets hold of the SWELL function provider
        /// via the Linux SWELL main entry point `SWELL_dllMain()`. Calling this give the C++ side of
        /// the plug-in the chance to initialize its SWELL function pointers as well, which is necessary
        /// for many use cases (e.g. creating dialog windows). This is only needed on Linux. On Windows
        /// we don't have SWELL and on macOS the mechanism of obtaining SWELL is different.
        pub fn register_swell_function_provider_called_from_rust(
            get_func: Option<crate::GetSwellFunc>,
        );
    }

    /// On macOS, this function is called from Objective-C side (`swell-modstub-custom.mm`).
    ///
    /// It lets Rust know about the SWELL function provider. It's called on REAPER startup.
    #[cfg(target_os = "macos")]
    #[no_mangle]
    unsafe extern "C" fn register_swell_called_from_cpp(get_func: Option<crate::GetSwellFunc>) {
        crate::register_swell_function_provider(get_func);
    }
}

mod hinstance {
    use crate::raw;
    use std::sync::Once;

    /// On Windows this will contain the module handle after REAPER start.
    pub static mut HINSTANCE: raw::HINSTANCE = std::ptr::null_mut();
    pub static INIT_HINSTANCE: Once = Once::new();
}
