use std::borrow::Cow;
use reaper_rs::high_level::Reaper;
use std::error::Error;

type TestStepResult = Result<(), Cow<'static, str>>;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub operation: Box<dyn FnOnce(&'static Reaper) -> TestStepResult>,
}

pub fn step(name: impl Into<Cow<'static, str>>, operation: impl FnOnce(&'static Reaper) -> TestStepResult + 'static) -> TestStep {
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
                Err(format!("[{}] was expected to be [{}] but it turned out to be {:?}", actual_expr, expected_expr, actual))
            };
            result?
        }
    }
}