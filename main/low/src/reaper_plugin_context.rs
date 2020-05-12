#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::raw;
use derive_more::Display;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null_mut, NonNull};
use vst::api::HostCallbackProc;
use vst::plugin::HostCallback;

/// This represents the context which is needed to access REAPER functions from plug-ins.
///
/// Once obtained, it is supposed to be passed to [`Reaper::load()`].
///
/// [`Reaper::load()`]: struct.Reaper.html#method.load
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperPluginContext {
    // The only reason why this is Option is that we can implement Default. We want Default in
    // order to write compilable example code in Rust documentation comments
    type_specific: TypeSpecificReaperPluginContext,
    h_instance: raw::HINSTANCE,
}

/// Additional stuff available in the plug-in context specific to a certain plug-in type.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum TypeSpecificReaperPluginContext {
    /// This is an extension plug-in.
    Extension(ReaperExtensionPluginContext),
    /// This is a VST plug-in.
    Vst(ReaperVstPluginContext),
}

/// Additional data available in the context of extension plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct ReaperExtensionPluginContext {
    caller_version: c_int,
    hwnd_main: NonNull<raw::HWND__>,
    register: unsafe extern "C" fn(name: *const c_char, infostruct: *mut c_void) -> c_int,
    get_func: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
}

/// Additional data available in the context of VST plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct ReaperVstPluginContext {
    host_callback: HostCallbackProc,
}

impl ReaperPluginContext {
    /// Creates a plug-in context from an extension entry point plug-in info.
    ///
    /// It requires the pointer to a [`reaper_plugin_info_t`] struct which REAPER provides when
    /// calling the `ReaperPluginEntry` function (the main entry point for any extension
    /// plug-in).
    ///
    /// It's recommended to use the `reaper_extension_plugin` macro in the
    /// [reaper-macros](https://crates.io/crates/reaper-macros) crate instead of calling
    /// this function directly.
    ///
    /// # Errors
    ///
    /// Returns an error if the given plug-in info is not suitable for loading REAPER functions.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`reaper_plugin_info_t`]: raw/struct.reaper_plugin_info_t.html
    pub unsafe fn from_extension_plugin(
        h_instance: raw::HINSTANCE,
        rec: raw::reaper_plugin_info_t,
    ) -> Result<ReaperPluginContext, ContextFromExtensionPluginError> {
        use ContextFromExtensionPluginError::*;
        if rec.caller_version != raw::REAPER_PLUGIN_VERSION as c_int {
            return Err(CallerVersionIncompatible);
        }
        let get_func = rec.GetFunc.ok_or(FunctionProviderNotAvailable)?;
        let register = rec
            .Register
            .expect("plug-in info doesn't container Register function pointer");
        Ok(ReaperPluginContext {
            type_specific: TypeSpecificReaperPluginContext::Extension(
                ReaperExtensionPluginContext {
                    caller_version: rec.caller_version,
                    hwnd_main: NonNull::new(rec.hwnd_main)
                        .expect("plug-in info doesn't contain main window handle"),
                    register,
                    get_func,
                },
            ),
            h_instance,
        })
    }

    /// Creates a plug-in context from a VST host callback.
    ///
    /// It requires the host callback which [vst-rs](https://crates.io/crates/vst) passes to the
    /// plugin's [`new()`] function.
    ///
    /// # Errors
    ///
    /// Returns an error if the given host callback is not suitable for loading REAPER functions.
    ///
    /// [`new()`]: https://docs.rs/vst/0.2.0/vst/plugin/trait.Plugin.html#method.new
    pub fn from_vst_plugin(
        host_callback: HostCallbackProc,
    ) -> Result<ReaperPluginContext, ContextFromVstPluginError> {
        Ok(ReaperPluginContext {
            type_specific: TypeSpecificReaperPluginContext::Vst(ReaperVstPluginContext {
                host_callback,
            }),
            // TODO-medium
            h_instance: null_mut(),
        })
    }

    /// Returns a generic API function by its name.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    #[allow(overflowing_literals)]
    pub unsafe fn GetFunc(&self, name: *const c_char) -> *mut c_void {
        use TypeSpecificReaperPluginContext::*;
        match &self.type_specific {
            Extension(context) => (context.get_func)(name),
            Vst(context) => {
                // Invoke host callback
                (context.host_callback)(
                    null_mut(),
                    0xdead_beef,
                    0xdead_f00d,
                    0,
                    name as *mut c_void,
                    0.0,
                ) as *mut c_void
            }
        }
    }

    /// On Windows, this returns the `HINSTANCE` passed to `DllMain`.
    ///
    /// The returned `HINSTANCE` represents the handle of the module (DLL) containing the plug-in.
    ///
    /// On Linux, this returns `None`.
    pub fn h_instance(&self) -> raw::HINSTANCE {
        self.h_instance
    }

    /// On Linux, this returns a generic SWELL API function by its name.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetSwellFunc(
        &self,
        name: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_void {
        todo!()
    }
}

/// An error which can occur when attempting to create a REAPER plug-in context from an extension
/// plug-in.
#[derive(Debug, Clone, Eq, PartialEq, Display)]
pub enum ContextFromExtensionPluginError {
    /// `caller_version` doesn't match `REAPER_PLUGIN_VERSION`.
    #[display(fmt = "caller version incompatible")]
    CallerVersionIncompatible,
    /// `GetFunc` pointer is not set.
    #[display(fmt = "function provider not available")]
    FunctionProviderNotAvailable,
}

impl std::error::Error for ContextFromExtensionPluginError {}

/// An error which can occur when attempting to create a REAPER plug-in context from a VST plug-in.
#[derive(Debug, Clone, Eq, PartialEq, Display)]
pub enum ContextFromVstPluginError {}

impl std::error::Error for ContextFromVstPluginError {}
