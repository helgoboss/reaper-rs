use std::borrow::Cow;
use crate::api::{TestStep, step};
use reaper_rs::high_level::{Project, Reaper, Track, ActionKind, get_media_track_guid};
use std::rc::Rc;
use std::cell::RefCell;
// TODO Change rxRust so we don't always have to import this ... see existing trait refactoring issue
use rxrust::prelude::*;
use rxrust::ops::TakeUntil;
use std::ops::{Deref, DerefMut};
use c_str_macro::c_str;
use std::ffi::CStr;

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
        step("Create empty project in new tab", |reaper, step| {
            // Given
            let current_project_before = reaper.get_current_project();
            let project_count_before = reaper.get_project_count();
            // When
            struct State { count: i32, project: Option<Project> }
            let state = track_changes(
                State { count: 0, project: None },
                |state| {
                    reaper.project_switched().take_until(step.finished).subscribe(move |p| {
                        let mut state = state.borrow_mut();
                        state.count += 1;
                        state.project = Some(p);
                    });
                },
            );
            let new_project = reaper.create_empty_project_in_new_tab();
            // Then
            check_eq!(current_project_before, current_project_before);
            check_eq!(reaper.get_project_count(), project_count_before + 1);
            check_eq!(reaper.get_projects().count() as u32, project_count_before + 1);
            check_ne!(reaper.get_current_project(), current_project_before);
            check_eq!(reaper.get_current_project(), new_project);
            check_ne!(reaper.get_projects().nth(0), Some(new_project));
            //            assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().first() == newProject);
//            assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().count() == projectCountBefore + 1);
            check_eq!(new_project.get_track_count(), 0);
            check!(new_project.get_index() > 0);
            check_eq!(new_project.get_file_path(), None);
            check_eq!(state.borrow().count, 1);
            check_eq!(state.borrow().project, Some(new_project));
            Ok(())
        }),
        step("Add track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            // When
            #[derive(Default)]
            struct State { count: i32, track: Option<Track> }
            let state = track_changes(
                State::default(),
                |state| {
                    reaper.track_added().take_until(step.finished).subscribe(move |t| {
                        let mut state = state.borrow_mut();
                        state.count += 1;
                        state.track = Some(t.into());
                    });
                },
            );
            let new_track = project.add_track();
            // Then
            check_eq!(project.get_track_count(), 1);
            check_eq!(new_track.get_index(), 0);
            check_eq!(state.borrow().count, 1);
            check_eq!(state.borrow().track.clone(), Some(new_track));
            Ok(())
        }),
        step("FnMut action", |reaper, step| {
            // TODO Add this as new test
            return Ok(());
            let mut i = 0;
            let action1 = reaper.register_action(
                c_str!("reaperRsCounter"),
                c_str!("reaper-rs counter"),
                move || {
                    let owned = format!("Hello from Rust number {}\0", i);
                    let reaper = Reaper::instance();
                    reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
                    i += 1;
                },
                ActionKind::NotToggleable,
            );
            Ok(())
        }),
        step("Query master track", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            // When
            let master_track = project.get_master_track();
            // Then
            check!(master_track.is_master_track());
            Ok(())
        }),
        step("Query all tracks", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            project.add_track();
            // When
            let tracks = project.get_tracks();
            // Then
            check_eq!(tracks.count(), 2);
            Ok(())
        }),
        step("Query track by GUID", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let first_track = project.get_first_track().ok_or("No first track")?;
            let new_track = project.add_track();
            // When
            let found_track = project.get_track_by_guid(new_track.get_guid());
            // Then
            check!(found_track.is_available());
            check_eq!(&found_track, &new_track);
            check_ne!(&found_track, &first_track);
            check_eq!(new_track.get_guid(), &get_media_track_guid(new_track.get_media_track()));
            Ok(())
        }),
    )
}