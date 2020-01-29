use std::error::Error;
use reaper_rs::high_level::{Reaper, ActionKind, toggleable, Project};
use std::os::raw::{c_int, c_char};
use reaper_rs::medium_level;
use reaper_rs::low_level;
use reaper_rs::high_level;
use c_str_macro::c_str;
use std::ffi::{CString, CStr};
use std::borrow::BorrowMut;
use rxrust::prelude::*;

struct MyControlSurface {}

impl medium_level::ControlSurface for MyControlSurface {
    fn run(&self) {
        println!("Hello from medium-level ControlSurface")
    }
}

#[no_mangle]
extern "C" fn ReaperPluginEntry(h_instance: low_level::HINSTANCE, rec: *mut low_level::reaper_plugin_info_t) -> c_int {
    if rec.is_null() {
        return 0;
    }
    let rec = unsafe { *rec };
    if rec.caller_version != low_level::REAPER_PLUGIN_VERSION as c_int {
        return 0;
    }
    if let Some(GetFunc) = rec.GetFunc {
        // Low-level
        let low_level_reaper = low_level::Reaper::with_all_functions_loaded(
            &low_level::create_reaper_plugin_function_provider(GetFunc)
        );
//        let cpp_surface = low_level_reaper.setup_control_surface(MyControlSurface {});
//        low_level_reaper.plugin_register.unwrap()(c_str!("csurf_inst").as_ptr(), cpp_surface);

        // Medium-level
        let medium_level_reaper = medium_level::Reaper::new(low_level_reaper);
        medium_level_reaper.register_control_surface(MyControlSurface {});

        // High-level
        high_level::Reaper::setup(medium_level_reaper);
//        let high_level_surface = high_level::ControlSurface::new();
        1
    } else {
        0
    }
}

fn use_high_level() {
    let high_level_reaper = Reaper::instance();
    high_level_reaper.show_console_msg(c_str!("Loaded reaper-rs integration test plugin\n"));
    let mut i = 0;
    let action1 = high_level_reaper.register_action(
        c_str!("reaperRsCounter"),
        c_str!("reaper-rs counter"),
        move || {
            let owned = format!("Hello from Rust number {}\0", i);
            let reaper = Reaper::instance();
            reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
            i += 1;
        },
        ActionKind::NotToggleable,
    );
    let action2 = high_level_reaper.register_action(
        c_str!("reaperRsIntegrationTests"),
        c_str!("reaper-rs integration tests"),
        || { execute_tests(Reaper::instance()) },
        ActionKind::NotToggleable,
    );
    let action3 = high_level_reaper.register_action(
        c_str!("reaperRsExample"),
        c_str!("reaper-rs example"),
        || { example_code(Reaper::instance()); },
        ActionKind::NotToggleable,
    );
}

fn example_code(reaper: &Reaper) -> Result<(), Box<dyn Error>> {
    let project = reaper.get_current_project();
    let projects = reaper.get_projects();
    projects.for_each(|p| {
        let owned = format!("Project {} at index {}\0", p.get_file_path().unwrap_or("<None>".to_owned()), p.get_index());
        reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
    });

    let track = project.get_first_track().ok_or("No first track")?;
    let track_name = track.get_name();
    let owned = format!("Track name is {}\0", track_name);
    reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
    Ok(())
}

fn execute_tests(reaper: &Reaper) {
    create_empty_project_in_new_tab(reaper);
}

fn create_empty_project_in_new_tab(reaper: &Reaper) -> Result<(), Box<dyn Error>> {
    // Given
    let current_project_before = reaper.get_current_project();
    let project_count_before = reaper.get_project_count();
    // When
//    Reaper::instance().project_switched().subscribe(|p: Project| {
//        println!("Project index {}", p.get_index());
//    });
    // Then
    Ok(())
}