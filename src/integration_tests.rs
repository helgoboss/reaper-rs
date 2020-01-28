use std::error::Error;
use crate::high_level::{Reaper, ActionKind, toggleable};
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
        let low = low_level::Reaper::with_all_functions_loaded(
            &low_level::create_reaper_plugin_function_provider(GetFunc)
        );
        let medium = medium_level::Reaper::new(low);
        medium.show_console_msg(c_str!("Loaded reaper-rs integration test plugin"));
        let high = Reaper::new(medium);
        let mut i = 0;
        let action1 = high.register_action(
            c_str!("reaperRsCounter"),
            c_str!("reaper-rs counter"),
            move || {
                let owned = format!("Hello from Rust number {}\0", i);
                let reaper = Reaper::installed();
                reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
                i += 1;
            },
            ActionKind::NotToggleable,
        );
        let action2 = high.register_action(
            c_str!("reaperRsIntegrationTests"),
            c_str!("reaper-rs integration tests"),
            || { execute_tests(Reaper::installed()) },
            ActionKind::NotToggleable,
        );
        let action3 = high.register_action(
            c_str!("reaperRsExample"),
            c_str!("reaper-rs example"),
            || { example_code(Reaper::installed()); },
            ActionKind::NotToggleable,
        );
        Reaper::install(high);
        1
    } else {
        0
    }
}

fn example_code(reaper: &Reaper) -> Result<(), Box<dyn Error>> {
    let project = reaper.get_current_project();
    let projects = reaper.get_projects();
    projects.for_each(|p| {
        let owned = format!("Project {} at index {}\0", p.get_file_path().unwrap_or("<None>".to_owned()), p.get_index());
        reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
    });

//    let track = project.get_first_track().ok_or("No first track")?;
//    let track_name = track.get_name();
//    let owned = format!("Track name is {}\0", track_name);
//    reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
    Ok(())
}

fn execute_tests(reaper: &Reaper) {
    create_empty_project_in_new_tab(reaper);
}

fn create_empty_project_in_new_tab(reaper: &Reaper) -> Result<(), Box<dyn Error>> {
    // Given
    let current_project_before = reaper.get_current_project();
//    let project_count_before = reaper.get_project_count();
    // Then
    Ok(())
}