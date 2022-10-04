use crossbeam_channel::{Receiver, Sender};
use reaper_high::{MainThreadTask, Reaper, TaskSupport, DEFAULT_MAIN_THREAD_TASK_CHANNEL_CAPACITY};
use reaper_medium::ReaperVersion;
use reaper_rx::{ActionRx, ActionRxProvider, ControlSurfaceRx, MainRx};
use rxrust::prelude::*;
use std::borrow::Cow;
use std::error::Error;

type TestStepFinished = LocalSubject<'static, (), ()>;
pub struct TestStepContext {
    pub finished: TestStepFinished,
}
type TestStepResult = Result<(), Box<dyn Error>>;

type TestOperation = dyn FnOnce(&Reaper, TestStepContext) -> TestStepResult;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub version_restriction: VersionRestriction,
    pub operation: Box<TestOperation>,
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

pub(crate) struct Test {
    main_rx: MainRx,
    task_support: TaskSupport,
    pub(crate) task_sender: Sender<MainThreadTask>,
    pub(crate) task_receiver: Receiver<MainThreadTask>,
}

impl Default for Test {
    fn default() -> Self {
        let (sender, receiver) =
            crossbeam_channel::bounded(DEFAULT_MAIN_THREAD_TASK_CHANNEL_CAPACITY);
        Self {
            main_rx: Default::default(),
            task_support: TaskSupport::new(sender.clone()),
            task_sender: sender,
            task_receiver: receiver,
        }
    }
}

/// Okay because static getter checks thread.
unsafe impl Sync for Test {}
unsafe impl Send for Test {}

impl Test {
    pub fn control_surface_rx() -> &'static ControlSurfaceRx {
        Test::get().main_rx.control_surface()
    }

    pub fn task_support() -> &'static TaskSupport {
        &Test::get().task_support
    }

    pub(crate) fn get() -> &'static Test {
        Reaper::get().require_main_thread();
        &TEST
    }
}

impl ActionRxProvider for Test {
    fn action_rx() -> &'static ActionRx {
        Test::get().main_rx.action()
    }
}

static TEST: once_cell::sync::Lazy<Test> = once_cell::sync::Lazy::new(Default::default);
