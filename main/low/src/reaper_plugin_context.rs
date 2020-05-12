#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use super::raw::{reaper_plugin_info_t, REAPER_PLUGIN_VERSION};
use crate::raw;
use std::ffi::CStr;
use std::os::raw::{c_int, c_void};
use std::ptr::{null_mut, NonNull};
use vst::api::HostCallbackProc;
use vst::plugin::HostCallback;

type FunctionProvider = Box<dyn Fn(&CStr) -> isize>;

/// This represents the context which is needed to access REAPER functions from plug-ins.
///
/// Once obtained, it is supposed to be passed to [`Reaper::load()`].
///
/// [`Reaper::load()`]: struct.Reaper.html#method.load
pub struct ReaperPluginContext {
    /// Function which obtains a function pointer for a given REAPER function name.
    pub(crate) function_provider: FunctionProvider,
}

/// Additional type-specific data available in the plug-in context.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReaperPluginContextData {
    /// This is an extension plug-in.
    ExtensionPluginData(ReaperExtensionPluginContextData),
    /// This is a VST plug-in.
    VstPluginData(ReaperVstPluginContextData),
}

/// Additional data available in the context of extension plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperExtensionPluginContextData {}

/// Additional data available in the context of VST plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperVstPluginContextData {}

impl ReaperPluginContext {
    /// Creates a plug-in context from an extension entry point plug-in info.
    ///
    /// It requires the [`reaper_plugin_info_t`] struct that REAPER provides when calling the
    /// `ReaperPluginEntry` function (the main entry point for any extension plug-in).
    ///
    /// It's recommended to use the `reaper_extension_plugin` macro in the
    /// [reaper-macros](https://crates.io/crates/reaper-macros) crate instead of calling
    /// this function directly.
    ///
    /// [`reaper_plugin_info_t`]: raw/struct.reaper_plugin_info_t.html
    pub fn from_extension_plugin(
        rec: *mut reaper_plugin_info_t,
    ) -> Result<ReaperPluginContext, &'static str> {
        let function_provider = create_extension_plugin_function_provider(rec)?;
        Ok(ReaperPluginContext { function_provider })
    }

    /// Creates a plug-in context from a VST host callback.
    ///
    /// It requires the host callback which [vst-rs](https://crates.io/crates/vst) passes to the
    /// plugin's [`new()`] function.
    ///
    /// [`new()`]: https://docs.rs/vst/0.2.0/vst/plugin/trait.Plugin.html#method.new
    pub fn from_vst_plugin(host: HostCallback) -> Result<ReaperPluginContext, &'static str> {
        let host_callback = host.raw_callback().ok_or("Host callback not available")?;
        let function_provider = create_vst_plugin_function_provider(host_callback);
        Ok(ReaperPluginContext { function_provider })
    }

    /// Returns a generic API function by its name.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetFunc(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
        todo!()
    }

    /// On Windows, this returns the `HINSTANCE` passed to `DllMain`.
    ///
    /// The returned `HINSTANCE` represents the handle of the module (DLL) containing the plug-in.
    ///
    /// On Linux, this returns `None`.
    pub fn h_instance(&self) -> raw::HINSTANCE {
        todo!()
    }

    /// On Linux, this returns a generic SWELL API function by its name.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetSwellFunc(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
        todo!()
    }
}

impl ReaperExtensionPluginContextData {
    /// Returns the caller version from `reaper_plugin_info_t`.
    pub fn caller_version(&self) -> c_int {
        todo!()
    }

    /// Returns the main window from `reaper_plugin_info_t`.
    pub fn hwnd_main(&self) -> raw::HWND {
        todo!()
    }

    /// This is the same like `plugin_register()`.
    ///
    /// Usually not needed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn Register(
        name: *const ::std::os::raw::c_char,
        infostruct: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        todo!()
    }
}

impl ReaperVstPluginContextData {
    /// Returns host context information for this VST plug-in.
    ///
    /// Some API functions take context pointers such as `*mut ReaProject`, `*mut MediaTrack`,
    /// `*mut MediaItem_Take` etc. A VST running within REAPER can request its own context.
    ///
    /// Valid values for request include:
    /// - 1: retrieve `*mut MediaTrack` (`null` if not running as track-FX)
    /// - 2: retrieve `*mut MediaItem_Take` (`null` if not running as take-FX)
    /// - 3: retrieve `*mut ReaProject`
    /// - 5: retrieve channel count of containing track (result is a `c_int`)
    pub fn request_host_context(&self, request_value: isize) -> *mut c_void {
        todo!()
    }
}

fn create_extension_plugin_function_provider(
    rec: *mut reaper_plugin_info_t,
) -> Result<FunctionProvider, &'static str> {
    if rec.is_null() {
        return Err("rec not available");
    }
    let rec = unsafe { *rec };
    if rec.caller_version != REAPER_PLUGIN_VERSION as c_int {
        return Err("Caller version doesn't match");
    }
    let get_func = rec.GetFunc.ok_or("GetFunc function pointer not set")?;
    Ok(Box::new(move |name| unsafe {
        get_func(name.as_ptr()) as isize
    }))
}

fn create_vst_plugin_function_provider(host_callback: HostCallbackProc) -> FunctionProvider {
    Box::new(move |name| {
        #[allow(overflowing_literals)]
        host_callback(
            null_mut(),
            0xdead_beef,
            0xdead_f00d,
            0,
            name.as_ptr() as *mut c_void,
            0.0,
        )
    })
}
