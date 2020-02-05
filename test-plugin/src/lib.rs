use std::error::Error;
use reaper_rs::high_level::{Reaper, ActionKind, toggleable, Project, create_reaper_panic_hook, create_default_console_msg_formatter, create_reaper_console_logger};
use std::os::raw::{c_int, c_char};
use reaper_rs::medium_level;
use reaper_rs::low_level;
use reaper_rs::high_level;
use c_str_macro::c_str;
use std::ffi::{CString, CStr};
use std::borrow::BorrowMut;
use rxrust::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::panic;
use reaper_rs::low_level::firewall;
use slog::{o, Drain, OwnedKVList, Record};
use std::io::LineWriter;


// TODO Integrate some of this into main
#[no_mangle]
extern "C" fn ReaperPluginEntry(h_instance: low_level::HINSTANCE, rec: *mut low_level::reaper_plugin_info_t) -> c_int {
    firewall(|| {
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
            // Medium-level
            let medium_level_reaper = medium_level::Reaper::new(low_level_reaper);
            // High-level
            high_level::Reaper::setup(medium_level_reaper);
            setup_logging();
            let reaper = Reaper::instance();
            reaper.show_console_msg(c_str!("Loaded reaper-rs integration test plugin\n"));
            reaper.activate();
            reaper.register_action(
                c_str!("reaperRsIntegrationTests"),
                c_str!("reaper-rs integration tests"),
                || {
                    reaper_rs_test::execute_integration_test();
                },
                ActionKind::NotToggleable,
            );
            1
        } else {
            0
        }
    }).unwrap_or(0)
}

fn setup_logging() {
    let logger = create_reaper_console_logger();
    panic::set_hook(create_reaper_panic_hook(
        logger,
        Some(create_default_console_msg_formatter("info@helgoboss.org")),
    ));
}