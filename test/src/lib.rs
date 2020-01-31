use reaper_rs::high_level::{Reaper, Project};
use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;
use rxrust::prelude::*;
use std::collections::VecDeque;
use std::ffi::CString;
use std::panic;
use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};

pub struct ReaperRsIntegrationTest {
    reaper: &'static Reaper,
    test_steps: VecDeque<TestStep>,
}

type Operation = Box<dyn FnOnce()>;

struct TestStep {
    name: String,
    operation: Operation,
}

impl ReaperRsIntegrationTest {
    pub fn new(reaper: &'static Reaper) -> ReaperRsIntegrationTest {
        ReaperRsIntegrationTest {
            reaper,
            test_steps: VecDeque::new(),
        }
    }

    pub fn execute(&mut self) {
        self.test_steps.clear();
        self.reaper.clear_console();
        self.log("# Testing reaper-rs");
        let result = panic::catch_unwind(|| {
//            self.create_empty_project_in_new_tab();
            panic!("OOOOOHHHH NEEEEEIN")
        });
        if let Err(panic) = result {
            let error_msg = match panic.downcast::<&str>() {
                Ok(p) => p.to_string(),
                Err(panic) => match panic.downcast::<String>() {
                    Ok(p) => *p,
                    Err(_) => String::from("Unknown error")
                }
            };
            self.log(format!("Failure while building test steps: {}", error_msg))
        }
    }

    fn log(&self, msg: impl Into<Cow<'static, str>>) {
        let msg = match msg.into() {
            Borrowed(b) => CString::new(b),
            Owned(o) => CString::new(o),
        };
        self.reaper.show_console_msg(&msg.unwrap())
    }

    fn create_empty_project_in_new_tab(&self) -> Result<(), Box<dyn Error>> {
        // Given
        let current_project_before = self.reaper.get_current_project();
        let project_count_before = self.reaper.get_project_count();
        // When
        struct State { count: i32, event_project: Option<Project> }
        let mut state = Rc::new(RefCell::new(State { count: 0, event_project: None }));
        let mut mirrored_state = state.clone();
        self.reaper.project_switched().subscribe(move |p: Project| {
            let mut state = (*state).borrow_mut();
            state.count += 1;
            state.event_project = Some(p);
        });
        self.reaper.create_empty_project_in_new_tab();
        // Then
        assert_eq!(mirrored_state.borrow().count, 1);
        Ok(())
    }
}