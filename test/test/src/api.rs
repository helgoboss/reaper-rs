use reaper_rs_high::Reaper;
use reaper_rs_medium::ReaperVersion;
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

pub enum VersionRestriction {
    AllVersions,
    Min(ReaperVersion<'static>),
    Max(ReaperVersion<'static>),
}
