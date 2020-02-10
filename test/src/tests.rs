use std::borrow::Cow;
use crate::api::{TestStep, step};
use reaper_rs::high_level::{Project, Reaper, Track, ActionKind, get_media_track_guid, Guid, InputMonitoringMode};
use std::rc::Rc;
use std::cell::RefCell;
// TODO Change rxRust so we don't always have to import this ... see existing trait refactoring issue
use rxrust::prelude::*;
use rxrust::ops::TakeUntil;
use std::ops::{Deref, DerefMut};
use c_str_macro::c_str;
use std::ffi::{CStr, CString};
use std::convert::TryFrom;

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
            let first_track = get_first_track()?;
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
        step("Query non-existent track by GUID", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            // When
            let guid = Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            let found_track = project.get_track_by_guid(&guid);
            // Then
            check!(!found_track.is_available());
            Ok(())
        }),
        step("Query track project", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track = get_first_track()?;
            // When
            let track_project = track.get_project();
            // Then
            check_eq!(track_project, project);
            Ok(())
        }),
        step("Query track name", |reaper, _| {
            // Given
            let track = get_first_track()?;
            // When
            let track_name = track.get_name();
            // Then
            check_eq!(track_name.as_bytes().len(), 0);
            Ok(())
        }),
        step("Set track name", |reaper, step| {
            // Given
            let track = get_first_track()?;
            // When
            // TODO Factor this state pattern out
            #[derive(Default)]
            struct State { count: i32, track: Option<Track> }
            let state = track_changes(
                State::default(),
                |state| {
                    reaper.track_name_changed().take_until(step.finished).subscribe(move |t| {
                        let mut state = state.borrow_mut();
                        state.count += 1;
                        state.track = Some(t.into());
                    });
                },
            );
            track.set_name(c_str!("Foo Bla"));
            // Then
            check_eq!(track.get_name(), c_str!("Foo Bla").to_owned());
            check_eq!(state.borrow().count, 1);
            check_eq!(&state.borrow().track, &Some(track));
            Ok(())
        }),
        step("Query track input monitoring", |reaper, _| {
            // Given
            let track = get_first_track()?;
            // When
            let mode = track.get_input_monitoring_mode();
            // Then
            check_eq!(mode, InputMonitoringMode::Off);
            Ok(())
        }),
        step("Set track input monitoring", |reaper, step| {
            // Given
            let track = get_first_track()?;
            // When
            #[derive(Default)]
            struct State { count: i32, track: Option<Track> }
            let state = track_changes(
                State::default(),
                |state| {
                    reaper.track_input_monitoring_changed().take_until(step.finished).subscribe(move |t| {
                        let mut state = state.borrow_mut();
                        state.count += 1;
                        state.track = Some(t.into());
                    });
                },
            );
            track.set_input_monitoring_mode(InputMonitoringMode::NotWhenPlaying);
            // Then
            check_eq!(track.get_input_monitoring_mode(), InputMonitoringMode::NotWhenPlaying);
            check_eq!(state.borrow().count, 1);
            check_eq!(&state.borrow().track, &Some(track));
            Ok(())
        }),
    )
}

fn get_first_track() -> Result<Track, &'static str> {
    Reaper::instance().get_current_project().get_first_track().ok_or("First track not found")
}