use std::borrow::Cow;
use crate::api::{TestStep, step};
use reaper_rs::high_level::{Project, Reaper};
use std::rc::Rc;
use std::cell::RefCell;
// TODO Change rxRust so we don't always have to import this ... see existing trait refactoring issue
use rxrust::prelude::*;
use std::ops::{Deref, DerefMut};

fn share<T>(value: T) -> (Rc<RefCell<T>>, Rc<RefCell<T>>) {
    let shareable = Rc::new(RefCell::new(value));
    let mirror = shareable.clone();
    (shareable, mirror)
}

// Use for tracking changes made to a value within static closures that would move (take ownership)
// of that value
fn track_changes<T>(initial_value: T, op: impl FnOnce(Rc<RefCell<T>>)) -> Rc<RefCell<T>> {
    let (value, mirrored_value) = share(initial_value);
    op(value);
    mirrored_value
}


pub fn create_test_steps() -> impl IntoIterator<Item=TestStep> {
    vec!(
        step("Create empty project in new tab", |reaper| {
            // Given
            let current_project_before = reaper.get_current_project();
            let project_count_before = reaper.get_project_count();
            // When
            struct State { count: i32, project: Option<Project> }
            let state = track_changes(State { count: 0, project: None }, |state| {
                reaper.project_switched().subscribe_all(
                    move |p: Project| {
                        let mut state = state.borrow_mut();
                        state.count += 1;
                        state.project = Some(p);
                    },
                    |_| {},
                    || println!("Complete!"),
                );
            });
            reaper.create_empty_project_in_new_tab();
            // Then
            check_eq!(state.borrow().count, 1);
            Ok(())
        }),
        step("Add track", |reaper| {
            // Given
            // When
            // Then
            check_eq!("2", "5");
            Ok(())
        })
    )
}