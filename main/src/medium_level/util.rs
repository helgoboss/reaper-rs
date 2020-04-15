use super::{ControlSurface, DelegatingControlSurface};
use crate::medium_level::ReaperVersion;

/// The medium-level variant of
/// [`low_level::install_control_surface`](../../low_level/fn.install_control_surface.html).
pub fn install_control_surface(
    control_surface: impl ControlSurface + 'static,
    reaper_version: &ReaperVersion,
) {
    let delegating_control_surface = DelegatingControlSurface::new(control_surface, reaper_version);
    crate::low_level::install_control_surface(delegating_control_surface);
}
