#[macro_use]
mod assert;
mod api;
mod invocation_mock;
mod tests;

use crate::api::{Test, TestStep, TestStepContext, VersionRestriction};
use crate::tests::create_test_steps;
use reaper_high::{
    ChangeDetector, ControlSurfaceEvent, ControlSurfaceMiddleware, MiddlewareControlSurface, Reaper,
};
use rxrust::prelude::*;

use std::collections::VecDeque;

use reaper_medium::RegistrationHandle;
use reaper_rx::{
    ActionRx, ActionRxHookPostCommand, ActionRxHookPostCommand2, ActionRxProvider,
    ControlSurfaceRxDriver,
};
use slog::info;
use std::ops::Deref;
use std::panic::AssertUnwindSafe;

/// Executes the complete integration test.
///
/// Calls the given callback as soon as finished (either when the first test step failed
/// or when all steps have executed successfully).
pub fn execute_integration_test(on_finish: impl Fn(Result<(), &str>) + 'static) {
    Reaper::get().clear_console();
    log("# Testing reaper-rs\n");
    let steps: VecDeque<_> = create_test_steps().collect();
    let step_count = steps.len();
    let rx_setup = RxSetup::setup();
    execute_next_step(steps, step_count, move |result| {
        rx_setup.teardown();
        on_finish(result);
    });
}

#[derive(Debug)]
struct TestControlSurfaceMiddleware {
    change_detector: ChangeDetector,
    rx_driver: ControlSurfaceRxDriver,
}

impl TestControlSurfaceMiddleware {
    fn new() -> Self {
        Self {
            change_detector: ChangeDetector::new(),
            rx_driver: ControlSurfaceRxDriver::new(Test::control_surface_rx().clone()),
        }
    }
}

impl ControlSurfaceMiddleware for TestControlSurfaceMiddleware {
    fn handle_event(&self, event: ControlSurfaceEvent) {
        self.change_detector.process(event, |e| {
            self.rx_driver.handle_change(e);
        });
    }
}

struct RxSetup {
    control_surface_reg_handle:
        RegistrationHandle<MiddlewareControlSurface<TestControlSurfaceMiddleware>>,
}

impl ActionRxProvider for RxSetup {
    fn action_rx() -> &'static ActionRx {
        Test::action_rx()
    }
}

impl RxSetup {
    fn setup() -> RxSetup {
        let mut session = Reaper::get().medium_session();
        session
            .plugin_register_add_hook_post_command::<ActionRxHookPostCommand<Self>>()
            .unwrap();
        session
            .plugin_register_add_hook_post_command_2::<ActionRxHookPostCommand2<Self>>()
            .unwrap();
        RxSetup {
            control_surface_reg_handle: {
                let surface = MiddlewareControlSurface::new(TestControlSurfaceMiddleware::new());
                session
                    .plugin_register_add_csurf_inst(Box::new(surface))
                    .expect("couldn't register test control surface")
            },
        }
    }

    fn teardown(&self) {
        let mut session = Reaper::get().medium_session();
        unsafe {
            let _ = session.plugin_register_remove_csurf_inst(self.control_surface_reg_handle);
        }
        session.plugin_register_remove_hook_post_command_2::<ActionRxHookPostCommand2<Self>>();
        session.plugin_register_remove_hook_post_command::<ActionRxHookPostCommand<Self>>();
    }
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
    let reaper = Reaper::get();
    if reaper_version_matches(&step) {
        let result = {
            let mut finished = LocalSubject::new();
            let context = TestStepContext {
                finished: finished.clone(),
            };
            let step_name = step.name.clone();
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                (step.operation)(reaper.deref(), context)
            }))
            .unwrap_or_else(|_| Err(format!("Test [{}] panicked", step_name).into()));
            finished.complete();
            result
        };
        match result {
            Ok(()) => {
                reaper
                    .do_later_in_main_thread_from_main_thread_asap(move || {
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
        reaper
            .do_later_in_main_thread_from_main_thread_asap(move || {
                execute_next_step(steps, step_count, on_finish)
            })
            .expect("couldn't schedule next test step");
    }
}

fn reaper_version_matches(step: &TestStep) -> bool {
    use VersionRestriction::*;
    match &step.version_restriction {
        AllVersions => true,
        Min(v) => Reaper::get().version() >= *v,
        Max(v) => Reaper::get().version() <= *v,
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

fn log(msg: impl Into<String>) {
    let msg = msg.into();
    let reaper = Reaper::get();
    info!(reaper.logger(), "{}", &msg);
    reaper.show_console_msg(msg)
}
