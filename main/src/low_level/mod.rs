#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings;
mod util;

pub use bindings::root::{
    ReaProject, MediaTrack, ACCEL, gaccel_register_t, HINSTANCE, REAPER_PLUGIN_VERSION,
    reaper_plugin_info_t, KbdSectionInfo, HWND, GUID, TrackEnvelope,
};
use bindings::root::reaper_rs_control_surface::get_control_surface;
pub use control_surface::ControlSurface;
pub use util::firewall;

mod types;

mod control_surface;

use std::os::raw::{c_char, c_void, c_int};
use std::ffi::CStr;
use std::convert::AsRef;
use c_str_macro::c_str;
use std::ptr::null_mut;
use vst::api::HostCallbackProc;
use std::sync::Once;
use std::error::Error;

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut CONTROL_SURFACE_INSTANCE: Option<Box<dyn ControlSurface>> = None;
static INIT_CONTROL_SURFACE_INSTANCE: Once = Once::new();


// This returns a mutable reference. In general this mutability should not be used, just in case
// of control surface methods where it's sure that REAPER never reenters them! See
// ControlSurface doc.
pub(super) fn get_control_surface_instance() -> &'static mut Box<dyn ControlSurface> {
    unsafe {
        CONTROL_SURFACE_INSTANCE.as_mut().unwrap()
    }
}

pub fn get_reaper_plugin_function_provider(rec: *mut reaper_plugin_info_t) -> Result<FunctionProvider, &'static str> {
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

pub fn create_reaper_plugin_function_provider(GetFunc: types::GetFunc) -> FunctionProvider {
    Box::new(move |name| {
        unsafe { GetFunc(name.as_ptr()) as isize }
    })
}

pub fn create_reaper_vst_plugin_function_provider(host_callback: HostCallbackProc) -> FunctionProvider {
    Box::new(move |name| {
        #[allow(overflowing_literals)]
            host_callback(null_mut(), 0xdeadbeef, 0xdeadf00d, 0, name.as_ptr() as *mut c_void, 0.0)
    })
}

// TODO Log early errors
pub fn bootstrap_reaper_plugin(
    h_instance: HINSTANCE,
    rec: *mut reaper_plugin_info_t,
    init: fn(ReaperPluginContext) -> Result<(), Box<dyn Error>>,
) -> i32 {
    firewall(|| {
        let function_provider = match get_reaper_plugin_function_provider(rec) {
            Err(_) => return 0,
            Ok(p) => p
        };
        let context = ReaperPluginContext {
            function_provider
        };
        match init(context) {
            Ok(_) => 1,
            Err(_) => 0
        }
    }).unwrap_or(0)
}

pub struct ReaperPluginContext {
    pub function_provider: FunctionProvider,
}

pub type FunctionProvider = Box<dyn Fn(&CStr) -> isize>;

macro_rules! gen_reaper_struct {
    ($($func:ident),+) => {
        #[derive(Default)]
        pub struct Reaper {
            $(
                pub $func: Option<types::$func>,
            )*
        }

        impl Reaper {
            // This is provided in addition to the original API functions because we use another
            // loading mechanism, not the one in reaper_plugin_functions.h (in order to collect all
            // function pointers into struct fields instead of global variables, and in order to
            // still keep the possibility of loading only certain functions)
            pub fn with_all_functions_loaded(get_func: FunctionProvider) -> Reaper {
                unsafe {
                    Reaper {
                        $(
                            $func: std::mem::transmute(get_func(c_str!(stringify!($func)))),
                        )*
                    }
                }
            }

            // This is provided in addition to the original API functions because a pure
            // plugin_register("csurf_inst", my_rust_trait_implementing_IReaperControlSurface) isn't
            // going to cut it. Rust structs can't implement pure virtual C++ interfaces.
            // This function sets up the given ControlSurface implemented in Rust but doesn't yet register
            // it. Can be called only once.
            // Installed control surface is totally independent from this REAPER instance. So
            // destructor doesn't set the CONTROL_SURFACE_INSTANCE to None. The user needs to take
            // care of unregistering the control surface if he registered it before.
            // Once installed, it stays installed until this module unloaded
            pub fn install_control_surface(&self, control_surface: impl ControlSurface + 'static) {
                // TODO Ensure that only called if there's not a control surface registered already
                // Ideally we would have a generic static but as things are now, we need to box it.
                // However, this is not a big deal because control surfaces are only used in the
                // main thread where these minimal performance differences are not significant.
                unsafe {
                    // Save boxed control surface to static variable so that extern "C" functions implemented
                    // in Rust have something to delegate to.
                    INIT_CONTROL_SURFACE_INSTANCE.call_once(|| {
                        CONTROL_SURFACE_INSTANCE = Some(Box::new(control_surface));
                    });
                }
            }

            // It returns a pointer to a C++ object that will delegate to given Rust ControlSurface.
            // The pointer needs to be passed to plugin_register("csurf_inst", <here>) for registering or
            // plugin_register("-csurf_inst", <here>) for unregistering.
            pub fn get_cpp_control_surface(&self) -> *mut c_void {
                // Create and return C++ IReaperControlSurface implementations which calls extern "C"
                // functions implemented in RUst
                unsafe { get_control_surface() }
            }
        }
    }
}

gen_reaper_struct![
    EnumProjects,
    GetTrack,
    ShowConsoleMsg,
    ValidatePtr2,
    GetSetMediaTrackInfo,
    plugin_register,
    GetMainHwnd,
    KBD_OnMainActionEx,
    SectionFromUniqueID,
    NamedCommandLookup,
    ClearConsole,
    CountTracks,
    InsertTrackAtIndex,
    TrackList_UpdateAllExternalSurfaces,
    GetMediaTrackInfo_Value,
    GetAppVersion,
    GetTrackEnvelopeByName,
    GetTrackAutomationMode,
    GetGlobalAutomationOverride,
    TrackFX_GetCount,
    TrackFX_GetRecCount,
    TrackFX_GetFXGUID
];

#[macro_export]
macro_rules! customize_reaper_with_functions {
    ($($func:ident),+) => {
        impl $crate::low_level::Reaper {
            pub fn with_custom_functions_loaded(get_func: FunctionProvider) -> $crate::low_level::Reaper {
                unsafe {
                    $crate::low_level::Reaper {
                        $(
                            $func: std::mem::transmute(get_func(c_str!(stringify!($func)))),
                        )*
                        ..Default::default()
                    }
                }
            }
        }
    }
}