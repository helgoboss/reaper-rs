use super::{Reaper, ControlSurface};
use std::sync::Once;
use std::os::raw::c_void;
use super::bindings::root::reaper_rs_control_surface::get_control_surface;

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut CONTROL_SURFACE_INSTANCE: Option<Box<dyn ControlSurface>> = None;
static INIT_CONTROL_SURFACE_INSTANCE: Once = Once::new();

// This returns a mutable reference. In general this mutability should not be used, just in case
// of control surface methods where it's sure that REAPER never reenters them! See
// ControlSurface doc.
pub fn get_control_surface_instance() -> &'static mut Box<dyn ControlSurface> {
    unsafe { CONTROL_SURFACE_INSTANCE.as_mut().unwrap() }
}


impl Reaper {
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
        // TODO-low Ensure that only called if there's not a control surface registered already
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