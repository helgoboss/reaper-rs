use reaper_rs::high_level::{Reaper, Project};
use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;
use rxrust::prelude::*;

pub struct ReaperRsIntegrationTest {
    reaper: &'static Reaper
}

impl ReaperRsIntegrationTest {
    pub fn new(reaper: &'static Reaper) -> ReaperRsIntegrationTest {
        ReaperRsIntegrationTest { reaper }
    }

    pub fn execute(&self) {
        self.create_empty_project_in_new_tab();
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