use super::{ControlSurface, DelegatingControlSurface};

/// The medium-level variant of
/// [`low_level::install_control_surface`](../../low_level/fn.install_control_surface.html).
pub fn install_control_surface(control_surface: impl ControlSurface + 'static) {
    let delegating_control_surface = DelegatingControlSurface::new(control_surface);
    crate::low_level::install_control_surface(delegating_control_surface);
}
