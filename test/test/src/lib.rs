#[macro_use]
mod assert;
mod api;
mod invocation_mock;
mod tests;

use crate::api::{Test, TestStep, TestStepContext, VersionRestriction};
use crate::tests::create_test_steps;
use reaper_high::{
    ChangeDetectionMiddleware, ControlSurfaceEvent, ControlSurfaceMiddleware, FutureMiddleware,
    FutureSupport, MiddlewareControlSurface, Reaper, DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
};
use rxrust::prelude::*;

use anyhow::anyhow;
use reaper_rx::{ActionRxHookPostCommand, ActionRxHookPostCommand2, ControlSurfaceRxMiddleware};
use std::fmt::Display;
use std::panic::AssertUnwindSafe;

pub struct IntegrationTest {
    future_support: FutureSupport,
}

impl IntegrationTest {
    pub fn setup() -> Self {
        let mut session = Reaper::get().medium_session();
        session
            .plugin_register_add_hook_post_command::<ActionRxHookPostCommand<Test>>()
            .unwrap();
        let _ = session.plugin_register_add_hook_post_command_2::<ActionRxHookPostCommand2<Test>>();
        let (spawner, executor) = reaper_high::run_loop_executor::new_spawner_and_executor(
            DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
        );
        let (local_spawner, local_executor) =
            reaper_high::local_run_loop_executor::new_spawner_and_executor(
                DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
            );
        let future_support = FutureSupport::new(spawner, local_spawner);
        let future_middleware = FutureMiddleware::new(executor, local_executor);
        let surface =
            MiddlewareControlSurface::new(TestControlSurfaceMiddleware::new(future_middleware));
        session
            .plugin_register_add_csurf_inst(Box::new(surface))
            .expect("couldn't register test control surface");
        Self { future_support }
    }

    pub fn future_support(&self) -> &FutureSupport {
        &self.future_support
    }
}

/// Executes the complete integration test.
pub async fn execute_integration_test() -> anyhow::Result<()> {
    Reaper::get().clear_console();
    log("# Testing reaper-rs\n");
    execute_integration_test_internal()
        .await
        .inspect(|_| log("\n**Integration test was successful**\n\n"))
        .inspect_err(|e| log_failure(e))
}
async fn execute_integration_test_internal() -> anyhow::Result<()> {
    let steps: Vec<_> = create_test_steps().collect();
    for (i, step) in steps.into_iter().enumerate() {
        log_step(i, &step.name);
        if !reaper_version_matches(&step) {
            // REAPER version doesn't match
            let reason = match step.version_restriction {
                VersionRestriction::Min(_) => "REAPER version too low",
                VersionRestriction::Max(_) => "REAPER version too high",
                _ => unreachable!(),
            };
            log_skip(reason);
            continue;
        }
        let future = async {
            let reaper = Reaper::get();
            let mut finished = LocalSubject::new();
            let context = TestStepContext {
                finished: finished.clone(),
            };
            let step_name = step.name.clone();
            let result =
                std::panic::catch_unwind(AssertUnwindSafe(|| (step.operation)(reaper, context)))
                    .unwrap_or_else(|_| Err(anyhow!(format!("Test [{step_name}] panicked"))));
            finished.complete();
            result
        };
        future.await?;
    }
    Ok(())
}

#[derive(Debug)]
struct TestControlSurfaceMiddleware {
    change_detection_middleware: ChangeDetectionMiddleware,
    rx_middleware: ControlSurfaceRxMiddleware,
    future_middleware: FutureMiddleware,
}

impl TestControlSurfaceMiddleware {
    fn new(future_middleware: FutureMiddleware) -> Self {
        Self {
            change_detection_middleware: ChangeDetectionMiddleware::new(),
            rx_middleware: ControlSurfaceRxMiddleware::new(Test::control_surface_rx().clone()),
            future_middleware,
        }
    }
}

impl ControlSurfaceMiddleware for TestControlSurfaceMiddleware {
    fn run(&mut self) {
        self.future_middleware.run();
    }

    fn handle_event(&self, event: ControlSurfaceEvent) -> bool {
        self.change_detection_middleware.process(&event, |e| {
            self.rx_middleware.handle_change(e);
        })
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

fn log_failure(msg: impl Display) {
    log(format!("→ **FAILED**\n\n{msg}"));
}

fn log_step(step_index: usize, name: &str) {
    log(format!("{}. {}\n", step_index + 1, name));
}

fn log(msg: impl Into<String>) {
    let msg = msg.into();
    let reaper = Reaper::get();
    println!("{msg}");
    reaper.show_console_msg(msg)
}
