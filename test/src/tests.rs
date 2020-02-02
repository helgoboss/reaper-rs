use std::borrow::Cow;
use crate::api::{TestStep, step};
use reaper_rs::high_level::{Project, Reaper};
use std::rc::Rc;
use std::cell::RefCell;
// TODO Change rxRust so we don't always have to import this ... see existing trait refactoring issue
use rxrust::prelude::*;

pub fn create_test_steps() -> impl IntoIterator<Item=TestStep> {
    vec!(
        step("Create empty project in new tab", |reaper| {
            // Given
            let current_project_before = reaper.get_current_project();
            let project_count_before = reaper.get_project_count();
            // When
            struct State { count: i32, event_project: Option<Project> }
            let mut state = Rc::new(RefCell::new(State { count: 0, event_project: None }));
            let mirrored_state = state.clone();
            reaper.project_switched().subscribe(move |p: Project| {
                let mut state = (*state).borrow_mut();
                state.count += 1;
                state.event_project = Some(p);
            });
            reaper.create_empty_project_in_new_tab();
            // Then
            ensure!(mirrored_state.borrow().count == 2);
            Ok(())
        })
    )
}