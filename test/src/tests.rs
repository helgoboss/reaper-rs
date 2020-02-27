use std::borrow::Cow;
use crate::api::{TestStep, step};
use reaper_rs::high_level::{Project, Reaper, Track, ActionKind, get_media_track_guid, Guid, InputMonitoringMode, MidiRecordingInput, RecordingInput, MidiInputDevice, Volume, Pan, AutomationMode, ActionCharacter, toggleable, MessageBoxResult, MessageBoxKind, Tempo, StuffMidiMessageTarget, MidiEvent, MidiMessage, FxChain, FxParameterCharacter};
use std::rc::Rc;
use std::cell::{RefCell, Ref, Cell};
// TODO Change rxRust so we don't always have to import this ... see existing trait refactoring issue
use rxrust::prelude::*;
use std::ops::{Deref, DerefMut};
use c_str_macro::c_str;
use std::ffi::{CStr, CString};
use std::convert::TryFrom;
use super::mock::observe_invocations;
use std::ptr::null_mut;
use wmidi;
use std::iter;
use slog::debug;

pub fn create_test_steps() -> impl Iterator<Item=TestStep> {
    let steps_a = vec!(
        step("Create empty project in new tab", |reaper, step| {
            // Given
            let current_project_before = reaper.get_current_project();
            let project_count_before = reaper.get_project_count();
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.project_switched().take_until(step.finished).subscribe(move |p| {
                    mock.invoke(p);
                });
            });
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
            check!(new_project.get_file_path().is_none());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), new_project);
            Ok(())
        }),
        step("Add track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            // When
            #[derive(Default)]
            struct State { count: i32, track: Option<Track> }
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_added().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            let new_track = project.add_track();
            // Then
            check_eq!(project.get_track_count(), 1);
            check_eq!(new_track.get_index(), 0);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), new_track);
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
            let first_track = get_track(0)?;
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
            let track = get_track(0)?;
            // When
            let track_project = track.get_project();
            // Then
            check_eq!(track_project, project);
            Ok(())
        }),
        step("Query track name", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let track_name = track.get_name();
            // Then
            check_eq!(track_name.as_bytes().len(), 0);
            Ok(())
        }),
        step("Set track name", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            // TODO Factor this state pattern out
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_name_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.set_name(c_str!("Foo Bla"));
            // Then
            check_eq!(track.get_name(), c_str!("Foo Bla").to_owned());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Query track input monitoring", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let mode = track.get_input_monitoring_mode();
            // Then
            check_eq!(mode, InputMonitoringMode::Normal);
            Ok(())
        }),
        step("Set track input monitoring", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            #[derive(Default)]
            struct State { count: i32, track: Option<Track> }
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_input_monitoring_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.set_input_monitoring_mode(InputMonitoringMode::NotWhenPlaying);
            // Then
            check_eq!(track.get_input_monitoring_mode(), InputMonitoringMode::NotWhenPlaying);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Query track recording input", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let input = track.get_recording_input();
            // Then
            match input {
                RecordingInput::Mono => Ok(()),
                _ => Err("Expected MidiRecordingInput".into())
            }
        }),
        step("Set track recording input MIDI all/all", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_input_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.set_recording_input(MidiRecordingInput::from_all_devices_and_channels());
            // Then
            let input = track.get_recording_input();
            let input_data = match input {
                RecordingInput::Midi(d) => d,
                _ => return Err("Expected MIDI input".into())
            };
            check!(input_data.get_channel().is_none());
            check!(input_data.get_device().is_none());
            check_eq!(input_data.get_rec_input_index(), 6112);
            check_eq!(RecordingInput::from_rec_input_index(6112), input);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Set track recording input MIDI 4/5", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            track.set_recording_input(MidiRecordingInput::from_device_and_channel(MidiInputDevice::new(4), 5));
            // Then
            let input = track.get_recording_input();
            let input_data = match input {
                RecordingInput::Midi(d) => d,
                _ => return Err("Expected MIDI input".into())
            };
            check_eq!(input_data.get_channel(), Some(5));
            check_eq!(input_data.get_device().ok_or("Expected device")?.get_id(), 4);
            Ok(())
        }),
        step("Set track recording input MIDI 7/all", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            track.set_recording_input(MidiRecordingInput::from_all_channels_of_device(MidiInputDevice::new(7)));
            // Then
            let input = track.get_recording_input();
            let input_data = match input {
                RecordingInput::Midi(d) => d,
                _ => return Err("Expected MIDI input".into())
            };
            check!(input_data.get_channel().is_none());
            check_eq!(input_data.get_device(), Some(MidiInputDevice::new(7)));
            Ok(())
        }),
        step("Set track recording input MIDI all/15", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            track.set_recording_input(MidiRecordingInput::from_all_devices_with_channel(15));
            // Then
            let input = track.get_recording_input();
            let input_data = match input {
                RecordingInput::Midi(d) => d,
                _ => return Err("Expected MIDI input".into())
            };
            check_eq!(input_data.get_channel(), Some(15));
            check!(input_data.get_device().is_none());
            Ok(())
        }),
        step("Query track volume", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let volume = track.get_volume();
            // Then
            check_eq!(volume.get_reaper_value(), 1.0);
            check_eq!(volume.get_db(), 0.0);
            check_eq!(volume.get_normalized_value(), 0.71599999999999997);
            Ok(())
        }),
        step("Set track volume", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_volume_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.set_volume(Volume::of_normalized_value(0.25));
            // Then
            let volume = track.get_volume();
            check_eq!(volume.get_reaper_value(), 0.031588093366685013);
            check_eq!(volume.get_db(), -30.009531739774296);
            check_eq!(volume.get_normalized_value(), 0.25000000000003497);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Query track pan", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let pan = track.get_pan();
            // Then
            check_eq!(pan.get_reaper_value(), 0.0);
            check_eq!(pan.get_normalized_value(), 0.5);
            Ok(())
        }),
        step("Set track pan", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_pan_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.set_pan(Pan::of_normalized_value(0.25));
            // Then
            let pan = track.get_pan();
            check_eq!(pan.get_reaper_value(), -0.5);
            check_eq!(pan.get_normalized_value(), 0.25);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Query track selection state", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            // When
            let is_selected = track.is_selected();
            // Then
            check!(!is_selected);
            check_eq!(project.get_selected_track_count(false), 0);
            Ok(())
        }),
        step("Select track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            let track2 = project.get_track_by_index(2).ok_or("No track at index 2")?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_selected_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.select();
            track2.select();
            // Then
            check!(track.is_selected());
            check!(track2.is_selected());
            check_eq!(project.get_selected_track_count(false), 2);
            let first_selected_track = project.get_first_selected_track(false)
                .ok_or("Couldn't get first selected track")?;
            check_eq!(first_selected_track.get_index(), 0);
            check_eq!(project.get_selected_tracks(false).count(), 2);
            check_eq!(mock.invocation_count(), 2);
            check_eq!(mock.last_arg(), track2);
            Ok(())
        }),
        step("Unselect track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_selected_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.unselect();
            // Then
            check!(!track.is_selected());
            check_eq!(project.get_selected_track_count(false), 1);
            let first_selected_track = project.get_first_selected_track(false)
                .ok_or("Couldn't get first selected track")?;
            check_eq!(first_selected_track.get_index(), 2);
            check_eq!(project.get_selected_tracks(false).count(), 1);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Select master track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let master_track = project.get_master_track();
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_selected_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            project.unselect_all_tracks();
            master_track.select();
            // Then
            check!(master_track.is_selected());
            check_eq!(project.get_selected_track_count(true), 1);
            let first_selected_track = project.get_first_selected_track(true)
                .ok_or("Couldn't get first selected track")?;
            check!(first_selected_track.is_master_track());
            check_eq!(project.get_selected_tracks(true).count(), 1);
            // TODO REAPER doesn't notify us about master track selection currently
            check_eq!(mock.invocation_count(), 1);
            let last_arg: Track = mock.last_arg().into();
            check_eq!(last_arg.get_index(), 2);
            Ok(())
        }),
        step("Query track auto arm mode", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let is_in_auto_arm_mode = track.has_auto_arm_enabled();
            // Then
            check!(!is_in_auto_arm_mode);
            Ok(())
        }),
        step("Query track arm state", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let is_armed = track.is_armed(true);
            let is_armed_ignoring_auto_arm = track.is_armed(false);
            // Then
            check!(!is_armed);
            check!(!is_armed_ignoring_auto_arm);
            Ok(())
        }),
        step("Arm track in normal mode", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.arm(true);
            // Then
            check!(track.is_armed(true));
            check!(track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Disarm track in normal mode", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.disarm(true);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Enable track auto-arm mode", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            track.enable_auto_arm();
            // Then
            check!(track.has_auto_arm_enabled());
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            Ok(())
        }),
        step("Arm track in auto-arm mode", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.arm(true);
            // Then
            check!(track.is_armed(true));
            // TODO Interesting! GetMediaTrackInfo_Value read with I_RECARM seems to support auto-arm already!
            // So maybe we should remove the chunk check and the parameter supportAutoArm
            check!(track.is_armed(false));
            check!(track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Disarm track in auto-arm mode", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.disarm(true);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Disable track auto-arm mode", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            track.disable_auto_arm();
            // Then
            check!(!track.has_auto_arm_enabled());
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            Ok(())
        }),
        step("Switch to normal track mode while armed", |reaper, _| {
            // Given
            let track = get_track(0)?;
            track.arm(true);
            check!(track.is_armed(true));
            // When
            track.disable_auto_arm();
            // Then
            check!(!track.has_auto_arm_enabled());
            check!(track.is_armed(true));
            check!(track.is_armed(false));
            Ok(())
        }),
        step("Switch track to auto-arm mode while armed", |reaper, _| {
            // Given
            let track = get_track(0)?;
            track.unselect();
            // When
            track.enable_auto_arm();
            // Then
            check!(track.has_auto_arm_enabled());
            check!(track.is_armed(true));
            check!(track.is_armed(false));
            Ok(())
        }),
        step("Disarm track in auto-arm mode (ignoring auto-arm)", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.disarm(false);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Arm track in auto-arm mode (ignoring auto-arm)", |reaper, step| {
            // Given
            let track = get_track(0)?;
            track.enable_auto_arm();
            check!(track.has_auto_arm_enabled());
            check!(!track.is_armed(true));
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_arm_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.arm(false);
            // Then
            check!(track.is_armed(true));
            check!(track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Select track exclusively", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
            let track_3 = project.get_track_by_index(2).ok_or("Missing track 3")?;
            track_1.unselect();
            track_2.select();
            track_3.select();
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_selected_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track_1.select_exclusively();
            // Then
            check!(track_1.is_selected());
            check!(!track_2.is_selected());
            check!(!track_3.is_selected());
            check_eq!(project.get_selected_track_count(false), 1);
            check!(project.get_first_selected_track(false).is_some());
            check_eq!(project.get_selected_tracks(false).count(), 1);
            check_eq!(mock.invocation_count(), 3);
            Ok(())
        }),
        step("Remove track", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track_count_before = project.get_track_count();
            let track_1 = project.get_track_by_number(1).ok_or("Missing track 1")?;
            let track_2 = project.get_track_by_number(2).ok_or("Missing track 2")?;
            let track_2_guid = track_2.get_guid();
            check!(track_1.is_available());
            check_eq!(track_2.get_index(), 1);
            check!(track_2.is_available());
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_removed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            project.remove_track(&track_1);
            // Then
            check_eq!(project.get_track_count(), track_count_before - 1);
            check!(!track_1.is_available());
            check_eq!(track_2.get_index(), 0);
            check_eq!(track_2.get_guid(), track_2_guid);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track_1);
            Ok(())
        }),
        step("Query track automation mode", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let automation_mode = track.get_automation_mode();
            let global_automation_override = reaper.get_global_automation_override();
            let effective_automation_mode = track.get_effective_automation_mode();
            // Then
            check_eq!(automation_mode, AutomationMode::TrimRead);
            check_eq!(global_automation_override, AutomationMode::NoOverride);
            check_eq!(effective_automation_mode, AutomationMode::TrimRead);
            Ok(())
        }),
        step("Query track send count", |reaper, _| {
            // Given
            let track = get_track(0)?;
            // When
            let send_count = track.get_send_count();
            // Then
            check_eq!(send_count, 0);
            check!(track.get_send_by_index(0).is_none());
            check!(!track.get_send_by_target_track(track.clone()).is_available());
            check!(!track.get_index_based_send_by_index(0).is_available());
            check_eq!(track.get_sends().count(), 0);
            Ok(())
        }),
        step("Add track send", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
            // When
            let send = track_1.add_send_to(track_2.clone());
            // Then
            check_eq!(track_1.get_send_count(), 1);
            check_eq!(track_1.get_send_by_index(0), Some(send));
            check!(track_1.get_send_by_target_track(track_2.clone()).is_available());
            check!(!track_2.get_send_by_target_track(track_1.clone()).is_available());
            check!(track_1.get_index_based_send_by_index(0).is_available());
            check_eq!(track_1.get_sends().count(), 1);
            Ok(())
        }),
        step("Query track send", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
            let track_3 = project.add_track();
            // When
            let send_to_track_2 = track_1.get_send_by_target_track(track_2.clone());
            let send_to_track_3 = track_1.add_send_to(track_3.clone());
            // Then
            check!(send_to_track_2.is_available());
            check!(send_to_track_3.is_available());
            check_eq!(send_to_track_2.get_index(), 0);
            check_eq!(send_to_track_3.get_index(), 1);
            check_eq!(send_to_track_2.get_source_track(), track_1);
            check_eq!(send_to_track_3.get_source_track(), track_1);
            check_eq!(send_to_track_2.get_target_track(), track_2);
            check_eq!(send_to_track_3.get_target_track(), track_3);
            check_eq!(send_to_track_2.get_volume().get_db(), 0.0);
            check_eq!(send_to_track_3.get_volume().get_db(), 0.0);
            Ok(())
        }),
        step("Set track send volume", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_3 = project.get_track_by_index(2).ok_or("Missing track 3")?;
            let send = track_1.get_send_by_target_track(track_3);
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_send_volume_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            send.set_volume(Volume::of_normalized_value(0.25));
            // Then
            check_eq!(send.get_volume().get_db(), -30.009531739774296);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), send);
            Ok(())
        }),
        step("Set track send pan", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_3 = project.get_track_by_index(2).ok_or("Missing track 3")?;
            let send = track_1.get_send_by_target_track(track_3);
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_send_pan_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            send.set_pan(Pan::of_normalized_value(0.25));
            // Then
            check_eq!(send.get_pan().get_reaper_value(), -0.5);
            check_eq!(send.get_pan().get_normalized_value(), 0.25);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), send);
            Ok(())
        }),
        step("Query action", |reaper, _| {
            // Given
            let track = get_track(0)?;
            track.select_exclusively();
            check!(!track.is_muted());
            // When
            let toggle_action = reaper.get_main_section().get_action_by_command_id(6);
            let normal_action = reaper.get_main_section().get_action_by_command_id(41075);
            let normal_action_by_index = reaper.get_main_section().get_action_by_index(normal_action.get_index());
            // Then
            check!(toggle_action.is_available());
            check!(normal_action.is_available());
            check_eq!(toggle_action.get_character(), ActionCharacter::Toggle);
            check_eq!(normal_action.get_character(), ActionCharacter::Trigger);
            check!(!toggle_action.is_on());
            check!(!normal_action.is_on());
            check_eq!(toggle_action.clone(), toggle_action);
            check_eq!(toggle_action.get_command_id(), 6);
            check!(toggle_action.get_command_name().is_none());
            check_eq!(toggle_action.get_name(), Some(c_str!("Track: Toggle mute for selected tracks")));
            check!(toggle_action.get_index() > 0);
            check_eq!(toggle_action.get_section(), reaper.get_main_section());
            check_eq!(normal_action_by_index, normal_action);
            Ok(())
        }),
        step("Invoke action", |reaper, step| {
            // Given
            let action = reaper.get_main_section().get_action_by_command_id(6);
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.action_invoked().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            action.invoke_as_trigger(None);
            // Then
            check!(action.is_on());
            check!(track.is_muted());
            // TODO Actually it would be nice if the actionInvoked event would be raised but it isn't
            check_eq!(mock.invocation_count(), 0);
            Ok(())
        }),
        step("Test actionInvoked event", |reaper, step| {
            // Given
            let action = reaper.get_main_section().get_action_by_command_id(1582);
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.action_invoked().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            reaper.medium.main_on_command_ex(action.get_command_id() as i32, 0, null_mut());
            // Then
            check_eq!(mock.invocation_count(), 1);
            check_eq!(*mock.last_arg(), action);
            Ok(())
        }),
        step("Unmute track", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_mute_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.unmute();
            // Then
            check!(!track.is_muted());
            // For some reason REAPER doesn't call SetSurfaceMute on control surfaces when an action
            // caused the muting. So HelperControlSurface still thinks the track was unmuted and
            // therefore will not fire a change event!
            check_eq!(mock.invocation_count(), 0);
            Ok(())
        }),
        step("Mute track", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_mute_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.mute();
            // Then
            check!(track.is_muted());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Solo track", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_solo_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.solo();
            // Then
            check!(track.is_solo());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Unsolo track", |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_solo_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            track.unsolo();
            // Then
            check!(!track.is_solo());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Generate GUID", |reaper, _| {
            // Given
            // When
            let guid = reaper.generate_guid();
            // Then
            check_eq!(guid.to_string_with_braces().len(), 38);
            Ok(())
        }),
        step("Main section functions", |reaper, _| {
            // Given
            let section = reaper.get_main_section();
            // When
            let actions = section.get_actions();
            // Then
            check_eq!(actions.count() as u32, section.get_action_count());
            Ok(())
        }),
        step("Register and unregister action", |reaper, _| {
            // Given
            // When
            // TODO Rename RegisteredAction to ActionRegistration or something like that
            let (mock, reg) = observe_invocations(|mock| {
                reaper.register_action(
                    c_str!("reaperRsTest"),
                    c_str!("reaper-rs test action"),
                    move || {
                        mock.invoke(42);
                    },
                    ActionKind::NotToggleable,
                )
            });
            let action = reaper.get_action_by_command_name(c_str!("reaperRsTest").into());
            // Then
            check!(action.is_available());
            check_eq!(mock.invocation_count(), 0);
            action.invoke_as_trigger(None);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), 42);
            check_eq!(action.get_character(), ActionCharacter::Trigger);
            check!(action.get_command_id() > 0);
            check_eq!(action.get_command_name(), Some(c_str!("reaperRsTest")));
            check!(action.get_index() >= 0);
            check!(!action.is_on());
            check_eq!(action.get_name(), Some(c_str!("reaper-rs test action")));
            reg.unregister();
            check!(!action.is_available());
            Ok(())
        }),
        step("Register and unregister toggle action", |reaper, _| {
            // Given
            // When
            let (mock, reg) = observe_invocations(|mock| {
                let cloned_mock = mock.clone();
                reaper.register_action(
                    c_str!("reaperRsTest2"),
                    c_str!("reaper-rs test toggle action"),
                    move || {
                        mock.invoke(43);
                    },
                    toggleable(move || {
                        cloned_mock.invocation_count() % 2 == 1
                    }),
                )
            });
            let action = reaper.get_action_by_command_name(c_str!("reaperRsTest2").into());
            // Then
            check!(action.is_available());
            check_eq!(mock.invocation_count(), 0);
            check!(!action.is_on());
            action.invoke_as_trigger(None);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), 43);
            check!(action.is_on());
            check_eq!(action.get_character(), ActionCharacter::Toggle);
            check!(action.get_command_id() > 0);
            check_eq!(action.get_command_name(), Some(c_str!("reaperRsTest2")));
            check!(action.get_index() >= 0);
            check_eq!(action.get_name(), Some(c_str!("reaper-rs test toggle action")));
            reg.unregister();
            check!(!action.is_available());
            Ok(())
        }),
    ).into_iter();
    // TODO Insert FX tests HERE!
    let steps_b = vec!(
        step("Insert track at", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
            let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_added().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            let new_track = project.insert_track_at(1);
            new_track.set_name(c_str!("Inserted track"));
            // Then
            check_eq!(project.get_track_count(), 4);
            check_eq!(new_track.get_index(), 1);
            check_eq!(new_track.get_name().as_c_str(), c_str!("Inserted track"));
            check_eq!(track_2.get_index(), 2);
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), new_track);
            Ok(())
        }),
        step("Query MIDI input devices", |reaper, _| {
            // Given
            // When
            let devs = reaper.get_midi_input_devices();
            let dev_0 = reaper.get_midi_input_device_by_id(0);
            // Then
            // TODO There might be no MIDI input devices
//            check_ne!(devs.count(), 0);
//            check!(dev_0.is_available());
            Ok(())
        }),
        step("Query MIDI output devices", |reaper, _| {
            // Given
            // When
            let devs = reaper.get_midi_output_devices();
            let dev_0 = reaper.get_midi_output_device_by_id(0);
            // Then
            check_ne!(devs.count(), 0);
            check!(dev_0.is_available());
            Ok(())
        }),
        step("Stuff MIDI messages", |reaper, step| {
            // Given
            let msg = wmidi::MidiMessage::NoteOn(
                wmidi::Channel::Ch1, wmidi::Note::A4, wmidi::U7::try_from(100).unwrap());
            let mut bytes = vec![0u8; msg.bytes_size()];
            msg.copy_to_slice(bytes.as_mut_slice()).unwrap();
            // When
            reaper.midi_message_received()
                .take_until(step.finished)
                .subscribe(move |evt| {
                    // Right now not invoked because MIDI message arrives async.
                    // TODO As soon as we have an Observable which is not generic on Observer, introduce
                    //  steps which return an Observable<TestStepResult, ()> in order to test
                    //  asynchronously that stuffed MIDI messages arrived via midi_message_received().
                });
            reaper.stuff_midi_message(
                StuffMidiMessageTarget::VirtualMidiKeyboard,
                (bytes[0], bytes[1], bytes[2]),
            );
            // Then
            Ok(())
        }),
        step("Use undoable", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.track_name_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            let track_mirror = track.clone();
            project.undoable(c_str!("ReaPlus integration test operation"), move || {
                track_mirror.set_name(c_str!("Renamed"));
            });
            let label = project.get_label_of_last_undoable_action();
            // Then
            check_eq!(track.get_name().as_c_str(), c_str!("Renamed"));
            check_eq!(label, Some(c_str!("ReaPlus integration test operation")));
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), track);
            Ok(())
        }),
        step("Undo", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            // When
            let successful = project.undo();
            let label = project.get_label_of_last_redoable_action();
            // Then
            check!(successful);
            check_eq!(track.get_name().as_bytes().len(), 0);
            check_eq!(label, Some(c_str!("ReaPlus integration test operation")));
            Ok(())
        }),
        step("Redo", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            let track = get_track(0)?;
            // When
            let successful = project.redo();
            let label = project.get_label_of_last_undoable_action();
            // Then
            check!(successful);
            check_eq!(track.get_name().as_c_str(), c_str!("Renamed"));
            check_eq!(label, Some(c_str!("ReaPlus integration test operation")));
            Ok(())
        }),
        step("Get REAPER window", |reaper, _| {
            // Given
            // When
            let window = reaper.get_main_window();
            // Then
            check!(!window.is_null());
            Ok(())
        }),
        step("Mark project as dirty", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            // When
            project.mark_as_dirty();
            // Then
            // TODO Doesn't say very much because it has been dirty before already. Save before!?
            check!(project.is_dirty());
            Ok(())
        }),
        step("Get project tempo", |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            // When
            let tempo = project.get_tempo();
            // Then
            check_eq!(tempo.get_bpm(), 120.0);
            check_eq!(tempo.get_normalized_value(), 119.0 / 959.0);
            Ok(())
        }),
        step("Set project tempo", |reaper, step| {
            // Given
            let project = reaper.get_current_project();
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.master_tempo_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            project.set_tempo(Tempo::of_bpm(130.0), false);
            // Then
            check_eq!(project.get_tempo().get_bpm(), 130.0);
            // TODO There should be only one event invocation
            check_eq!(mock.invocation_count(), 2);
            check_eq!(mock.last_arg(), true);
            Ok(())
        }),
        step("Show message box", |reaper, _| {
            // Given
            // When
            let result = reaper.show_message_box(c_str!("Tests are finished"), c_str!("ReaPlus"), MessageBoxKind::Ok);
            // Then
            check_eq!(result, MessageBoxResult::Ok);
            Ok(())
        }),
    ).into_iter();
    let reaper = Reaper::instance();
    let output_fx_steps = create_fx_steps(
        "Output FX chain",
        || get_track(0).map(|t| t.get_normal_fx_chain()),
    );
    let input_fx_steps = create_fx_steps(
        "Input FX chain",
        || get_track(1).map(|t| t.get_input_fx_chain()),
    );
    iter::empty()
        .chain(steps_a)
        .chain(output_fx_steps)
        .chain(input_fx_steps)
        .chain(steps_b)
}

