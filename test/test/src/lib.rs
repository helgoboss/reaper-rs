#[macro_use]
mod assert;
mod api;
mod invocation_mock;
mod tests;

use crate::api::{Test, TestStep, TestStepContext, VersionRestriction};
use crate::tests::create_test_steps;
use reaper_high::{
    ChangeDetectionMiddleware, ControlSurfaceEvent, ControlSurfaceMiddleware, MainTaskMiddleware,
    MiddlewareControlSurface, Reaper,
};
use rxrust::prelude::*;

use std::collections::VecDeque;

use reaper_medium::RegistrationHandle;
use reaper_rx::{ActionRxHookPostCommand, ActionRxHookPostCommand2, ControlSurfaceRxMiddleware};
use std::error::Error;
use std::fmt::Display;
use std::panic::AssertUnwindSafe;
use tracing::info;

/// Executes the complete integration test.
///
/// Calls the given callback as soon as finished (either when the first test step failed
/// or when all steps have executed successfully).
pub fn execute_integration_test(on_finish: impl Fn(Result<(), Box<dyn Error>>) + 'static) {
    Reaper::get().clear_console();
    log("# Testing reaper-rs\n");
    let steps: VecDeque<_> = create_test_steps().collect();
    let step_count = steps.len();
    let rx_setup = RxSetup::setup();
    execute_next_step(steps, step_count, move |result| {
        // We keep the surface around after teardown because this whole thing is driven by the surface.
        // See plugin_register_remove_csurf_inst. Would otherwise result in undefined behavior, maybe crash.
        let surface = rx_setup.teardown();
        on_finish(result);
        surface
    });
}

#[derive(Debug)]
struct TestControlSurfaceMiddleware {
    change_detection_middleware: ChangeDetectionMiddleware,
    rx_middleware: ControlSurfaceRxMiddleware,
    main_task_middleware: MainTaskMiddleware,
}

impl TestControlSurfaceMiddleware {
    fn new() -> Self {
        Self {
            change_detection_middleware: ChangeDetectionMiddleware::new(),
            rx_middleware: ControlSurfaceRxMiddleware::new(Test::control_surface_rx().clone()),
            main_task_middleware: MainTaskMiddleware::new(
                Test::get().task_sender.clone(),
                Test::get().task_receiver.clone(),
            ),
        }
    }
}

impl ControlSurfaceMiddleware for TestControlSurfaceMiddleware {
    fn run(&mut self) {
        self.main_task_middleware.run();
    }

    fn handle_event(&self, event: ControlSurfaceEvent) -> bool {
        self.change_detection_middleware.process(&event, |e| {
            self.rx_middleware.handle_change(e);
        })
    }
}

struct RxSetup {
    control_surface_reg_handle:
        RegistrationHandle<MiddlewareControlSurface<TestControlSurfaceMiddleware>>,
}

impl RxSetup {
    fn setup() -> RxSetup {
        let mut session = Reaper::get().medium_session();
        session
            .plugin_register_add_hook_post_command::<ActionRxHookPostCommand<Test>>()
            .unwrap();
        let _ = session.plugin_register_add_hook_post_command_2::<ActionRxHookPostCommand2<Test>>();
        RxSetup {
            control_surface_reg_handle: {
                let surface = MiddlewareControlSurface::new(TestControlSurfaceMiddleware::new());
                session
                    .plugin_register_add_csurf_inst(Box::new(surface))
                    .expect("couldn't register test control surface")
            },
        }
    }

    fn teardown(&self) -> Option<Box<MiddlewareControlSurface<TestControlSurfaceMiddleware>>> {
        let mut session = Reaper::get().medium_session();
        let csurf_inst =
            unsafe { session.plugin_register_remove_csurf_inst(self.control_surface_reg_handle) };
        session.plugin_register_remove_hook_post_command_2::<ActionRxHookPostCommand2<Test>>();
        session.plugin_register_remove_hook_post_command::<ActionRxHookPostCommand<Test>>();
        csurf_inst
    }
}

fn execute_next_step(
    mut steps: VecDeque<TestStep>,
    step_count: usize,
    on_finish: impl Fn(
            Result<(), Box<dyn Error>>,
        ) -> Option<Box<MiddlewareControlSurface<TestControlSurfaceMiddleware>>>
        + 'static,
) -> Option<Box<MiddlewareControlSurface<TestControlSurfaceMiddleware>>> {
    let step = match steps.pop_front() {
        Some(step) => step,
        None => {
            log("\n**Integration test was successful**\n\n");
            return on_finish(Ok(()));
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
            let result =
                std::panic::catch_unwind(AssertUnwindSafe(|| (step.operation)(reaper, context)))
                    .unwrap_or_else(|_| Err(format!("Test [{step_name}] panicked").into()));
            finished.complete();
            result
        };
        match result {
            Ok(()) => {
                Test::task_support()
                    .do_later_in_main_thread_from_main_thread_asap(move || {
                        execute_next_step(steps, step_count, on_finish);
                    })
                    .expect("couldn't schedule next test step");
                None
            }
            Err(e) => {
                log_failure(&e);
                on_finish(Err(e))
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
        Test::task_support()
            .do_later_in_main_thread_from_main_thread_asap(move || {
                execute_next_step(steps, step_count, on_finish);
            })
            .expect("couldn't schedule next test step");
        None
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
    log(format!("→ **SKIPPED** ({msg})"));
}

fn log_failure(msg: &impl Display) {
    log(format!("→ **FAILED**\n\n{msg}"));
}

fn log_step(step_index: usize, name: &str) {
    log(format!("{}. {}\n", step_index + 1, name));
}

fn log(msg: impl Into<String>) {
    let msg = msg.into();
    let reaper = Reaper::get();
    info!("{msg}");
    reaper.show_console_msg(msg)
}
