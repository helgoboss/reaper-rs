use super::{ControlSurface, DelegatingControlSurface};

// Once installed, it stays installed until this module unloaded
pub fn install_control_surface(control_surface: impl ControlSurface + 'static) {
    let delegating_control_surface = DelegatingControlSurface::new(control_surface);
    crate::low_level::install_control_surface(delegating_control_surface);
}
