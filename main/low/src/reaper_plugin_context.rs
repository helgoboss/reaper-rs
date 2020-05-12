#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::raw;
use derive_more::Display;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null_mut, NonNull};
use vst::api::{AEffect, HostCallbackProc};
use vst::plugin::HostCallback;

type GetFuncFn = unsafe extern "C" fn(name: *const c_char) -> *mut c_void;
type RegisterFn = unsafe extern "C" fn(name: *const c_char, infostruct: *mut c_void) -> c_int;
pub(crate) type GetSwellFuncFn = unsafe extern "C" fn(name: *const c_char) -> *mut c_void;

/// This represents the context which is needed to access REAPER functions from plug-ins.
///
/// Once obtained, it is supposed to be passed to [`Reaper::load()`].
///
/// [`Reaper::load()`]: struct.Reaper.html#method.load
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperPluginContext {
    type_specific: TypeSpecificReaperPluginContext,
    h_instance: raw::HINSTANCE,
    get_swell_func: Option<GetSwellFuncFn>,
}

/// Additional stuff available in the plug-in context specific to a certain plug-in type.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TypeSpecificReaperPluginContext {
    /// This is an extension plug-in.
    Extension(ReaperExtensionPluginContext),
    /// This is a VST plug-in.
    Vst(ReaperVstPluginContext),
}

/// Additional data available in the context of extension plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperExtensionPluginContext {
    caller_version: c_int,
    hwnd_main: NonNull<raw::HWND__>,
    register: RegisterFn,
    get_func: GetFuncFn,
}

/// Additional data available in the context of VST plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ReaperVstPluginContext {
    host_callback: HostCallbackProc,
}

impl ReaperPluginContext {
    /// Creates a plug-in context from an extension entry point plug-in info.
    ///
    /// It requires a module handle and the pointer to a [`reaper_plugin_info_t`] struct. REAPER
    /// provides both when calling the `ReaperPluginEntry` function (the main entry point for
    /// any extension plug-in).
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
        static_context: StaticReaperExtensionPluginContext,
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
            get_swell_func: static_context.get_swell_func,
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
        host: &HostCallback,
        static_context: StaticReaperVstPluginContext,
    ) -> Result<ReaperPluginContext, ContextFromVstPluginError> {
        use ContextFromVstPluginError::*;
        let host_callback = host.raw_callback().ok_or(HostCallbackNotAvailable)?;
        Ok(ReaperPluginContext {
            type_specific: TypeSpecificReaperPluginContext::Vst(ReaperVstPluginContext {
                host_callback,
            }),
            h_instance: static_context.h_instance,
            get_swell_func: static_context.get_swell_func,
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

    /// On Windows, this returns the `HINSTANCE` passed to `DllMain` (VST plug-ins) or
    /// `ReaperPluginEntry` (extension plug-ins).
    ///
    /// The returned `HINSTANCE` represents the handle of the module (DLL) containing the plug-in.
    ///
    /// On Linux, this returns `null`.
    pub fn h_instance(&self) -> raw::HINSTANCE {
        self.h_instance
    }

    /// On Linux, this returns a generic SWELL API function by its name.
    ///
    /// On Windows, this just returns `null`.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetSwellFunc(
        &self,
        name: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_void {
        let get_swell_func = match self.get_swell_func.as_ref() {
            None => return null_mut(),
            Some(f) => f,
        };
        get_swell_func(name)
    }

    /// Returns the type-specific plug-in context.
    pub fn type_specific(&self) -> &TypeSpecificReaperPluginContext {
        &self.type_specific
    }
}

impl ReaperExtensionPluginContext {
    /// Returns the caller version from `reaper_plugin_info_t`.
    pub fn caller_version(&self) -> c_int {
        self.caller_version
    }

    /// Returns the main window from `reaper_plugin_info_t`.
    pub fn hwnd_main(&self) -> NonNull<raw::HWND__> {
        self.hwnd_main
    }

    /// This is the same like [`Reaper::plugin_register()`].
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`Reaper::plugin_register()`]: struct.Reaper.html#method.plugin_register
    pub unsafe fn Register(
        &self,
        name: *const ::std::os::raw::c_char,
        infostruct: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        (self.register)(name, infostruct)
    }
}

impl ReaperVstPluginContext {
    /// Generic host callback function for communicating with REAPER from the VST plug-in.
    ///
    /// This is just a pass-through to the VST host callback.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn host_callback(
        self,
        effect: *mut AEffect,
        opcode: i32,
        index: i32,
        value: isize,
        ptr: *mut c_void,
        opt: f32,
    ) -> isize {
        (self.host_callback)(effect, opcode, index, value, ptr, opt)
    }
}

/// An error which can occur when attempting to create a REAPER plug-in context from an extension
/// plug-in.
#[derive(Clone, Eq, PartialEq, Debug, Display)]
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
#[derive(Clone, Eq, PartialEq, Debug, Display)]
pub enum ContextFromVstPluginError {
    #[display(fmt = "host callback not available")]
    HostCallbackNotAvailable,
}

impl std::error::Error for ContextFromVstPluginError {}

/// Contains those parts of the REAPER VST plug-in context which must be obtained in static scope.
///
/// An instance of this struct is provided by the function returned by the function
/// `reaper_vst_plugin::static_context()` which is generated by the [`reaper_vst_plugin`]
/// macro.
///
/// [`reaper_vst_plugin`]: macro.reaper_vst_plugin.html
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StaticReaperVstPluginContext {
    /// The `HINSTANCE` representing the handle of the module (DLL) containing the plug-in.
    ///
    /// Windows only.
    pub h_instance: raw::HINSTANCE,
    /// Function which returns SWELL function by its name.
    ///
    /// Linux only.
    pub get_swell_func: Option<GetSwellFuncFn>,
}

impl Default for StaticReaperVstPluginContext {
    fn default() -> Self {
        StaticReaperVstPluginContext {
            h_instance: null_mut(),
            get_swell_func: None,
        }
    }
}

/// Contains those parts of the REAPER extension plug-in context which must be obtained in static
/// scope.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StaticReaperExtensionPluginContext {
    /// Function which returns SWELL function by its name.
    ///
    /// Linux only.
    pub get_swell_func: Option<GetSwellFuncFn>,
}

impl Default for StaticReaperExtensionPluginContext {
    fn default() -> Self {
        StaticReaperExtensionPluginContext {
            get_swell_func: None,
        }
    }
}
