#[macro_use]
mod api;
mod tests;

use reaper_rs::high_level::{Reaper, Project};
use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;
use std::ffi::CString;
use std::panic;
use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};
use crate::tests::create_test_steps;

pub fn execute_integration_test() {
    let reaper = Reaper::instance();
    reaper.clear_console();
    log("# Testing reaper-rs");
    let steps = create_test_steps();
    for s in steps {
        log_heading(s.name);
        let result = (s.operation)(reaper);
        match result {
            Ok(()) => log("\nSuccessful"),
            Err(msg) => {
                log_failure(msg);
                return;
            }
        }
    }
    log("\n\nIntegration test was successful")
}

fn log_failure(msg: &str) {
    log(format!("\nFailed: {}", msg));
}

fn log_heading(name: impl Into<Cow<'static, str>>) {
    log("\n\n## ");
    log(name);
}

fn log(msg: impl Into<Cow<'static, str>>) {
    let msg = match msg.into() {
        // We need to copy the string and append the 0 byte
        Borrowed(b) => CString::new(b),
        // We can move the string and append the 0 byte
        Owned(o) => CString::new(o),
    };
    Reaper::instance().show_console_msg(&msg.unwrap())
}