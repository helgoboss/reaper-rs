use crate::{raw, GetSwellFunc, StaticPluginContext};
use fragile::Fragile;
use std::ptr::null_mut;

/// Exposes the (hopefully) obtained static plug-in context.
///
/// This is typically called by one of the plugin macros.
pub fn static_plugin_context() -> StaticPluginContext {
    StaticPluginContext {
        h_instance: hinstance::HINSTANCE
            .get()
            .map(|i| *i.get())
            .unwrap_or(null_mut()),
        get_swell_func: swell::GET_SWELL_FUNC.get().copied(),
    }
}

/// Registers the given SWELL function provider globally.
///
/// As a result it can later be picked up e.g. by using [`static_extension_plugin_context()`].
/// This is typically called by some SWELL entry point in one of the plugin macros.
///
/// [`static_extension_plugin_context()`]: fn.static_extension_plugin_context.html
pub fn register_swell_function_provider(get_func: GetSwellFunc) -> Result<(), &'static str> {
    // Save provider in static variable.
    swell::GET_SWELL_FUNC
        .set(get_func)
        .map_err(|_| "SWELL function provider registered already")?;
    // On Linux Rust will get informed first about the SWELL function provider, so we need to pass
    // it on to the C++ side.
    #[cfg(target_os = "linux")]
    unsafe {
        swell::register_swell_function_provider_called_from_rust(get_func);
    }
    Ok(())
}

/// Registers the module handle globally.
///
/// As a result it can later be picked up by using [`static_plugin_context()`].
/// This is typically called on Windows only by some entry point in one of the plugin macros.
///
/// [`static_plugin_context()`]: fn.static_plugin_context.html
pub fn register_hinstance(hinstance: raw::HINSTANCE) -> Result<(), &'static str> {
    hinstance::HINSTANCE
        .set(Fragile::new(hinstance))
        .map_err(|_| "HINSTANCE registered already")
}

mod swell {
    use std::sync::OnceLock;

    /// On Linux/macOS this will contain the SWELL function provider after REAPER start.
    pub static GET_SWELL_FUNC: OnceLock<crate::GetSwellFunc> = OnceLock::new();

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
        if let Some(get_func) = get_func {
            let _ = crate::register_swell_function_provider(get_func);
        }
    }
}

mod hinstance {
    use crate::raw;
    use fragile::Fragile;
    use std::sync::OnceLock;

    /// On Windows, this will contain the module handle after the plug-in has been loaded.
    pub static HINSTANCE: OnceLock<Fragile<raw::HINSTANCE>> = OnceLock::new();
}
