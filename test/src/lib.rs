#[macro_use]
mod assert;
mod api;
mod tests;
mod mock;

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
use std::iter::FromIterator;
use crate::api::{TestStep, TestStepContext};
use rxrust::prelude::*;

pub fn execute_integration_test() {
    let reaper = Reaper::instance();
    reaper.clear_console();
    log("# Testing reaper-rs");
    let mut steps = VecDeque::from_iter(create_test_steps());
    execute_next_step(reaper, steps);
}

fn execute_next_step(reaper: &'static Reaper, mut steps: VecDeque<TestStep>) {
    let step = match steps.pop_front() {
        Some(step) => step,
        None => {
            log("\n\nIntegration test was successful");
            return;
        }
    };
    log_heading(step.name);
    let result = {
        let mut finished = Subject::local();
        let context = TestStepContext {
            finished: finished.fork()
        };
        let result = (step.operation)(reaper, context);
        finished.complete();
        result
    };
    match result {
        Ok(()) => {
            log("\nSuccessful");
            reaper.execute_later_in_main_thread(move || execute_next_step(reaper, steps));
        },
        Err(msg) => log_failure(&msg)
    }
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