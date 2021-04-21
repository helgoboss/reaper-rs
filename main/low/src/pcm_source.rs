#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pcm_source::*;
use crate::{firewall, raw};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null, null_mut, NonNull};

impl raw::PCM_source {
    pub fn GetLength(&self) -> f64 {
        unsafe { PCM_source_GetLength(self as *const _ as _) }
    }

    pub fn IsAvailable(&self) -> bool {
        unsafe { PCM_source_IsAvailable(self as *const _ as _) }
    }

    pub fn Duplicate(&self) -> *mut raw::PCM_source {
        unsafe { PCM_source_Duplicate(self as *const _ as _) }
    }

    pub fn GetType(&self) -> *const c_char {
        unsafe { PCM_source_GetType(self as *const _ as _) }
    }

    pub fn GetFileName(&self) -> *const c_char {
        unsafe { PCM_source_GetFileName(self as *const _ as _) }
    }

    pub fn GetSource(&self) -> *mut raw::PCM_source {
        unsafe { PCM_source_GetSource(self as *const _ as _) }
    }

    pub fn Extended(
        &self,
        call: c_int,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> c_int {
        unsafe { PCM_source_Extended(self as *const _ as _, call, parm1, parm2, parm3) }
    }

    pub fn SetAvailable(&self, avail: bool) {
        unsafe {
            PCM_source_SetAvailable(self as *const _ as _, avail);
        }
    }

    pub fn SetFileName(&self, newfn: *const ::std::os::raw::c_char) -> bool {
        unsafe { PCM_source_SetFileName(self as *const _ as _, newfn) }
    }

    pub fn SetSource(&self, src: *mut raw::PCM_source) {
        unsafe {
            PCM_source_SetSource(self as *const _ as _, src);
        }
    }

    pub fn GetNumChannels(&self) -> ::std::os::raw::c_int {
        unsafe { PCM_source_GetNumChannels(self as *const _ as _) }
    }

    pub fn GetSampleRate(&self) -> f64 {
        unsafe { PCM_source_GetSampleRate(self as *const _ as _) }
    }

    pub fn GetLengthBeats(&self) -> f64 {
        unsafe { PCM_source_GetLengthBeats(self as *const _ as _) }
    }

    pub fn GetBitsPerSample(&self) -> ::std::os::raw::c_int {
        unsafe { PCM_source_GetBitsPerSample(self as *const _ as _) }
    }

    pub fn GetPreferredPosition(&self) -> f64 {
        unsafe { PCM_source_GetPreferredPosition(self as *const _ as _) }
    }

    pub fn PropertiesWindow(&self, hwndParent: raw::HWND) -> ::std::os::raw::c_int {
        unsafe { PCM_source_PropertiesWindow(self as *const _ as _, hwndParent) }
    }

    pub fn GetSamples(&self, block: *mut raw::PCM_source_transfer_t) {
        unsafe {
            PCM_source_GetSamples(self as *const _ as _, block);
        }
    }

    pub fn GetPeakInfo(&self, block: *mut raw::PCM_source_peaktransfer_t) {
        unsafe {
            PCM_source_GetPeakInfo(self as *const _ as _, block);
        }
    }

    pub fn SaveState(&self, ctx: *mut raw::ProjectStateContext) {
        unsafe {
            PCM_source_SaveState(self as *const _ as _, ctx);
        }
    }

    pub fn LoadState(
        &self,
        firstline: *const ::std::os::raw::c_char,
        ctx: *mut raw::ProjectStateContext,
    ) -> ::std::os::raw::c_int {
        unsafe { PCM_source_LoadState(self as *const _ as _, firstline, ctx) }
    }

    pub fn Peaks_Clear(&self, deleteFile: bool) {
        unsafe {
            PCM_source_Peaks_Clear(self as *const _ as _, deleteFile);
        }
    }

    pub fn PeaksBuild_Begin(&self) -> ::std::os::raw::c_int {
        unsafe { PCM_source_PeaksBuild_Begin(self as *const _ as _) }
    }

    pub fn PeaksBuild_Run(&self) -> ::std::os::raw::c_int {
        unsafe { PCM_source_PeaksBuild_Run(self as *const _ as _) }
    }

    pub fn PeaksBuild_Finish(&self) {
        unsafe {
            PCM_source_PeaksBuild_Finish(self as *const _ as _);
        }
    }
}

/// This is the Rust analog to the C++ virtual base class `PCM_source`.
///
/// An implementation of this trait can be passed to [`add_cpp_pcm_source()`].
///
/// [`add_cpp_pcm_source()`]: fn.add_cpp_pcm_source.html
pub trait PCM_source {
    fn Duplicate(&mut self) -> *mut raw::PCM_source;

