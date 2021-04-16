// This file is included in the generated bindings. It's not a separate module because this stuff
// must be part of the (generated) root module and I think splitting modules over multiple files
// isn't possible.

// # Start of manually written bindings.

// Type written manually because it needs a critical section on Windows and a mutex on Unix.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct preview_register_t {
    #[cfg(unix)]
    pub mutex: pthread_mutex_t,
    #[cfg(windows)]
    pub cs: winapi::um::minwinbase::CRITICAL_SECTION,
    pub src: *mut root::PCM_source,
    pub m_out_chan: ::std::os::raw::c_int,
    pub curpos: f64,
    pub loop_: bool,
    pub volume: f64,
    pub peakvol: [f64; 2usize],
    pub preview_track: *mut ::std::os::raw::c_void,
}
impl Default for preview_register_t {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
// # End of manually written bindings
