use reaper_high::Reaper;
use reaper_medium::ReaperVersion;
use reaper_rx::{ActionRx, ControlSurfaceRx, MainRx};
use rxrust::prelude::*;
use std::borrow::Cow;

type TestStepFinished = LocalSubject<'static, (), ()>;
pub struct TestStepContext {
    pub finished: TestStepFinished,
}
type TestStepResult = Result<(), Cow<'static, str>>;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub version_restriction: VersionRestriction,
    pub operation: Box<dyn FnOnce(&Reaper, TestStepContext) -> TestStepResult>,
}

pub fn step<Op>(
    version_restriction: VersionRestriction,
    name: impl Into<Cow<'static, str>>,
    operation: Op,
) -> TestStep
where
    Op: FnOnce(&Reaper, TestStepContext) -> TestStepResult + 'static,
{
    TestStep {
        version_restriction,
        name: name.into(),
        operation: Box::new(operation),
    }
}

// Although currently not used, the Min and Max feature should be available
#[allow(dead_code)]
pub enum VersionRestriction {
    /// Executes this step in all REAPER versions.
    AllVersions,
    /// Executes this step in all REAPER versions equal or above the given one.
    Min(ReaperVersion<'static>),
    /// Executes this step in all REAPER versions equal or below the given one.
    Max(ReaperVersion<'static>),
}

#[derive(Default)]
pub(crate) struct Test {
    main_rx: MainRx,
}

/// Okay because static getter checks thread.
unsafe impl Sync for Test {}
unsafe impl Send for Test {}

impl Test {
    pub fn action_rx() -> &'static ActionRx {
        Test::get().main_rx.action()
    }

    pub fn control_surface_rx() -> &'static ControlSurfaceRx {
        Test::get().main_rx.control_surface()
    }

    fn get() -> &'static Test {
        Reaper::get().require_main_thread();
        &TEST
    }
}

static TEST: once_cell::sync::Lazy<Test> = once_cell::sync::Lazy::new(Default::default);
