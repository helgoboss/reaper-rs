#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::bindings::root::reaper_pcm_sink::*;
use crate::{firewall, raw};
use std::os::raw::c_void;
use std::ptr::{null, NonNull};

impl raw::PCM_sink {
    pub fn GetOutputInfoString(
        &self,
        buf: *mut ::std::os::raw::c_char,
        buflen: ::std::os::raw::c_int,
    ) {
        unsafe {
            rust_to_cpp_PCM_sink_GetOutputInfoString(self as *const _ as _, buf, buflen);
        }
    }
    pub fn GetStartTime(&self) -> f64 {
        unsafe { rust_to_cpp_PCM_sink_GetStartTime(self as *const _ as _) }
    }
    pub fn SetStartTime(&self, st: f64) {
        unsafe {
            rust_to_cpp_PCM_sink_SetStartTime(self as *const _ as _, st);
        }
    }
    pub fn GetFileName(&self) -> *const ::std::os::raw::c_char {
        unsafe { rust_to_cpp_PCM_sink_GetFileName(self as *const _ as _) }
    }
    pub fn GetNumChannels(&self) -> ::std::os::raw::c_int {
        unsafe { rust_to_cpp_PCM_sink_GetNumChannels(self as *const _ as _) }
    }
    pub fn GetLength(&self) -> f64 {
        unsafe { rust_to_cpp_PCM_sink_GetLength(self as *const _ as _) }
    }
    pub fn GetFileSize(&self) -> ::std::os::raw::c_longlong {
        unsafe { rust_to_cpp_PCM_sink_GetFileSize(self as *const _ as _) }
    }
    pub fn WriteMIDI(
        &self,
        events: *mut raw::MIDI_eventlist,
        len: ::std::os::raw::c_int,
        samplerate: f64,
    ) {
        unsafe {
            rust_to_cpp_PCM_sink_WriteMIDI(self as *const _ as _, events, len, samplerate);
        }
    }
    pub fn WriteDoubles(
        &self,
        samples: *mut *mut raw::ReaSample,
        len: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
        offset: ::std::os::raw::c_int,
        spacing: ::std::os::raw::c_int,
    ) {
        unsafe {
            rust_to_cpp_PCM_sink_WriteDoubles(
                self as *const _ as _,
                samples,
                len,
                nch,
                offset,
                spacing,
            );
        }
    }
    pub fn WantMIDI(&self) -> bool {
        unsafe { rust_to_cpp_PCM_sink_WantMIDI(self as *const _ as _) }
    }
    pub fn GetLastSecondPeaks(
        &self,
        sz: ::std::os::raw::c_int,
        buf: *mut raw::ReaSample,
    ) -> ::std::os::raw::c_int {
        unsafe { rust_to_cpp_PCM_sink_GetLastSecondPeaks(self as *const _ as _, sz, buf) }
    }
    pub fn GetPeakInfo(&self, block: *mut raw::PCM_source_peaktransfer_t) {
        unsafe {
            rust_to_cpp_PCM_sink_GetPeakInfo(self as *const _ as _, block);
        }
    }
    pub fn Extended(
        &self,
        call: ::std::os::raw::c_int,
        parm1: *mut ::std::os::raw::c_void,
        parm2: *mut ::std::os::raw::c_void,
        parm3: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        unsafe { rust_to_cpp_PCM_sink_Extended(self as *const _ as _, call, parm1, parm2, parm3) }
    }
}

