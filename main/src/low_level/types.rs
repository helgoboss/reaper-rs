//! Types copied from generated bindings
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::{ReaProject, MediaTrack};

pub type GetFunc = unsafe extern "C" fn(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void;

pub type EnumProjects = fn(
    idx: ::std::os::raw::c_int,
    projfnOutOptional: *mut ::std::os::raw::c_char,
    projfnOutOptional_sz: ::std::os::raw::c_int,
) -> *mut ReaProject;

pub type GetTrack = fn(
    proj: *mut ReaProject,
    trackidx: ::std::os::raw::c_int,
) -> *mut MediaTrack;

pub type ValidatePtr2 = fn(
    proj: *mut ReaProject,
    pointer: *mut ::std::os::raw::c_void,
    ctypename: *const ::std::os::raw::c_char,
) -> bool;

pub type GetSetMediaTrackInfo = fn(
    tr: *mut MediaTrack,
    parmname: *const ::std::os::raw::c_char,
    setNewValue: *mut ::std::os::raw::c_void,
) -> *mut ::std::os::raw::c_void;

pub type ShowConsoleMsg = fn(msg: *const ::std::os::raw::c_char);

pub type plugin_register = fn(
    name: *const ::std::os::raw::c_char,
    infostruct: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int;