fn create_fx_steps(
    prefix: &'static str,
    get_fx_chain: impl Fn() -> Result<FxChain, &'static str> + 'static + Copy,
) -> impl Iterator<Item=TestStep> {
    let steps = vec!(
        step("Query fx chain", move |reaper, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            // Then
            check_eq!(fx_chain.get_fx_count(), 0);
            check_eq!(fx_chain.get_fxs().count(), 0);
            check!(fx_chain.get_fx_by_index(0).is_none());
            check!(fx_chain.get_first_fx().is_none());
            check!(fx_chain.get_last_fx().is_none());
            let non_existing_guid = Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            check!(!fx_chain.get_fx_by_guid(&non_existing_guid).is_available());
            check!(!fx_chain.get_fx_by_guid_and_index(&non_existing_guid, 0).is_available());
            check!(fx_chain.get_first_fx_by_name(c_str!("bla")).is_none());
            check!(fx_chain.get_chunk().is_none());
            Ok(())
        }),
        step("Add track fx by original name", move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.fx_added().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            let fx = fx_chain.add_fx_by_original_name(c_str!("ReaControlMIDI (Cockos)"));
            // Then
            check!(fx.is_some());
            check_eq!(fx_chain.get_fx_count(), 1);
            check_eq!(fx_chain.get_fxs().count(), 1);
            check_eq!(fx_chain.get_fx_by_index(0), fx);
            check_eq!(fx_chain.get_first_fx(), fx);
            check_eq!(fx_chain.get_last_fx(), fx);
            let fx = fx.unwrap();
            let guid = fx.get_guid();
            check!(guid.is_some());
            let guid = guid.unwrap();
            let guid_string = guid.to_string_without_braces();
            check_eq!(guid_string.len(), 36);
            check!(guid_string.find(|c| c == '{' || c == '}').is_none());
            check!(fx_chain.get_fx_by_guid(&guid).is_available());
            check_eq!(fx_chain.get_fx_by_guid(&guid), fx);
            check!(fx_chain.get_fx_by_guid_and_index(&guid, 0).is_available());
            // If this doesn't work, then the index hasn't automatically corrected itself
            check!(fx_chain.get_fx_by_guid_and_index(&guid, 1).is_available());
            let non_existing_guid = Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            check!(!fx_chain.get_fx_by_guid_and_index(&non_existing_guid, 0).is_available());
            check_eq!(fx_chain.get_first_fx_by_name(c_str!("ReaControlMIDI (Cockos)")), Some(fx.clone()));
            let chain_chunk = fx_chain.get_chunk();
            check!(chain_chunk.is_some());
            let chain_chunk = chain_chunk.unwrap();
            check!(chain_chunk.starts_with("<FXCHAIN"));
            check!(chain_chunk.ends_with("\n>"));
            let first_tag = chain_chunk.find_first_tag(0);
            check!(first_tag.is_some());
            let first_tag = first_tag.unwrap();
            check_eq!(first_tag.get_content().deref(), chain_chunk.get_content().deref());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), fx);
            Ok(())
        }),
        step("Check track fx with 1 fx", move |reaper, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.get_track();
            // When
            let fx_1 = fx_chain.get_fx_by_index(0).ok_or("Couldn't find first fx")?;
            // Then
            check!(fx_1.is_available());
            check_eq!(fx_1.get_index(), 0);
            check_eq!(fx_1.get_query_index(), if fx_chain.is_input_fx() { 0x1000000 } else {0});
            check!(fx_1.get_guid().is_some());
            check_eq!(fx_1.get_name().as_c_str(), c_str!("VST: ReaControlMIDI (Cockos)"));
            let chunk = fx_1.get_chunk();
            check!(chunk.starts_with("BYPASS 0 0 0"));
            //            debug!(reaper.logger, "{:?}", chunk.get_parent_chunk());
            check!(chunk.ends_with("\nWAK 0 0"));
            let tag_chunk = fx_1.get_tag_chunk();
            check!(tag_chunk.starts_with(r#"<VST "VST: ReaControlMIDI (Cockos)" reacontrolmidi"#));
            check!(tag_chunk.ends_with("\n>"));
            let state_chunk = fx_1.get_state_chunk();
            check!(!state_chunk.contains("<"));
            check!(!state_chunk.contains(">"));

            let fx_1_info = fx_1.get_info();
            let stem = fx_1_info.file_name.file_stem().ok_or("No stem")?;
            check_eq!(stem, "reacontrolmidi");
            check_eq!(fx_1_info.type_expression, "VST");
            check_eq!(fx_1_info.sub_type_expression, "VST");
            check_eq!(fx_1_info.effect_name, "ReaControlMIDI");
            check_eq!(fx_1_info.vendor_name, "Cockos");

            check_eq!(fx_1.get_track(), track);
            check_eq!(fx_1.is_input_fx(), fx_chain.is_input_fx());
            check_eq!(fx_1.get_chain(), fx_chain);
            check_eq!(fx_1.get_parameter_count(), 17);
            check_eq!(fx_1.get_parameters().count(), 17);
            check!(fx_1.get_parameter_by_index(15).is_available());
            check!(!fx_1.get_parameter_by_index(17).is_available());
            check!(fx_1.is_enabled());
            Ok(())
        }),
        step("Disable track fx", move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx_1 = fx_chain.get_fx_by_index(0).ok_or("Couldn't find first fx")?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.fx_enabled_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            fx_1.disable();
            // Then
            check!(!fx_1.is_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), fx_1);
            Ok(())
        }),
        step("Enable track fx", move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx_1 = fx_chain.get_fx_by_index(0).ok_or("Couldn't find first fx")?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper.fx_enabled_changed().take_until(step.finished).subscribe(move |t| {
                    mock.invoke(t);
                });
            });
            fx_1.enable();
            // Then
            check!(fx_1.is_enabled());
            check_eq!(mock.invocation_count(), 1);
            check_eq!(mock.last_arg(), fx_1);
            Ok(())
        }),
        step("Check track fx with 2 fx", move |reaper, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.get_track();
            // When
            let fx_1 = fx_chain.get_fx_by_index(0).ok_or("Couldn't find first fx")?;
            let fx_2 = fx_chain.add_fx_by_original_name(c_str!("ReaSynth (Cockos)"))
                .ok_or("Couldn't add ReaSynth")?;
            // Then
            check!(fx_1.is_available());
            check!(fx_2.is_available());
            check_eq!(fx_1.get_index(), 0);
            check_eq!(fx_2.get_index(), 1);
            check_eq!(fx_1.get_query_index(), if fx_chain.is_input_fx() { 0x1000000 } else { 0 });
            check_eq!(fx_2.get_query_index(), if fx_chain.is_input_fx() { 0x1000001 } else { 1 });
            check!(fx_1.get_guid().is_some());
            check!(fx_2.get_guid().is_some());
            check_eq!(fx_1.get_name().as_c_str(), c_str!("VST: ReaControlMIDI (Cockos)"));
            check_eq!(fx_2.get_name().as_c_str(), c_str!("VSTi: ReaSynth (Cockos)"));
            let chunk_1 = fx_1.get_chunk();
            check!(chunk_1.starts_with("BYPASS 0 0 0"));
            check!(chunk_1.ends_with("\nWAK 0 0"));
            let tag_chunk_1 = fx_1.get_tag_chunk();
            check!(tag_chunk_1.starts_with(r#"<VST "VST: ReaControlMIDI (Cockos)" reacontrolmidi"#));
            check!(tag_chunk_1.ends_with("\n>"));
            let state_chunk_1 = fx_1.get_state_chunk();
            check!(!state_chunk_1.contains("<"));
            check!(!state_chunk_1.contains(">"));
            let fx_1_info = fx_1.get_info();
            let fx_2_info = fx_2.get_info();
            let stem_1 = fx_1_info.file_name.file_stem().ok_or("No stem")?;
            let stem_2 = fx_2_info.file_name.file_stem().ok_or("No stem")?;
            check_eq!(stem_1, "reacontrolmidi");
            check_eq!(stem_2, "reasynth");
            check_eq!(fx_1.get_track(), track);
            check_eq!(fx_2.get_track(), track);
            check_eq!(fx_1.is_input_fx(), fx_chain.is_input_fx());
            check_eq!(fx_2.is_input_fx(), fx_chain.is_input_fx());
            check_eq!(fx_1.get_chain(), fx_chain);
            check_eq!(fx_2.get_chain(), fx_chain);
            check_eq!(fx_1.get_parameter_count(), 17);
            check_eq!(fx_2.get_parameter_count(), 15);
            check_eq!(fx_1.get_parameters().count(), 17);
            check_eq!(fx_2.get_parameters().count(), 15);
            check!(fx_1.get_parameter_by_index(15).is_available());
            check!(!fx_1.get_parameter_by_index(17).is_available());
            check!(track.get_fx_by_query_index(if fx_chain.is_input_fx() { 0x1000000 } else { 0 }).is_some());
            check!(track.get_fx_by_query_index(if fx_chain.is_input_fx() { 0x1000001 } else { 1 }).is_some());
            check!(!track.get_fx_by_query_index(if fx_chain.is_input_fx() { 0 } else { 0x1000000 }).is_some());
            check!(!track.get_fx_by_query_index(if fx_chain.is_input_fx() { 1 } else { 0x1000001 }).is_some());
            if !fx_chain.is_input_fx() {
                let first_instrument_fx = fx_chain.get_first_instrument_fx()
                    .ok_or("Couldn't find instrument FX")?;
                check_eq!(first_instrument_fx.get_index(), 1);
            }
            Ok(())
        }),
        step("Check fx parameter", move |reaper, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.get_track();
            let fx = fx_chain.get_fx_by_index(0).ok_or("Couldn't find first fx")?;
            // When
            let p = fx.get_parameter_by_index(5);
            // Then
            check!(p.is_available());
            check_eq!(p.get_name().as_c_str(), c_str!("Pitch Wheel"));
            check_eq!(p.get_index(), 5);
            check_eq!(p.get_character(), FxParameterCharacter::Continuous);
            check_eq!(p.clone(), p);
            check_eq!(p.get_formatted_value().as_c_str(), c_str!("0"));
            check_eq!(p.get_normalized_value(), 0.5);
            check_eq!(p.get_reaper_value(), 0.5);
            check_eq!(p.format_normalized_value(p.get_normalized_value()).as_c_str(), c_str!("0"));
            check_eq!(p.get_fx(), fx);
            check!(p.get_step_size().is_none());
            Ok(())
        }),
    );
    steps.into_iter().map(move |s| {
        TestStep {
            name: format!("{} - {}", prefix, s.name).into(),
            ..s
        }
    })
}

fn get_track(index: u32) -> Result<Track, &'static str> {
    Reaper::instance().get_current_project().get_track_by_index(index).ok_or("Track not found")
}