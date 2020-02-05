use std::error::Error;
use reaper_rs::high_level::{Reaper, ActionKind, toggleable, Project, create_reaper_panic_hook, create_default_console_msg_formatter, create_reaper_console_logger};
use std::os::raw::{c_int, c_char};
use reaper_rs::medium_level;
use reaper_rs::low_level;
use reaper_rs::high_level;
use reaper_rs::reaper_plugin_entry;
use c_str_macro::c_str;
use std::ffi::{CString, CStr};
use std::borrow::BorrowMut;
use rxrust::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::panic;
use reaper_rs::low_level::{firewall, reaper_plugin_info_t, REAPER_PLUGIN_VERSION, create_reaper_plugin_function_provider, get_reaper_plugin_function_provider, FunctionProvider, ReaperPluginContext};
use slog::{o, Drain, OwnedKVList, Record};
use std::io::LineWriter;

reaper_plugin_entry!(init);

fn init(context: ReaperPluginContext) -> Result<(), &'static str> {
    let low = low_level::Reaper::with_all_functions_loaded(context.function_provider);
    let medium = medium_level::Reaper::new(low);
    Reaper::setup(medium);
    // ---
    setup_logging();
    let reaper = Reaper::instance();
    reaper.activate();
    // ---
    reaper.show_console_msg(c_str!("Loaded reaper-rs integration test plugin\n"));
    reaper.register_action(
        c_str!("reaperRsIntegrationTests"),
        c_str!("reaper-rs integration tests"),
        || reaper_rs_test::execute_integration_test(),
        ActionKind::NotToggleable,
    );
    Ok(())
}

fn setup_logging() {
    let logger = create_reaper_console_logger();
    panic::set_hook(create_reaper_panic_hook(
        logger,
        Some(create_default_console_msg_formatter("info@helgoboss.org")),
    ));
}