#[macro_use]
mod assert;
mod api;
mod mock;
mod tests;

use crate::api::{TestStep, TestStepContext, VersionRestriction};
use crate::tests::create_test_steps;
use reaper_high::{Reaper, ReaperSession};
use rxrust::prelude::*;

use std::collections::VecDeque;

use reaper_medium::ReaperStringArg;

use std::iter::FromIterator;
use std::ops::Deref;
use std::panic::AssertUnwindSafe;

/// Executes the complete integration test.
///
/// Calls the given callback as soon as finished (either when the first test step failed
/// or when all steps have executed successfully).
pub fn execute_integration_test(on_finish: impl Fn(Result<(), &str>) + 'static) {
    Reaper::get().clear_console();
    log("# Testing reaper-rs\n");
    let steps = VecDeque::from_iter(create_test_steps());
    let step_count = steps.len();
    execute_next_step(steps, step_count, on_finish);
}

fn execute_next_step(
    mut steps: VecDeque<TestStep>,
    step_count: usize,
    on_finish: impl Fn(Result<(), &str>) + 'static,
) {
    let step = match steps.pop_front() {
        Some(step) => step,
        None => {
            log("\n**Integration test was successful**\n\n");
            on_finish(Ok(()));
            return;
        }
    };
    log_step(step_count - steps.len() - 1, &step.name);
    let session = ReaperSession::get();
    if reaper_version_matches(&step) {
        let result = {
            let mut finished = LocalSubject::new();
            let context = TestStepContext {
                finished: finished.clone(),
            };
            let step_name = step.name.clone();
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                (step.operation)(session.deref(), context)
            }))
            .unwrap_or_else(|_| Err(format!("Test [{}] panicked", step_name).into()));
            finished.complete();
            result
        };
        match result {
            Ok(()) => {
                session
                    .execute_later_in_main_thread_asap(move || {
                        execute_next_step(steps, step_count, on_finish)
                    })
                    .expect("couldn't schedule next test step");
            }
            Err(msg) => {
                log_failure(&msg);
                on_finish(Err(&msg));
            }
        }
    } else {
        // REAPER version doesn't match
        let reason = match step.version_restriction {
            VersionRestriction::Min(_) => "REAPER version too low",
            VersionRestriction::Max(_) => "REAPER version too high",
            _ => unreachable!(),
        };
        log_skip(reason);
        session
            .execute_later_in_main_thread_asap(move || {
                execute_next_step(steps, step_count, on_finish)
            })
            .expect("couldn't schedule next test step");
    }
}

fn reaper_version_matches(step: &TestStep) -> bool {
    use VersionRestriction::*;
    match &step.version_restriction {
        AllVersions => true,
        Min(v) => Reaper::get().get_version() >= *v,
        Max(v) => Reaper::get().get_version() <= *v,
    }
}

fn log_skip(msg: &str) {
    log(format!("→ **SKIPPED** ({})", msg));
}

fn log_failure(msg: &str) {
    log(format!("→ **FAILED**\n\n{}", msg));
}

fn log_step(step_index: usize, name: &str) {
    log(format!("{}. {}\n", step_index + 1, name));
}

fn log<'a>(msg: impl Into<ReaperStringArg<'a>>) {
    Reaper::get().show_console_msg(msg)
}
