#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::types;
use crate::bindings::{ReaProject, MediaTrack};
use std::os::raw::c_char;
use std::ffi::CStr;
use std::convert::AsRef;
use c_str_macro::c_str;

pub fn create_reaper_plugin_function_provider(GetFunc: types::GetFunc) -> impl Fn(&CStr) -> isize {
    move |name| {
        unsafe { GetFunc(name.as_ptr()) as isize }
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