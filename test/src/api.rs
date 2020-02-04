use std::borrow::Cow;
use reaper_rs::high_level::Reaper;
use std::error::Error;
use rxrust::prelude::*;
use rxrust::subject::SubjectValue;

type TestStepResult = Result<(), Cow<'static, str>>;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub operation: Box<dyn FnOnce(&'static Reaper) -> TestStepResult>,
}

pub fn step<Op>(name: impl Into<Cow<'static, str>>, operation: Op) -> TestStep
    where
        Op: FnOnce(&'static Reaper) -> TestStepResult + 'static
{
    TestStep {
        name: name.into(),
        operation: Box::new(operation),
    }
}

type Finished = LocalSubject<'static, SubjectValue<()>, SubjectValue<()>>;

pub fn step_until<Op>(name: impl Into<Cow<'static, str>>, operation: Op) -> TestStep
    where
        Op: FnOnce(&'static Reaper, Finished) -> TestStepResult + 'static,
{
    TestStep {
        name: name.into(),
        operation: Box::new(|reaper| {
            let mut test_over: Finished = Subject::local();
            let result = operation(reaper, test_over.fork());
            test_over.next(());
            result
        }),
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
                Err(format!("[{}] was expected to be [{}] but it turned out to be {:?}", actual_expr, expected_expr, actual))
            };
            result?
        }
    }
}