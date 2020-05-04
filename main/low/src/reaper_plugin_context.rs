use super::raw::{reaper_plugin_info_t, REAPER_PLUGIN_VERSION};
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use vst::api::HostCallbackProc;
use vst::plugin::HostCallback;

type FunctionProvider = Box<dyn Fn(&std::ffi::CStr) -> isize>;

/// This represents the context which is needed to access REAPER functions from plug-ins.
///
/// Once obtained, it is supposed to be passed to [`Reaper::load()`].
///
/// [`Reaper::load()`]: struct.Reaper.html#method.load
pub struct ReaperPluginContext {
    /// Function which obtains a function pointer for a given REAPER function name.
    pub(crate) function_provider: FunctionProvider,
}

impl ReaperPluginContext {
    /// Creates a plug-in context from an extension entry point plug-in info.
    ///
    /// It requires the [`reaper_plugin_info_t`] struct that REAPER provides when calling the
    /// `ReaperPluginEntry` function (the main entry point for any extension plug-in).
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
    /// [`new()`]: /vst/plugin/trait.Plugin.html#method.new
    pub fn from_vst_plugin(host: HostCallback) -> Result<ReaperPluginContext, &'static str> {
        let host_callback = host.raw_callback().ok_or("Host callback not available")?;
        let function_provider = create_vst_plugin_function_provider(host_callback);
        Ok(ReaperPluginContext { function_provider })
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
            0xdeadbeef,
            0xdeadf00d,
            0,
            name.as_ptr() as *mut c_void,
            0.0,
        )
    })
}
