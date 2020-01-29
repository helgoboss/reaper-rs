#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings;

pub use bindings::root::{
    ReaProject, MediaTrack, ACCEL, gaccel_register_t, HINSTANCE, REAPER_PLUGIN_VERSION,
    reaper_plugin_info_t
};
pub use bindings::root::reaper_rs_control_surface::create_control_surface;
pub use control_surface::IReaperControlSurface;

mod types;

mod control_surface;
use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::convert::AsRef;
use c_str_macro::c_str;
use std::ptr::null_mut;
use vst::api::HostCallbackProc;

pub fn create_reaper_plugin_function_provider(GetFunc: types::GetFunc) -> impl Fn(&CStr) -> isize {
    move |name| {
        unsafe { GetFunc(name.as_ptr()) as isize }
    }
}

pub fn create_reaper_vst_plugin_function_provider(host_callback: HostCallbackProc) -> impl Fn(&CStr) -> isize {
    move |name| {
        #[allow(overflowing_literals)]
            host_callback(null_mut(), 0xdeadbeef, 0xdeadf00d, 0, name.as_ptr() as *mut c_void, 0.0)
    }
}

macro_rules! gen_reaper_struct {
    ($($func:ident),+) => {
        #[derive(Default)]
        pub struct Reaper {
            $(
                pub $func: Option<types::$func>,
            )*
        }

        impl Reaper {
            pub fn with_all_functions_loaded(get_func: &impl Fn(&CStr) -> isize) -> Reaper {
                unsafe {
                    Reaper {
                        $(
                            $func: std::mem::transmute(get_func(c_str!(stringify!($func)))),
                        )*
                    }
                }
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
    plugin_register
];

#[macro_export]
macro_rules! customize_reaper_with_functions {
    ($($func:ident),+) => {
        impl $crate::low_level::Reaper {
            pub fn with_custom_functions_loaded(get_func: &impl Fn(&CStr) -> isize) -> $crate::low_level::Reaper {
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