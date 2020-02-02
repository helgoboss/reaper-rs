use std::borrow::Cow;
use reaper_rs::high_level::Reaper;
use std::error::Error;

pub struct TestStep {
    pub name: Cow<'static, str>,
    pub operation: Box<dyn FnOnce(&'static Reaper) -> Result<(), &str>>,
}

pub fn step(name: impl Into<Cow<'static, str>>, operation: impl FnOnce(&'static Reaper) -> Result<(), &str> + 'static) -> TestStep {
    TestStep {
        name: name.into(),
        operation: Box::new(operation),
    }
}

macro_rules! ensure {
    ($condition:expr) => {
        crate::api::assert_with_result($condition, stringify!($condition))?;
    }
}

pub fn assert_with_result(condition: bool, cause: &str) -> Result<(), &str> {
    if (condition) {
        Ok(())
    } else {
        Err(cause)
    }
}