    fn IsAvailable(&mut self) -> bool;
    fn SetAvailable(&mut self, avail: bool) {
        let _ = avail;
    }
    fn GetType(&mut self) -> *const ::std::os::raw::c_char;
    fn GetFileName(&mut self) -> *const ::std::os::raw::c_char {
        null()
    }
    fn SetFileName(&mut self, newfn: *const ::std::os::raw::c_char) -> bool;

    fn GetSource(&mut self) -> *mut raw::PCM_source {
        null_mut()
    }
    fn SetSource(&mut self, src: *mut raw::PCM_source) {
        let _ = src;
    }
    fn GetNumChannels(&mut self) -> ::std::os::raw::c_int;
    fn GetSampleRate(&mut self) -> f64;
    fn GetLength(&mut self) -> f64;
    fn GetLengthBeats(&mut self) -> f64 {
        -1.0
    }
    fn GetBitsPerSample(&mut self) -> ::std::os::raw::c_int {
        0
    }
    fn GetPreferredPosition(&mut self) -> f64 {
        -1.0
    }

    fn PropertiesWindow(&mut self, hwndParent: raw::HWND) -> ::std::os::raw::c_int;

    fn GetSamples(&mut self, block: *mut raw::PCM_source_transfer_t);
    fn GetPeakInfo(&mut self, block: *mut raw::PCM_source_peaktransfer_t);

    fn SaveState(&mut self, ctx: *mut raw::ProjectStateContext);
    fn LoadState(
        &mut self,
        firstline: *const ::std::os::raw::c_char,
        ctx: *mut raw::ProjectStateContext,
    ) -> ::std::os::raw::c_int;

    fn Peaks_Clear(&mut self, deleteFile: bool);
    fn PeaksBuild_Begin(&mut self) -> ::std::os::raw::c_int;
    fn PeaksBuild_Run(&mut self) -> ::std::os::raw::c_int;
    fn PeaksBuild_Finish(&mut self);

