use std::error::Error;
use crate::high_level::{Reaper, ActionKind};
use std::os::raw::{c_int, c_char};
use crate::{bindings, high_level};
use crate::medium_level;
use crate::low_level;
use c_str_macro::c_str;
use std::ffi::{CString, CStr};
use crate::customize_reaper_with_functions;
use std::borrow::BorrowMut;

#[no_mangle]
extern "C" fn ReaperPluginEntry(h_instance: bindings::HINSTANCE, rec: *mut bindings::reaper_plugin_info_t) -> c_int {
    if rec.is_null() {
        return 0;
    }
    let rec = unsafe { *rec };
    if rec.caller_version != bindings::REAPER_PLUGIN_VERSION as c_int {
        return 0;
    }
    if let Some(GetFunc) = rec.GetFunc {
        let low = low_level::Reaper::with_all_functions_loaded(&low_level::create_reaper_plugin_function_provider(GetFunc));
        let medium = medium_level::Reaper::new(low);
        medium.show_console_msg(c_str!("Loaded reaper-rs integration test plugin"));
        let high = high_level::Reaper::new(medium);
        let mut i = 0;
        high.register_action(
            c_str!("reaperRsIntegrationTests"),
            c_str!("reaper-rs integration tests"),
            move || {
                let owned = format!("Hello from Rust number {}\0", i);
                high_level::Reaper::with_installed(|reaper| {
                    reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
                });
                i += 1;
            },
            ActionKind::NotToggleable,
        );
        high_level::Reaper::install(high);
        1
    } else {
        0
    }
}

fn execute_tests() {
//    create_empty_project_in_new_tab(reaper);
}

fn create_empty_project_in_new_tab(reaper: &high_level::Reaper) -> Result<(), Box<dyn Error>> {
    // Given
    let project = reaper.get_current_project();
    let track = project.get_first_track().ok_or("No first track")?;
    // When
    let track_name = track.get_name();
    // Then
    Ok(())
}