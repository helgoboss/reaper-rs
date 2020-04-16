use super::{ControlSurface, DelegatingControlSurface};
use crate::ReaperVersion;

/// The medium-level variant of
/// [`reaper_rs_low::install_control_surface`](../../low_level/fn.install_control_surface.html).
pub fn install_control_surface(
    control_surface: impl ControlSurface + 'static,
    reaper_version: &ReaperVersion,
) {
    let delegating_control_surface = DelegatingControlSurface::new(control_surface, reaper_version);
    reaper_rs_low::install_control_surface(delegating_control_surface);
}
