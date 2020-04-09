#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings;
mod util;

pub use bindings::root::reaper_rs_midi::*;
pub use bindings::root::{
    audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, reaper_plugin_info_t,
    GetActiveWindow, IReaperControlSurface, KbdCmd, KbdSectionInfo, MIDI_event_t, MediaTrack,
    ReaProject, TrackEnvelope, ACCEL, CSURF_EXT_SETBPMANDPLAYRATE, CSURF_EXT_SETFOCUSEDFX,
    CSURF_EXT_SETFXCHANGE, CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN, CSURF_EXT_SETFXPARAM,
    CSURF_EXT_SETFXPARAM_RECFX, CSURF_EXT_SETINPUTMONITOR, CSURF_EXT_SETLASTTOUCHEDFX,
    CSURF_EXT_SETSENDPAN, CSURF_EXT_SETSENDVOLUME, GUID, HINSTANCE, HWND, REAPER_PLUGIN_VERSION,
};
pub use control_surface::ControlSurface;
pub use util::firewall;

mod control_surface;
mod reaper;
pub use reaper::*;
mod reaper_impl;
pub use reaper_impl::*;

use c_str_macro::c_str;

use std::error::Error;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null_mut;
use vst::api::HostCallbackProc;
use vst::plugin::HostCallback;

pub fn get_reaper_plugin_function_provider(
    rec: *mut reaper_plugin_info_t,
) -> Result<FunctionProvider, &'static str> {
    if rec.is_null() {
        return Err("rec not available");
    }
    let rec = unsafe { *rec };
    if rec.caller_version != REAPER_PLUGIN_VERSION as c_int {
        return Err("Caller version doesn't match");
    }
    let GetFunc = rec.GetFunc.ok_or("GetFunc function pointer not set")?;
    Ok(create_reaper_plugin_function_provider(GetFunc))
}

pub fn create_reaper_plugin_function_provider(
    GetFunc: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
) -> FunctionProvider {
    Box::new(move |name| unsafe { GetFunc(name.as_ptr()) as isize })
}

pub fn create_reaper_vst_plugin_function_provider(
    host_callback: HostCallbackProc,
) -> FunctionProvider {
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

// TODO-low Log early errors
pub fn bootstrap_reaper_plugin(
    _h_instance: HINSTANCE,
    rec: *mut reaper_plugin_info_t,
    init: fn(ReaperPluginContext) -> Result<(), Box<dyn Error>>,
) -> i32 {
    firewall(|| {
        let context = match ReaperPluginContext::from_reaper_plugin(rec) {
            Err(_) => return 0,
            Ok(c) => c,
        };
        match init(context) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
    .unwrap_or(0)
}

pub struct ReaperPluginContext {
    pub function_provider: FunctionProvider,
}

impl ReaperPluginContext {
    pub fn from_reaper_plugin(
        rec: *mut reaper_plugin_info_t,
    ) -> Result<ReaperPluginContext, &'static str> {
        let function_provider = get_reaper_plugin_function_provider(rec)?;
        Ok(ReaperPluginContext { function_provider })
    }

    pub fn from_reaper_vst_plugin(host: HostCallback) -> Result<ReaperPluginContext, &'static str> {
        let host_callback = host.raw_callback().ok_or("Host callback not available")?;
        let function_provider = create_reaper_vst_plugin_function_provider(host_callback);
        Ok(ReaperPluginContext { function_provider })
    }
}