    fn Extended(
        &mut self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        let _ = call;
        let _ = parm1;
        let _ = parm2;
        let _ = parm3;
        0
    }
}

/// Creates a `PCM_source` object on C++ side and returns a pointer to it.
///
/// This function is provided because Rust structs can't implement C++ virtual base classes.
///
/// # Example
///
/// See [`add_cpp_control_surface()`]. Usage is very similar.
///
/// # Cleaning up
///
/// In order to avoid memory leaks, you must take care of removing the C++ counterpart
/// PCM source by calling [`remove_cpp_pcm_source()`].
///
/// # Safety
///
/// This function is highly unsafe. Better use the medium-level API instead.
///
/// [`remove_cpp_pcm_source()`]: fn.remove_cpp_pcm_source.html
/// [`add_cpp_control_surface()`]: fn.add_cpp_control_surface.html
pub unsafe fn add_cpp_pcm_source(
    callback_target: NonNull<Box<dyn PCM_source>>,
) -> NonNull<raw::PCM_source> {
    let instance = crate::bindings::root::reaper_pcm_source::add_pcm_source(
        callback_target.as_ptr() as *mut c_void,
    );
    NonNull::new_unchecked(instance)
}

/// Destroys a C++ `PCM_source` object.
///
/// Intended to be used on pointers returned from [`add_cpp_pcm_source()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`add_cpp_pcm_source()`]: fn.add_cpp_pcm_source.html
pub unsafe fn remove_cpp_pcm_source(source: NonNull<raw::PCM_source>) {
    crate::bindings::root::reaper_pcm_source::remove_pcm_source(source.as_ptr());
}

#[no_mangle]
extern "C" fn GetLength(callback_target: *mut Box<dyn PCM_source>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetLength()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn Duplicate(callback_target: *mut Box<dyn PCM_source>) -> *mut raw::PCM_source {
    firewall(|| unsafe { &mut *callback_target }.Duplicate()).unwrap_or(null_mut())
}
#[no_mangle]
extern "C" fn GetType(callback_target: *mut Box<dyn PCM_source>) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &mut *callback_target }.GetType()).unwrap_or(null())
}
#[no_mangle]
extern "C" fn GetFileName(
    callback_target: *mut Box<dyn PCM_source>,
) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &mut *callback_target }.GetFileName()).unwrap_or(null())
}
#[no_mangle]
extern "C" fn GetSource(callback_target: *mut Box<dyn PCM_source>) -> *mut raw::PCM_source {
    firewall(|| unsafe { &mut *callback_target }.GetSource()).unwrap_or(null_mut())
}
#[no_mangle]
extern "C" fn IsAvailable(callback_target: *mut Box<dyn PCM_source>) -> bool {
    firewall(|| unsafe { &mut *callback_target }.IsAvailable()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn SetAvailable(callback_target: *mut Box<dyn PCM_source>, avail: bool) {
    firewall(|| unsafe { &mut *callback_target }.SetAvailable(avail));
}
#[no_mangle]
extern "C" fn SetFileName(
    callback_target: *mut Box<dyn PCM_source>,
    newfn: *const ::std::os::raw::c_char,
) -> bool {
    firewall(|| unsafe { &mut *callback_target }.SetFileName(newfn)).unwrap_or_default()
}
#[no_mangle]
extern "C" fn SetSource(callback_target: *mut Box<dyn PCM_source>, src: *mut raw::PCM_source) {
    firewall(|| unsafe { &mut *callback_target }.SetSource(src));
}
#[no_mangle]
extern "C" fn GetNumChannels(callback_target: *mut Box<dyn PCM_source>) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetNumChannels()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn GetSampleRate(callback_target: *mut Box<dyn PCM_source>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetSampleRate()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn GetLengthBeats(callback_target: *mut Box<dyn PCM_source>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetLengthBeats()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn GetBitsPerSample(callback_target: *mut Box<dyn PCM_source>) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetBitsPerSample()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn GetPreferredPosition(callback_target: *mut Box<dyn PCM_source>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetPreferredPosition()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn PropertiesWindow(
    callback_target: *mut Box<dyn PCM_source>,
    hwndParent: raw::HWND,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.PropertiesWindow(hwndParent)).unwrap_or_default()
}
#[no_mangle]
extern "C" fn GetSamples(
    callback_target: *mut Box<dyn PCM_source>,
    block: *mut raw::PCM_source_transfer_t,
) {
    firewall(|| unsafe { &mut *callback_target }.GetSamples(block));
}
#[no_mangle]
extern "C" fn GetPeakInfo(
    callback_target: *mut Box<dyn PCM_source>,
    block: *mut raw::PCM_source_peaktransfer_t,
) {
    firewall(|| unsafe { &mut *callback_target }.GetPeakInfo(block));
}
#[no_mangle]
extern "C" fn SaveState(
    callback_target: *mut Box<dyn PCM_source>,
    ctx: *mut raw::ProjectStateContext,
) {
    firewall(|| unsafe { &mut *callback_target }.SaveState(ctx));
}
#[no_mangle]
extern "C" fn LoadState(
    callback_target: *mut Box<dyn PCM_source>,
    firstline: *const ::std::os::raw::c_char,
    ctx: *mut raw::ProjectStateContext,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.LoadState(firstline, ctx)).unwrap_or_default()
}
#[no_mangle]
extern "C" fn Peaks_Clear(callback_target: *mut Box<dyn PCM_source>, deleteFile: bool) {
    firewall(|| unsafe { &mut *callback_target }.Peaks_Clear(deleteFile));
}
#[no_mangle]
extern "C" fn PeaksBuild_Begin(callback_target: *mut Box<dyn PCM_source>) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.PeaksBuild_Begin()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn PeaksBuild_Run(callback_target: *mut Box<dyn PCM_source>) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.PeaksBuild_Run()).unwrap_or_default()
}
#[no_mangle]
extern "C" fn PeaksBuild_Finish(callback_target: *mut Box<dyn PCM_source>) {
    firewall(|| unsafe { &mut *callback_target }.PeaksBuild_Finish());
}
// TODO-low Here we have a name conflict with the Extended function from IReaperControlSurface.
//  This indicates that we should use prefixes (since namespaces are not supported in C).
//  The prefix should express the type (e.g. PCM_source) and the direction (C to Rust and back)
//  because for one method we have 2 functions, one for each direction.
#[no_mangle]
extern "C" fn rust_PCM_source_Extended(
    callback_target: *mut Box<dyn PCM_source>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.Extended(call, parm1, parm2, parm3))
        .unwrap_or_default()
}
