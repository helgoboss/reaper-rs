use std::borrow::Cow;
use reaper_rs::high_level::Reaper;
use std::error::Error;
use rxrust::subject::SubjectValue;
use rxrust::prelude::*;

type TestStepFinished = LocalSubject<'static, SubjectValue<()>, SubjectValue<()>>;
pub struct TestStepContext {
    pub finished: TestStepFinished
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

macro_rules! check {
    ($condition:expr) => {
        {
            let result = if ($condition) {
                Ok(())
            } else {
                Err(stringify!($condition))
            };
            result?
        }
    }
}

macro_rules! check_eq {
    ($actual:expr, $expected:expr) => {
        {
            let actual = $actual;
            let expected = $expected;
            let result = if (actual == expected) {
                Ok(())
            } else {
                let actual_expr = stringify!($actual);
                let expected_expr = stringify!($expected);
                Err(format!("[{}] was expected to be [{}] but is {:?}", actual_expr, expected_expr, actual))
            };
            result?
        }
    }
}

macro_rules! check_ne {
    ($actual:expr, $unexpected:expr) => {
        {
            let actual = $actual;
            let unexpected = $unexpected;
            let result = if (actual == unexpected) {
                let actual_expr = stringify!($actual);
                let unexpected_expr = stringify!($unexpected);
                Err(format!("[{}] was expected to not be [{}] but it is ({:?})", actual_expr, unexpected_expr, actual))
            } else {
                Ok(())
            };
            result?
        }
    }
}