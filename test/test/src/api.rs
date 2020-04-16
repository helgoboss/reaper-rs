use reaper_rs_high::Reaper;
use rxrust::prelude::*;
use std::borrow::Cow;

type TestStepFinished = LocalSubject<'static, (), ()>;
pub struct TestStepContext {
    pub finished: TestStepFinished,
}
type TestStepResult = Result<(), Cow<'static, str>>;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub operation: Box<dyn FnOnce(&'static Reaper, TestStepContext) -> TestStepResult>,
}

pub fn step<Op>(name: impl Into<Cow<'static, str>>, operation: Op) -> TestStep
where
    Op: FnOnce(&'static Reaper, TestStepContext) -> TestStepResult + 'static,
{
    TestStep {
        name: name.into(),
        operation: Box::new(operation),
    }
}