/// This is the Rust analog to the C++ virtual base class `PCM_sink`.
///
/// An implementation of this trait can be passed to [`create_cpp_to_rust_pcm_sink()`].
///
/// [`create_cpp_to_rust_pcm_sink()`]: fn.create_cpp_to_rust_pcm_sink.html
pub trait PCM_sink {
    fn GetOutputInfoString(
        &mut self,
        buf: *mut ::std::os::raw::c_char,
        buflen: ::std::os::raw::c_int,
    );
    fn GetStartTime(&mut self) -> f64;
    fn SetStartTime(&mut self, st: f64);
    fn GetFileName(&mut self) -> *const ::std::os::raw::c_char;
    fn GetNumChannels(&mut self) -> ::std::os::raw::c_int;
    fn GetLength(&mut self) -> f64;
    fn GetFileSize(&mut self) -> ::std::os::raw::c_longlong;
    fn WriteMIDI(
        &mut self,
        events: *mut raw::MIDI_eventlist,
        len: ::std::os::raw::c_int,
        samplerate: f64,
    );
    fn WriteDoubles(
        &mut self,
        samples: *mut *mut raw::ReaSample,
        len: ::std::os::raw::c_int,
        nch: ::std::os::raw::c_int,
        offset: ::std::os::raw::c_int,
        spacing: ::std::os::raw::c_int,
    );
    fn WantMIDI(&mut self) -> bool {
        false
    }
    fn GetLastSecondPeaks(
        &mut self,
        sz: ::std::os::raw::c_int,
        buf: *mut raw::ReaSample,
    ) -> ::std::os::raw::c_int {
        let _ = sz;
        let _ = buf;
        0
    }
    fn GetPeakInfo(&mut self, block: *mut raw::PCM_source_peaktransfer_t) {
        let _ = block;
    }

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

/// Creates a `PCM_sink` object on C++ side and returns a pointer to it.
///
/// This function is provided because Rust structs can't implement C++ virtual base classes.
///
/// # Example
///
/// See [`create_cpp_to_rust_control_surface()`]. Usage is very similar.
///
/// # Cleaning up
///
/// In order to avoid memory leaks, you must take care of removing the C++ counterpart
/// PCM sink by calling [`delete_cpp_pcm_sink()`].
///
/// # Safety
///
/// This function is highly unsafe. Better use the medium-level API instead.
///
/// [`delete_cpp_pcm_sink()`]: fn.remove_cpp_pcm_sink.html
/// [`create_cpp_to_rust_control_surface()`]: fn.create_cpp_to_rust_control_surface.html
pub unsafe fn create_cpp_to_rust_pcm_sink(
    callback_target: NonNull<Box<dyn PCM_sink>>,
) -> NonNull<raw::PCM_sink> {
    let instance = crate::bindings::root::reaper_pcm_sink::create_cpp_to_rust_pcm_sink(
        callback_target.as_ptr() as *mut c_void,
    );
    NonNull::new_unchecked(instance)
}

/// Destroys a C++ `PCM_sink` object.
///
/// Intended to be used on pointers returned from [`create_cpp_to_rust_pcm_sink()`].
///
/// # Safety
///
/// REAPER can crash if you pass an invalid pointer because C++ will attempt to free the wrong
/// location in memory.
///
/// [`create_cpp_to_rust_pcm_sink()`]: fn.create_cpp_to_rust_pcm_sink.html
pub unsafe fn delete_cpp_pcm_sink(sink: NonNull<raw::PCM_sink>) {
    crate::bindings::root::reaper_pcm_sink::delete_pcm_sink(sink.as_ptr());
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetOutputInfoString(
    callback_target: *mut Box<dyn PCM_sink>,
    buf: *mut ::std::os::raw::c_char,
    buflen: ::std::os::raw::c_int,
) {
    firewall(|| unsafe { &mut *callback_target }.GetOutputInfoString(buf, buflen));
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetStartTime(callback_target: *mut Box<dyn PCM_sink>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetStartTime()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_SetStartTime(callback_target: *mut Box<dyn PCM_sink>, st: f64) {
    firewall(|| unsafe { &mut *callback_target }.SetStartTime(st));
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetFileName(
    callback_target: *mut Box<dyn PCM_sink>,
) -> *const ::std::os::raw::c_char {
    firewall(|| unsafe { &mut *callback_target }.GetFileName()).unwrap_or(null())
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetNumChannels(
    callback_target: *mut Box<dyn PCM_sink>,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetNumChannels()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetLength(callback_target: *mut Box<dyn PCM_sink>) -> f64 {
    firewall(|| unsafe { &mut *callback_target }.GetLength()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetFileSize(
    callback_target: *mut Box<dyn PCM_sink>,
) -> ::std::os::raw::c_longlong {
    firewall(|| unsafe { &mut *callback_target }.GetFileSize()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_WriteMIDI(
    callback_target: *mut Box<dyn PCM_sink>,
    events: *mut raw::MIDI_eventlist,
    len: ::std::os::raw::c_int,
    samplerate: f64,
) {
    firewall(|| unsafe { &mut *callback_target }.WriteMIDI(events, len, samplerate));
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_WriteDoubles(
    callback_target: *mut Box<dyn PCM_sink>,
    samples: *mut *mut raw::ReaSample,
    len: ::std::os::raw::c_int,
    nch: ::std::os::raw::c_int,
    offset: ::std::os::raw::c_int,
    spacing: ::std::os::raw::c_int,
) {
    firewall(|| unsafe { &mut *callback_target }.WriteDoubles(samples, len, nch, offset, spacing));
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_WantMIDI(callback_target: *mut Box<dyn PCM_sink>) -> bool {
    firewall(|| unsafe { &mut *callback_target }.WantMIDI()).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetLastSecondPeaks(
    callback_target: *mut Box<dyn PCM_sink>,
    sz: ::std::os::raw::c_int,
    buf: *mut raw::ReaSample,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.GetLastSecondPeaks(sz, buf)).unwrap_or_default()
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_GetPeakInfo(
    callback_target: *mut Box<dyn PCM_sink>,
    block: *mut raw::PCM_source_peaktransfer_t,
) {
    firewall(|| unsafe { &mut *callback_target }.GetPeakInfo(block));
}

#[no_mangle]
extern "C" fn cpp_to_rust_PCM_sink_Extended(
    callback_target: *mut Box<dyn PCM_sink>,
    call: ::std::os::raw::c_int,
    parm1: *mut ::std::os::raw::c_void,
    parm2: *mut ::std::os::raw::c_void,
    parm3: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    firewall(|| unsafe { &mut *callback_target }.Extended(call, parm1, parm2, parm3))
        .unwrap_or_default()
}
