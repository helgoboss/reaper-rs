use super::{DelegatingControlSurface, ReaperControlSurface};
use crate::ReaperVersion;
use std::ffi::{CStr, CString};

/// The medium-level variant of
/// [`reaper_rs_low::install_control_surface`](../../low_level/fn.install_control_surface.html).
pub fn install_control_surface(
    control_surface: impl ReaperControlSurface + 'static,
    reaper_version: &ReaperVersion,
) {
    let delegating_control_surface = DelegatingControlSurface::new(control_surface, reaper_version);
    reaper_rs_low::install_control_surface(delegating_control_surface);
}

pub(crate) fn concat_c_strs(first: &CStr, second: &CStr) -> CString {
    CString::new([first.to_bytes(), second.to_bytes()].concat()).unwrap()
}
