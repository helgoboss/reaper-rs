use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::iter;
use std::ops::Deref;
use std::ptr::null_mut;

use c_str_macro::c_str;

use reaper_rs_high::{
    get_media_track_guid, toggleable, ActionCharacter, ActionKind, FxChain, FxParameterCharacter,
    FxParameterValueRange, Guid, MidiInputDevice, Pan, Reaper, Tempo, Track, Volume,
};
use rxrust::prelude::*;

use crate::api::{step, TestStep};

use super::mock::observe_invocations;
use crate::api::VersionRestriction::{AllVersions, Min};
use helgoboss_midi::test_util::{channel, key_number, u7};
use helgoboss_midi::{RawShortMessage, ShortMessageFactory};
use reaper_rs_medium::NotificationBehavior::NotifyAll;
use reaper_rs_medium::ProjectContext::CurrentProject;
use reaper_rs_medium::{
    ActionValueChange, AutomationMode, Bpm, CommandId, Db, EnvChunkName, FxAddByNameBehavior,
    FxShowFlag, GangBehavior, GlobalAutomationOverride, InputMonitoringMode, MasterTrackBehavior,
    MessageBoxResult, MessageBoxType, MidiInputDeviceId, MidiOutputDeviceId, ReaperNormalizedValue,
    ReaperPanValue, ReaperPointer, ReaperVersion, ReaperVolumeValue, RecordArmState,
    RecordingInput, StuffMidiMessageTarget, TrackFxChainType, TrackFxRef, TrackInfoKey, TrackRef,
    TrackSendCategory, TrackSendDirection, TransferBehavior, UndoBehavior, ValueChange,
};
use std::os::raw::c_void;
use std::rc::Rc;
use std::time::Duration;

/// Creates all integration test steps to be executed. The order matters!
pub fn create_test_steps() -> impl Iterator<Item = TestStep> {
    // In theory all steps could be declared inline. But that makes the IDE become terribly slow.
    let steps_a = vec![
        basics(),
        create_empty_project_in_new_tab(),
        add_track(),
        fn_mut_action(),
        query_master_track(),
        query_all_tracks(),
        query_track_by_guid(),
        query_non_existent_track_by_guid(),
        query_track_project(),
        query_track_name(),
        set_track_name(),
        query_track_input_monitoring(),
        set_track_input_monitoring(),
        query_track_recording_input(),
        set_track_recording_input_midi_all_all(),
        set_track_recording_input_midi_4_5(),
        set_track_recording_input_midi_7_all(),
        set_track_recording_input_midi_all_15(),
        query_track_volume(),
        set_track_volume(),
        set_track_volume_extreme_values(),
        query_track_pan(),
        set_track_pan(),
        query_track_selection_state(),
        select_track(),
        unselect_track(),
        select_master_track(),
        query_track_auto_arm_mode(),
        query_track_arm_state(),
        arm_track_in_normal_mode(),
        disarm_track_in_normal_mode(),
        enable_track_in_auto_arm_mode(),
        arm_track_in_auto_arm_mode(),
        disarm_track_in_auto_arm_mode(),
        disable_track_auto_arm_mode(),
        switch_to_normal_track_mode_while_armed(),
        switch_track_to_auto_arm_mode_while_armed(),
        disarm_track_in_auto_arm_mode_ignoring_auto_arm(),
        arm_track_in_auto_arm_mode_ignoring_auto_arm(),
        select_track_exclusively(),
        remove_track(),
        query_track_automation_mode(),
        query_track_send_count(),
        add_track_send(),
        query_track_send(),
        set_track_send_volume(),
        set_track_send_pan(),
        query_action(),
        invoke_action(),
        test_action_invoked_event(),
        unmute_track(),
        mute_track(),
        solo_track(),
        unsolo_track(),
        generate_guid(),
        main_section_functions(),
        register_and_unregister_action(),
        register_and_unregister_toggle_action(),
    ]
    .into_iter();
    let steps_b = vec![
        insert_track_at(),
        query_midi_input_devices(),
        query_midi_output_devices(),
        stuff_midi_devices(),
        use_undoable(),
        undo(),
        redo(),
        get_reaper_window(),
        mark_project_as_dirty(),
        get_project_tempo(),
        set_project_tempo(),
        show_message_box(),
    ]
    .into_iter();
    let output_fx_steps = create_fx_steps("Output FX chain", || {
        get_track(0).map(|t| t.get_normal_fx_chain())
    });
    let input_fx_steps = create_fx_steps("Input FX chain", || {
        get_track(1).map(|t| t.get_input_fx_chain())
    });
    iter::empty()
        .chain(steps_a)
        .chain(output_fx_steps)
        .chain(input_fx_steps)
        .chain(steps_b)
}

fn show_message_box() -> TestStep {
    step(AllVersions, "Show message box", |reaper, _| {
        // Given
        // When
        let result = reaper.show_message_box(
            c_str!("Tests are finished"),
            c_str!("reaper-rs"),
            MessageBoxType::Ok,
        );
        // Then
        check_eq!(result, MessageBoxResult::Ok);
        Ok(())
    })
}

fn set_project_tempo() -> TestStep {
    step(AllVersions, "Set project tempo", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .master_tempo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        project.set_tempo(
            Tempo::from_bpm(Bpm::new(130.0)),
            UndoBehavior::OmitUndoPoint,
        );
        // Then
        check_eq!(project.get_tempo().get_bpm(), Bpm::new(130.0));
        // TODO-low There should be only one event invocation
        check_eq!(mock.get_invocation_count(), 2);
        check_eq!(mock.get_last_arg(), ());
        Ok(())
    })
}

fn get_project_tempo() -> TestStep {
    step(AllVersions, "Get project tempo", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        // When
        let tempo = project.get_tempo();
        // Then
        check_eq!(tempo.get_bpm(), Bpm::new(120.0));
        check_eq!(tempo.get_normalized_value(), 119.0 / 959.0);
        Ok(())
    })
}

fn mark_project_as_dirty() -> TestStep {
    step(AllVersions, "Mark project as dirty", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        // When
        project.mark_as_dirty();
        // Then
        // TODO Doesn't say very much because it has been dirty before already. Save before!?
        check!(project.is_dirty());
        Ok(())
    })
}

fn get_reaper_window() -> TestStep {
    step(AllVersions, "Get REAPER window", |reaper, _| {
        // Given
        // When
        reaper.get_main_window();
        // Then
        Ok(())
    })
}

fn redo() -> TestStep {
    step(AllVersions, "Redo", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let successful = project.redo();
        let label = project.get_label_of_last_undoable_action();
        // Then
        check!(successful);
        check_eq!(track.get_name().as_c_str(), c_str!("Renamed"));
        check_eq!(
            label,
            Some(c_str!("reaper-rs integration test operation").to_owned())
        );
        Ok(())
    })
}

fn undo() -> TestStep {
    step(AllVersions, "Undo", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let successful = project.undo();
        let label = project.get_label_of_last_redoable_action();
        // Then
        check!(successful);
        check_eq!(track.get_name().as_bytes().len(), 0);
        check_eq!(
            label,
            Some(c_str!("reaper-rs integration test operation").to_owned())
        );
        Ok(())
    })
}

fn use_undoable() -> TestStep {
    step(AllVersions, "Use undoable", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_name_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let track_mirror = track.clone();
        project.undoable(c_str!("reaper-rs integration test operation"), move || {
            track_mirror.set_name(c_str!("Renamed"));
        });
        let label = project.get_label_of_last_undoable_action();
        // Then
        check_eq!(track.get_name().as_c_str(), c_str!("Renamed"));
        check_eq!(
            label,
            Some(c_str!("reaper-rs integration test operation").to_owned())
        );
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn stuff_midi_devices() -> TestStep {
    step(AllVersions, "Stuff MIDI messages", |reaper, step| {
        // Given
        let msg = RawShortMessage::note_on(channel(0), key_number(64), u7(100));
        // When
        reaper
            .midi_message_received()
            .take_until(step.finished)
            .subscribe(move |_evt| {
                // Right now not invoked because MIDI message arrives async.
                // TODO As soon as we have an Observable which is not generic on Observer,
                // introduce  steps which return an
                // Observable<TestStepResult, ()> in order to test
                //  asynchronously that stuffed MIDI messages arrived via
                // midi_message_received().
            });
        reaper.stuff_midi_message(StuffMidiMessageTarget::VirtualMidiKeyboardQueue, msg);
        // Then
        Ok(())
    })
}

fn query_midi_output_devices() -> TestStep {
    step(AllVersions, "Query MIDI output devices", |reaper, _| {
        // Given
        // When
        let devs = reaper.get_midi_output_devices();
        let dev_0 = reaper.get_midi_output_device_by_id(MidiOutputDeviceId::new(0));
        // Then
        check_ne!(devs.count(), 0);
        check!(dev_0.is_available());
        Ok(())
    })
}

fn query_midi_input_devices() -> TestStep {
    step(AllVersions, "Query MIDI input devices", |reaper, _| {
        // Given
        // When
        let _devs = reaper.get_midi_input_devices();
        let _dev_0 = reaper.get_midi_input_device_by_id(MidiInputDeviceId::new(0));
        // Then
        // TODO There might be no MIDI input devices
        //            check_ne!(devs.count(), 0);
        //            check!(dev_0.is_available());
        Ok(())
    })
}

fn insert_track_at() -> TestStep {
    step(AllVersions, "Insert track at", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_added()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let new_track = project.insert_track_at(1);
        new_track.set_name(c_str!("Inserted track"));
        // Then
        check_eq!(project.get_track_count(), 4);
        check_eq!(new_track.get_index(), Some(1));
        check_eq!(new_track.get_name().as_c_str(), c_str!("Inserted track"));
        check_eq!(track_2.get_index(), Some(2));
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), new_track);
        Ok(())
    })
}

fn register_and_unregister_toggle_action() -> TestStep {
    step(
        AllVersions,
        "Register and unregister toggle action",
        |reaper, _| {
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
                    toggleable(move || cloned_mock.get_invocation_count() % 2 == 1),
                )
            });
            let action = reaper.get_action_by_command_name(c_str!("reaperRsTest2").into());
            // Then
            let action_index = action.get_index();
            let command_id = action.get_command_id();
            check!(action.is_available());
            check_eq!(mock.get_invocation_count(), 0);
            check!(!action.is_on());
            action.invoke_as_trigger(None);
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), 43);
            check!(action.is_on());
            check_eq!(action.get_character(), ActionCharacter::Toggle);
            check!(action.get_command_id() > CommandId::new(1));
            check_eq!(
                action.get_command_name(),
                Some(c_str!("reaperRsTest2").to_owned())
            );
            check_eq!(
                action.get_name(),
                Some(c_str!("reaper-rs test toggle action").to_owned())
            );
            reg.unregister();
            check!(!action.is_available());
            Ok(())
        },
    )
}

fn register_and_unregister_action() -> TestStep {
    step(
        AllVersions,
        "Register and unregister action",
        |reaper, _| {
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
            check_eq!(mock.get_invocation_count(), 0);
            action.invoke_as_trigger(None);
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), 42);
            check_eq!(action.get_character(), ActionCharacter::Trigger);
            check!(action.get_command_id() > CommandId::new(1));
            check_eq!(
                action.get_command_name(),
                Some(c_str!("reaperRsTest").to_owned())
            );
            check!(!action.is_on());
            check_eq!(
                action.get_name(),
                Some(c_str!("reaper-rs test action").to_owned())
            );
            reg.unregister();
            check!(!action.is_available());
            Ok(())
        },
    )
}

fn main_section_functions() -> TestStep {
    step(AllVersions, "Main section functions", |reaper, _| {
        // Given
        let section = reaper.get_main_section();
        // When
        let actions = unsafe { section.get_actions() };
        // Then
        check_eq!(actions.count() as u32, section.get_action_count());
        Ok(())
    })
}

fn generate_guid() -> TestStep {
    step(AllVersions, "Generate GUID", |reaper, _| {
        // Given
        // When
        let guid = reaper.generate_guid();
        // Then
        check_eq!(guid.to_string_with_braces().len(), 38);
        Ok(())
    })
}

fn unsolo_track() -> TestStep {
    step(AllVersions, "Unsolo track", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_solo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unsolo();
        // Then
        check!(!track.is_solo());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn solo_track() -> TestStep {
    step(AllVersions, "Solo track", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_solo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.solo();
        // Then
        check!(track.is_solo());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn mute_track() -> TestStep {
    step(AllVersions, "Mute track", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_mute_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.mute();
        // Then
        check!(track.is_muted());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn unmute_track() -> TestStep {
    step(AllVersions, "Unmute track", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_mute_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unmute();
        // Then
        check!(!track.is_muted());
        // For some reason REAPER doesn't call SetSurfaceMute on control surfaces when an action
        // caused the muting. So HelperControlSurface still thinks the track was unmuted and
        // therefore will not fire a change event!
        check_eq!(mock.get_invocation_count(), 0);
        Ok(())
    })
}

fn test_action_invoked_event() -> TestStep {
    step(AllVersions, "Test actionInvoked event", |reaper, step| {
        // Given
        let action = reaper
            .get_main_section()
            .get_action_by_command_id(CommandId::new(1582));
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .action_invoked()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t.0);
                });
        });
        reaper
            .medium()
            .functions()
            .main_on_command_ex(action.get_command_id(), 0, CurrentProject);
        // Then
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(*mock.get_last_arg(), action);
        Ok(())
    })
}

fn invoke_action() -> TestStep {
    step(AllVersions, "Invoke action", |reaper, step| {
        // Given
        let action = reaper
            .get_main_section()
            .get_action_by_command_id(CommandId::new(6));
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .action_invoked()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        action.invoke_as_trigger(None);
        // Then
        check!(action.is_on());
        check!(track.is_muted());
        // TODO Actually it would be nice if the actionInvoked event would be raised but it
        // isn't
        check_eq!(mock.get_invocation_count(), 0);
        Ok(())
    })
}

fn query_action() -> TestStep {
    step(AllVersions, "Query action", |reaper, _| {
        // Given
        let track = get_track(0)?;
        track.select_exclusively();
        check!(!track.is_muted());
        // When
        let toggle_action = reaper
            .get_main_section()
            .get_action_by_command_id(CommandId::new(6));
        let normal_action = reaper
            .get_main_section()
            .get_action_by_command_id(CommandId::new(41075));
        let normal_action_by_index = reaper
            .get_main_section()
            .get_action_by_index(normal_action.get_index());
        // Then
        check!(toggle_action.is_available());
        check!(normal_action.is_available());
        check_eq!(toggle_action.get_character(), ActionCharacter::Toggle);
        check_eq!(normal_action.get_character(), ActionCharacter::Trigger);
        check!(!toggle_action.is_on());
        check!(!normal_action.is_on());
        check_eq!(toggle_action.clone(), toggle_action);
        check_eq!(toggle_action.get_command_id(), CommandId::new(6));
        check!(toggle_action.get_command_name().is_none());
        check_eq!(
            toggle_action.get_name(),
            Some(c_str!("Track: Toggle mute for selected tracks").to_owned())
        );
        check!(toggle_action.get_index() > 0);
        check_eq!(toggle_action.get_section(), reaper.get_main_section());
        check_eq!(normal_action_by_index, normal_action);
        Ok(())
    })
}

fn set_track_send_pan() -> TestStep {
    step(AllVersions, "Set track send pan", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
        let track_3 = project.get_track_by_index(2).ok_or("Missing track 3")?;
        let send = track_1.get_send_by_target_track(track_3);
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_send_pan_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        send.set_pan(Pan::from_normalized_value(0.25));
        // Then
        check_eq!(send.get_pan().get_reaper_value(), ReaperPanValue::new(-0.5));
        check_eq!(send.get_pan().get_normalized_value(), 0.25);
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), send);
        Ok(())
    })
}

fn set_track_send_volume() -> TestStep {
    step(AllVersions, "Set track send volume", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
        let track_3 = project.get_track_by_index(2).ok_or("Missing track 3")?;
        let send = track_1.get_send_by_target_track(track_3);
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_send_volume_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        send.set_volume(Volume::from_normalized_value(0.25));
        // Then
        check_eq!(send.get_volume().get_db(), Db::new(-30.009531739774296));
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), send);
        Ok(())
    })
}

fn query_track_send() -> TestStep {
    step(AllVersions, "Query track send", |reaper, _| {
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
        check_eq!(send_to_track_2.get_volume().get_db(), Db::ZERO_DB);
        check_eq!(send_to_track_3.get_volume().get_db(), Db::ZERO_DB);
        Ok(())
    })
}

fn add_track_send() -> TestStep {
    step(AllVersions, "Add track send", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        let track_1 = project.get_track_by_index(0).ok_or("Missing track 1")?;
        let track_2 = project.get_track_by_index(1).ok_or("Missing track 2")?;
        // When
        let send = track_1.add_send_to(track_2.clone());
        // Then
        check_eq!(track_1.get_send_count(), 1);
        check_eq!(track_1.get_send_by_index(0), Some(send));
        check!(
            track_1
                .get_send_by_target_track(track_2.clone())
                .is_available()
        );
        check!(
            !track_2
                .get_send_by_target_track(track_1.clone())
                .is_available()
        );
        check!(track_1.get_index_based_send_by_index(0).is_available());
        check_eq!(track_1.get_sends().count(), 1);
        Ok(())
    })
}

fn query_track_send_count() -> TestStep {
    step(AllVersions, "Query track send count", |_, _| {
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
    })
}

fn query_track_automation_mode() -> TestStep {
    step(AllVersions, "Query track automation mode", |reaper, _| {
        // Given
        let track = get_track(0)?;
        // When
        let automation_mode = track.get_automation_mode();
        let global_automation_override = reaper.get_global_automation_override();
        let effective_automation_mode = track.get_effective_automation_mode();
        // Then
        check_eq!(automation_mode, AutomationMode::TrimRead);
        check_eq!(global_automation_override, None);
        check_eq!(effective_automation_mode, Some(AutomationMode::TrimRead));
        Ok(())
    })
}

fn remove_track() -> TestStep {
    step(AllVersions, "Remove track", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track_count_before = project.get_track_count();
        let track_1 = project
            .get_track_by_ref(TrackRef::NormalTrack(0))
            .ok_or("Missing track 1")?;
        let track_2 = project
            .get_track_by_ref(TrackRef::NormalTrack(1))
            .ok_or("Missing track 2")?;
        let track_2_guid = track_2.get_guid();
        check!(track_1.is_available());
        check_eq!(track_2.get_index(), Some(1));
        check!(track_2.is_available());
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_removed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        project.remove_track(&track_1);
        // Then
        check_eq!(project.get_track_count(), track_count_before - 1);
        check!(!track_1.is_available());
        check_eq!(track_2.get_index(), Some(0));
        check_eq!(track_2.get_guid(), track_2_guid);
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track_1);
        Ok(())
    })
}

fn select_track_exclusively() -> TestStep {
    step(AllVersions, "Select track exclusively", |reaper, step| {
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
            reaper
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track_1.select_exclusively();
        // Then
        check!(track_1.is_selected());
        check!(!track_2.is_selected());
        check!(!track_3.is_selected());
        check_eq!(
            project.get_selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            1
        );
        check!(
            project
                .get_first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
                .is_some()
        );
        check_eq!(
            project
                .get_selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            1
        );
        check_eq!(mock.get_invocation_count(), 3);
        Ok(())
    })
}

fn arm_track_in_auto_arm_mode_ignoring_auto_arm() -> TestStep {
    step(
        AllVersions,
        "Arm track in auto-arm mode (ignoring auto-arm)",
        |reaper, step| {
            // Given
            let track = get_track(0)?;
            track.enable_auto_arm();
            check!(track.has_auto_arm_enabled());
            check!(!track.is_armed(true));
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.arm(false);
            // Then
            check!(track.is_armed(true));
            check!(track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), track);
            Ok(())
        },
    )
}

fn disarm_track_in_auto_arm_mode_ignoring_auto_arm() -> TestStep {
    step(
        AllVersions,
        "Disarm track in auto-arm mode (ignoring auto-arm)",
        |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.disarm(false);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), track);
            Ok(())
        },
    )
}

fn switch_track_to_auto_arm_mode_while_armed() -> TestStep {
    step(
        AllVersions,
        "Switch track to auto-arm mode while armed",
        |_, _| {
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
        },
    )
}

fn switch_to_normal_track_mode_while_armed() -> TestStep {
    step(
        AllVersions,
        "Switch to normal track mode while armed",
        |_, _| {
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
        },
    )
}

fn disable_track_auto_arm_mode() -> TestStep {
    step(AllVersions, "Disable track auto-arm mode", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.disable_auto_arm();
        // Then
        check!(!track.has_auto_arm_enabled());
        check!(!track.is_armed(true));
        check!(!track.is_armed(false));
        Ok(())
    })
}

fn disarm_track_in_auto_arm_mode() -> TestStep {
    step(
        AllVersions,
        "Disarm track in auto-arm mode",
        |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.disarm(true);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(track.has_auto_arm_enabled());
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), track);
            Ok(())
        },
    )
}

fn arm_track_in_auto_arm_mode() -> TestStep {
    step(AllVersions, "Arm track in auto-arm mode", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.arm(true);
        // Then
        check!(track.is_armed(true));
        // TODO Interesting! GetMediaTrackInfo_Value read with I_RECARM seems to support
        // auto-arm already! So maybe we should remove the chunk check and the
        // parameter supportAutoArm
        check!(track.is_armed(false));
        check!(track.has_auto_arm_enabled());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn enable_track_in_auto_arm_mode() -> TestStep {
    step(AllVersions, "Enable track auto-arm mode", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.enable_auto_arm();
        // Then
        check!(track.has_auto_arm_enabled());
        check!(!track.is_armed(true));
        check!(!track.is_armed(false));
        Ok(())
    })
}

fn disarm_track_in_normal_mode() -> TestStep {
    step(
        AllVersions,
        "Disarm track in normal mode",
        |reaper, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.disarm(true);
            // Then
            check!(!track.is_armed(true));
            check!(!track.is_armed(false));
            check!(!track.has_auto_arm_enabled());
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), track);
            Ok(())
        },
    )
}

fn arm_track_in_normal_mode() -> TestStep {
    step(AllVersions, "Arm track in normal mode", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.arm(true);
        // Then
        check!(track.is_armed(true));
        check!(track.is_armed(false));
        check!(!track.has_auto_arm_enabled());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn query_track_arm_state() -> TestStep {
    step(AllVersions, "Query track arm state", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let is_armed = track.is_armed(true);
        let is_armed_ignoring_auto_arm = track.is_armed(false);
        // Then
        check!(!is_armed);
        check!(!is_armed_ignoring_auto_arm);
        Ok(())
    })
}

fn query_track_auto_arm_mode() -> TestStep {
    step(AllVersions, "Query track auto arm mode", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let is_in_auto_arm_mode = track.has_auto_arm_enabled();
        // Then
        check!(!is_in_auto_arm_mode);
        Ok(())
    })
}

fn select_master_track() -> TestStep {
    step(AllVersions, "Select master track", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let master_track = project.get_master_track();
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        project.unselect_all_tracks();
        master_track.select();
        // Then
        check!(master_track.is_selected());
        check_eq!(
            project.get_selected_track_count(MasterTrackBehavior::IncludeMasterTrack),
            1
        );
        let first_selected_track = project
            .get_first_selected_track(MasterTrackBehavior::IncludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        check!(first_selected_track.is_master_track());
        check_eq!(
            project
                .get_selected_tracks(MasterTrackBehavior::IncludeMasterTrack)
                .count(),
            1
        );
        // TODO REAPER doesn't notify us about master track selection currently
        check_eq!(mock.get_invocation_count(), 1);
        let last_arg: Track = mock.get_last_arg().into();
        check_eq!(last_arg.get_index(), Some(2));
        Ok(())
    })
}

fn unselect_track() -> TestStep {
    step(AllVersions, "Unselect track", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unselect();
        // Then
        check!(!track.is_selected());
        check_eq!(
            project.get_selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            1
        );
        let first_selected_track = project
            .get_first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        check_eq!(first_selected_track.get_index(), Some(2));
        check_eq!(
            project
                .get_selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            1
        );
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn select_track() -> TestStep {
    step(AllVersions, "Select track", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        let track2 = project.get_track_by_index(2).ok_or("No track at index 2")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.select();
        track2.select();
        // Then
        check!(track.is_selected());
        check!(track2.is_selected());
        check_eq!(
            project.get_selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            2
        );
        let first_selected_track = project
            .get_first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        check_eq!(first_selected_track.get_index(), Some(0));
        check_eq!(
            project
                .get_selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            2
        );
        check_eq!(mock.get_invocation_count(), 2);
        check_eq!(mock.get_last_arg(), track2);
        Ok(())
    })
}

fn query_track_selection_state() -> TestStep {
    step(AllVersions, "Query track selection state", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let is_selected = track.is_selected();
        // Then
        check!(!is_selected);
        check_eq!(
            project.get_selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            0
        );
        Ok(())
    })
}

fn set_track_pan() -> TestStep {
    step(AllVersions, "Set track pan", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_pan_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_pan(Pan::from_normalized_value(0.25));
        // Then
        let pan = track.get_pan();
        check_eq!(pan.get_reaper_value(), ReaperPanValue::new(-0.5));
        check_eq!(pan.get_normalized_value(), 0.25);
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn query_track_pan() -> TestStep {
    step(AllVersions, "Query track pan", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let pan = track.get_pan();
        // Then
        check_eq!(pan.get_reaper_value(), ReaperPanValue::CENTER);
        check_eq!(pan.get_normalized_value(), 0.5);
        Ok(())
    })
}

fn set_track_volume() -> TestStep {
    step(AllVersions, "Set track volume", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_volume_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_volume(Volume::from_normalized_value(0.25));
        // Then
        let volume = track.get_volume();
        check_eq!(
            volume.get_reaper_value(),
            ReaperVolumeValue::new(0.031588093366685013)
        );
        check_eq!(volume.get_db(), Db::new(-30.009531739774296));
        check_eq!(volume.get_normalized_value(), 0.25000000000003497);
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn set_track_volume_extreme_values() -> TestStep {
    step(
        AllVersions,
        "Set track volume extreme values",
        |reaper, _| {
            // Given
            let track_1 = get_track(0)?;
            let track_2 = get_track(1)?;
            // When
            let track_1_result = unsafe {
                reaper.medium().functions().csurf_on_volume_change_ex(
                    track_1.get_raw(),
                    ValueChange::Absolute(ReaperVolumeValue::new(1.0 / 0.0)),
                    GangBehavior::DenyGang,
                );
                reaper
                    .medium()
                    .functions()
                    .get_track_ui_vol_pan(track_1.get_raw())
                    .unwrap()
            };
            let track_2_result = unsafe {
                reaper.medium().functions().csurf_on_volume_change_ex(
                    track_2.get_raw(),
                    ValueChange::Absolute(ReaperVolumeValue::new(f64::NAN)),
                    GangBehavior::DenyGang,
                );
                reaper
                    .medium()
                    .functions()
                    .get_track_ui_vol_pan(track_2.get_raw())
                    .unwrap()
            };
            // Then
            check_eq!(track_1_result.volume, ReaperVolumeValue::new(1.0 / 0.0));
            let track_1_volume = Volume::from_reaper_value(track_1_result.volume);
            check_eq!(track_1_volume.get_db(), Db::new(1.0 / 0.0));
            check_eq!(track_1_volume.get_normalized_value(), 1.0 / 0.0);
            check_eq!(
                track_1_volume.get_reaper_value(),
                ReaperVolumeValue::new(1.0 / 0.0)
            );

            check!(track_2_result.volume.get().is_nan());
            let track_2_volume = Volume::from_reaper_value(track_2_result.volume);
            check!(track_2_volume.get_db().get().is_nan());
            check!(track_2_volume.get_normalized_value().is_nan());
            check!(track_2_volume.get_reaper_value().get().is_nan());
            Ok(())
        },
    )
}

fn query_track_volume() -> TestStep {
    step(AllVersions, "Query track volume", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let volume = track.get_volume();
        // Then
        check_eq!(volume.get_reaper_value(), ReaperVolumeValue::ZERO_DB);
        check_eq!(volume.get_db(), Db::ZERO_DB);
        check_eq!(volume.get_normalized_value(), 0.71599999999999997);
        Ok(())
    })
}

fn set_track_recording_input_midi_all_15() -> TestStep {
    step(
        AllVersions,
        "Set track recording input MIDI all/15",
        |_, _| {
            // Given
            let track = get_track(0)?;
            let given_input = Some(RecordingInput::Midi {
                device_id: None,
                channel: Some(channel(15)),
            });
            // When
            track.set_recording_input(given_input);
            // Then
            check_eq!(track.get_recording_input(), given_input);
            Ok(())
        },
    )
}

fn set_track_recording_input_midi_7_all() -> TestStep {
    step(
        AllVersions,
        "Set track recording input MIDI 7/all",
        |_, _| {
            // Given
            let track = get_track(0)?;
            let given_input = Some(RecordingInput::Midi {
                device_id: Some(MidiInputDeviceId::new(7)),
                channel: None,
            });
            // When
            track.set_recording_input(given_input);
            // Then
            check_eq!(track.get_recording_input(), given_input);
            Ok(())
        },
    )
}

fn set_track_recording_input_midi_4_5() -> TestStep {
    step(AllVersions, "Set track recording input MIDI 4/5", |_, _| {
        // Given
        let track = get_track(0)?;
        let given_input = Some(RecordingInput::Midi {
            device_id: Some(MidiInputDeviceId::new(4)),
            channel: Some(channel(5)),
        });
        // When
        track.set_recording_input(given_input);
        // Then
        check_eq!(track.get_recording_input(), given_input);
        Ok(())
    })
}

fn set_track_recording_input_midi_all_all() -> TestStep {
    step(
        AllVersions,
        "Set track recording input MIDI all/all",
        |reaper, step| {
            // Given
            let track = get_track(0)?;
            let given_input = Some(RecordingInput::Midi {
                device_id: None,
                channel: None,
            });
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .track_input_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.set_recording_input(given_input);
            // Then
            let input = track.get_recording_input();
            check_eq!(input, given_input);
            let input = input.unwrap();
            check_eq!(u32::from(input), 6112);
            check_eq!(RecordingInput::try_from(6112 as u32), Ok(input));
            // TODO-high Search in project for 5198273 for a hacky way to solve this
            check_eq!(mock.get_invocation_count(), 0);
            // check_eq!(mock.get_last_arg(), track);
            Ok(())
        },
    )
}

fn query_track_recording_input() -> TestStep {
    step(AllVersions, "Query track recording input", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let input = track.get_recording_input();
        // Then
        match input {
            Some(RecordingInput::Mono(0)) => Ok(()),
            _ => Err("Expected MidiRecordingInput".into()),
        }
    })
}

fn set_track_input_monitoring() -> TestStep {
    step(AllVersions, "Set track input monitoring", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_input_monitoring_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_input_monitoring_mode(InputMonitoringMode::NotWhenPlaying);
        // Then
        check_eq!(
            track.get_input_monitoring_mode(),
            InputMonitoringMode::NotWhenPlaying
        );
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn query_track_input_monitoring() -> TestStep {
    step(AllVersions, "Query track input monitoring", |reaper, _| {
        // Given
        let track = get_track(0)?;
        // When
        let mode = track.get_input_monitoring_mode();
        // Then
        use InputMonitoringMode::*;
        if reaper.get_version() < ReaperVersion::from("6") {
            check_eq!(mode, Off);
        } else {
            check_eq!(mode, Normal);
        }
        Ok(())
    })
}

fn set_track_name() -> TestStep {
    step(AllVersions, "Set track name", |reaper, step| {
        // Given
        let track = get_track(0)?;
        // When
        // TODO Factor this state pattern out
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_name_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_name(c_str!("Foo Bla"));
        // Then
        check_eq!(track.get_name(), c_str!("Foo Bla").to_owned());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), track);
        Ok(())
    })
}

fn query_track_name() -> TestStep {
    step(AllVersions, "Query track name", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let track_name = track.get_name();
        // Then
        check_eq!(track_name.as_bytes().len(), 0);
        Ok(())
    })
}

fn query_track_project() -> TestStep {
    step(AllVersions, "Query track project", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        let track = get_track(0)?;
        // When
        let track_project = track.get_project();
        // Then
        check_eq!(track_project, project);
        Ok(())
    })
}

fn query_non_existent_track_by_guid() -> TestStep {
    step(
        AllVersions,
        "Query non-existent track by GUID",
        |reaper, _| {
            // Given
            let project = reaper.get_current_project();
            // When
            let guid = Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            let found_track = project.get_track_by_guid(&guid);
            // Then
            check!(!found_track.is_available());
            Ok(())
        },
    )
}

fn query_track_by_guid() -> TestStep {
    step(AllVersions, "Query track by GUID", |reaper, _| {
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
        check_eq!(
            new_track.get_guid(),
            &get_media_track_guid(new_track.get_raw())
        );
        Ok(())
    })
}

fn query_all_tracks() -> TestStep {
    step(AllVersions, "Query all tracks", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        project.add_track();
        // When
        let tracks = project.get_tracks();
        // Then
        check_eq!(tracks.count(), 2);
        Ok(())
    })
}

fn query_master_track() -> TestStep {
    step(AllVersions, "Query master track", |reaper, _| {
        // Given
        let project = reaper.get_current_project();
        // When
        let master_track = project.get_master_track();
        // Then
        check!(master_track.is_master_track());
        Ok(())
    })
}

fn fn_mut_action() -> TestStep {
    #[allow(unreachable_code)]
    step(AllVersions, "FnMut action", |_reaper, _| {
        // TODO-low Add this as new test
        return Ok(());
        let mut i = 0;
        let _action1 = _reaper.register_action(
            c_str!("reaperRsCounter"),
            c_str!("reaper-rs counter"),
            move || {
                let owned = format!("Hello from Rust number {}\0", i);
                let reaper = Reaper::get();
                reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap());
                i += 1;
            },
            ActionKind::NotToggleable,
        );
        Ok(())
    })
}

fn add_track() -> TestStep {
    step(AllVersions, "Add track", |reaper, step| {
        // Given
        let project = reaper.get_current_project();
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .track_added()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let new_track = project.add_track();
        // Then
        check_eq!(project.get_track_count(), 1);
        check_eq!(new_track.get_index(), Some(0));
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), new_track);
        Ok(())
    })
}

fn create_empty_project_in_new_tab() -> TestStep {
    step(
        AllVersions,
        "Create empty project in new tab",
        |reaper, step| {
            // Given
            let current_project_before = reaper.get_current_project();
            let project_count_before = reaper.get_project_count();
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .project_switched()
                    .take_until(step.finished)
                    .subscribe(move |p| {
                        mock.invoke(p);
                    });
            });
            let new_project = reaper.create_empty_project_in_new_tab();
            // Then
            check_eq!(current_project_before, current_project_before);
            check_eq!(reaper.get_project_count(), project_count_before + 1);
            check_eq!(
                reaper.get_projects().count() as u32,
                project_count_before + 1
            );
            check_ne!(reaper.get_current_project(), current_project_before);
            check_eq!(reaper.get_current_project(), new_project);
            check_ne!(reaper.get_projects().nth(0), Some(new_project));
            //
            // assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().first() ==
            // newProject);
            // assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().count() ==
            // projectCountBefore + 1);
            check_eq!(new_project.get_track_count(), 0);
            check!(new_project.get_index() > 0);
            check!(new_project.get_file_path().is_none());
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), new_project);
            Ok(())
        },
    )
}

fn basics() -> TestStep {
    step(AllVersions, "Basics", |reaper, _| {
        check!(Guid::try_from(c_str!("{hey}")).is_err());
        reaper.show_console_msg(c_str!("Test string types and encoding:\n"));
        reaper.show_console_msg(c_str!("- &CStr: 范例文字äöüß\n"));
        reaper.show_console_msg("- &str: 范例文字äöüß\n");
        reaper.show_console_msg(String::from("- String: 范例文字äöüß"));
        Ok(())
    })
}

type GetFxChain = Rc<dyn Fn() -> Result<FxChain, &'static str>>;

fn query_fx_chain(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Query fx chain", move |_, _| {
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
        check!(
            !fx_chain
                .get_fx_by_guid_and_index(&non_existing_guid, 0)
                .is_available()
        );
        check!(fx_chain.get_first_fx_by_name(c_str!("bla")).is_none());
        check!(fx_chain.get_chunk().is_none());
        Ok(())
    })
}
fn create_fx_steps(
    prefix: &'static str,
    get_fx_chain: impl Fn() -> Result<FxChain, &'static str> + 'static + Copy,
) -> impl Iterator<Item = TestStep> {
    let get_fx_chain = Rc::new(get_fx_chain);
    let steps = vec![
        query_fx_chain(get_fx_chain.clone()),
        add_track_fx_by_original_name(get_fx_chain.clone()),
        check_track_fx_with_1_fx(get_fx_chain.clone()),
        disable_track_fx(get_fx_chain.clone()),
        enable_track_fx(get_fx_chain.clone()),
        check_track_fx_with_2_fx(get_fx_chain.clone()),
        check_fx_parameter(get_fx_chain.clone()),
        check_fx_presets(get_fx_chain.clone()),
        set_fx_parameter_value(get_fx_chain.clone()),
        fx_parameter_value_changed_with_heuristic_fail(get_fx_chain.clone()),
        move_fx(get_fx_chain.clone()),
        remove_fx(get_fx_chain.clone()),
        add_fx_by_chunk(get_fx_chain.clone()),
        set_fx_chunk(get_fx_chain.clone()),
        set_fx_tag_chunk(get_fx_chain.clone()),
        set_fx_state_chunk(get_fx_chain.clone()),
        set_fx_chain_chunk(get_fx_chain.clone()),
        query_fx_floating_window(get_fx_chain.clone()),
        show_fx_in_floating_window(get_fx_chain.clone()),
        add_track_js_fx_by_original_name(get_fx_chain.clone()),
        query_track_js_fx_by_index(get_fx_chain.clone()),
    ];
    steps.into_iter().map(move |s| TestStep {
        name: format!("{} - {}", prefix, s.name).into(),
        ..s
    })
}

fn query_track_js_fx_by_index(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Query track JS fx by index",
        move |reaper, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.get_track();
            // When
            let fx = fx_chain.get_fx_by_index(2);
            // Then
            let fx = fx.ok_or("No FX found")?;
            check!(fx.is_available());
            check_eq!(fx.get_index(), 2);
            check_eq!(
                i32::from(fx.get_query_index()),
                if fx_chain.is_input_fx() { 0x1000002 } else { 2 }
            );
            check!(fx.get_guid().is_some());
            check_eq!(fx.get_name().as_c_str(), c_str!("JS: phaser"));
            let fx_chunk = fx.get_chunk();
            check!(fx_chunk.starts_with("BYPASS 0 0 0"));
            if reaper.get_version() < ReaperVersion::from("6") {
                check!(fx_chunk.ends_with("\nWAK 0"));
            } else {
                check!(fx_chunk.ends_with("\nWAK 0 0"));
            }
            let tag_chunk = fx.get_tag_chunk();
            check!(tag_chunk.starts_with(r#"<JS phaser """#));
            check!(tag_chunk.ends_with("\n>"));
            let state_chunk = fx.get_state_chunk();
            check!(!state_chunk.contains("<"));
            check!(!state_chunk.contains(">"));
            check_eq!(fx.get_track(), track);
            check_eq!(fx.is_input_fx(), fx_chain.is_input_fx());
            check_eq!(fx.get_chain(), fx_chain);
            check_eq!(fx.get_parameter_count(), 7);
            check_eq!(fx.get_parameters().count(), 7);
            let param1 = fx.get_parameter_by_index(0);
            check!(param1.is_available());
            // TODO-low Fix for input FX (there it's 1.0 for some reason)
            // check_eq!(param1.get_step_size(), Some(0.01));
            check_eq!(
                param1.get_value_range(),
                FxParameterValueRange {
                    min_val: 0.0,
                    mid_val: 5.0,
                    max_val: 10.0
                }
            );
            check!(fx.get_parameter_by_index(6).is_available());
            check!(!fx.get_parameter_by_index(7).is_available());
            let fx_info = fx.get_info();
            let stem = fx_info.file_name.file_stem().ok_or("No stem")?;
            check_eq!(stem, "phaser");
            Ok(())
        },
    )
}

fn add_track_js_fx_by_original_name(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Add track JS fx by original name",
        move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_added()
                    .take_until(step.finished.clone())
                    .subscribe(move |fx| {
                        mock.invoke(fx);
                    });
            });
            let fx = fx_chain.add_fx_by_original_name(c_str!("phaser"));
            // Then
            let fx = fx.ok_or("No FX added")?;
            check_eq!(fx_chain.get_fx_count(), 3);
            check_eq!(fx_chain.get_fx_by_index(2), Some(fx.clone()));
            check_eq!(fx_chain.get_last_fx(), Some(fx.clone()));
            let fx_guid = fx.get_guid().ok_or("No GUID")?;
            check!(fx_chain.get_fx_by_guid(&fx_guid).is_available());
            let guid = Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            check!(!fx_chain.get_fx_by_guid_and_index(&guid, 0).is_available());
            check!(
                fx_chain
                    .get_first_fx_by_name(c_str!("ReaControlMIDI (Cockos)"))
                    .is_some()
            );
            check_eq!(
                fx_chain.get_first_fx_by_name(c_str!("phaser")),
                Some(fx.clone())
            );
            if reaper.get_version() < ReaperVersion::from("6") {
                // Mmh
                if fx_chain.is_input_fx() {
                    check_eq!(mock.get_invocation_count(), 2);
                } else {
                    check_eq!(mock.get_invocation_count(), 3);
                }
            } else {
                check_eq!(mock.get_invocation_count(), 1);
                check_eq!(mock.get_last_arg(), fx.clone());
            }
            Ok(())
        },
    )
}

fn show_fx_in_floating_window(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Show fx in floating window",
        move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx = fx_chain
                .get_fx_by_index(0)
                .ok_or("Couldn't find first fx")?;
            // When
            let (fx_opened_mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_opened()
                    .take_until(step.finished.clone())
                    .subscribe(move |fx| {
                        mock.invoke(fx);
                    });
            });
            let (fx_focused_mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_focused()
                    .take_until(step.finished)
                    .subscribe(move |fx| {
                        mock.invoke(fx);
                    });
            });
            fx.show_in_floating_window();
            // Then
            check!(fx.get_floating_window().is_some());
            check!(fx.window_is_open());
            check!(fx.window_has_focus());
            check!(fx_opened_mock.get_invocation_count() >= 1);
            if !fx_chain.is_input_fx() || reaper.get_version() >= ReaperVersion::from("5.95") {
                // In previous versions it wrongly reports as normal FX
                check_eq!(fx_opened_mock.get_last_arg(), fx);
            }
            check_eq!(fx_focused_mock.get_invocation_count(), 0);
            // Should be > 0 but doesn't work
            check!(reaper.get_focused_fx().is_none()); // Should be Some but doesn't work
            Ok(())
        },
    )
}

fn query_fx_floating_window(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Query fx floating window", move |reaper, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // When
        // Then
        check!(fx.get_floating_window().is_none());
        check!(!fx.window_is_open());
        check!(!fx.window_has_focus());
        check!(reaper.get_focused_fx().is_none());
        Ok(())
    })
}

fn set_fx_chain_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx chain chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let track = fx_chain.get_track();
        let other_fx_chain = if fx_chain.is_input_fx() {
            track.get_normal_fx_chain()
        } else {
            track.get_input_fx_chain()
        };
        let fx_chain_chunk = format!(
            "{}{}",
            if fx_chain.is_input_fx() {
                "<FXCHAIN"
            } else {
                "<FXCHAIN_REC"
            },
            r#"
SHOW 0
LASTSEL 0
DOCKED 0
BYPASS 0 0 0
<VST "VST: ReaControlMIDI (Cockos)" reacontrolmidi.dll 0 "" 1919118692
ZG1jcu5e7f4AAAAAAAAAAOYAAAABAAAAAAAQAA==
/////wAAAAAAAAAAAAAAAAkAAAAMAAAAAQAAAP8/AAAAIAAAACAAAAAAAAA1AAAAQzpcVXNlcnNcYmtsdW1cQXBwRGF0YVxSb2FtaW5nXFJFQVBFUlxEYXRhXEdNLnJlYWJhbmsAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYAAABNYWpvcgANAAAAMTAyMDM0MDUwNjA3AAEAAAAAAAAAAAAAAAAKAAAA
DQAAAAEAAAAAAAAAAAAAAAAAAAA=
AAAQAAAA
>
FLOATPOS 0 0 0 0
FXID {80028901-3762-477F-BE48-EA8324C178AA}
WAK 0
BYPASS 0 0 0
<VST "VSTi: ReaSynth (Cockos)" reasynth.dll 0 "" 1919251321
eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
AAAQAAAA
>
FLOATPOS 0 0 0 0
FXID {5FF5FB09-9102-4CBA-A3FB-3467BA1BFEAA}
WAK 0
>
"#
        );
        // When
        other_fx_chain.set_chunk(fx_chain_chunk.as_str());
        // Then
        check_eq!(other_fx_chain.get_fx_count(), 2);
        check_eq!(fx_chain.get_fx_count(), 2);
        Ok(())
    })
}

fn set_fx_state_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx state chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.get_fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain
            .get_fx_by_index(1)
            .ok_or("Couldn't find synth fx")?;
        let synth_param_5 = synth_fx.get_parameter_by_index(5);
        synth_param_5.set_normalized_value(ReaperNormalizedValue::new(0.0));
        check_ne!(
            synth_param_5.get_formatted_value().as_c_str(),
            c_str!("-6.00")
        );
        let fx_state_chunk = r#"eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
  776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
  AAAQAAAA"#;
        // When
        synth_fx.set_state_chunk(fx_state_chunk);
        // Then
        check_eq!(synth_fx.get_index(), 1);
        check_eq!(
            synth_fx.get_name().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        check_eq!(
            synth_param_5.get_formatted_value().as_c_str(),
            c_str!("-6.00")
        );
        check_eq!(midi_fx.get_index(), 0);
        check_eq!(
            midi_fx.get_name().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        Ok(())
    })
}

fn set_fx_tag_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx tag chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx_1 = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find MIDI fx 1")?;
        let midi_fx_2 = fx_chain
            .get_fx_by_index(1)
            .ok_or("Couldn't find MIDI fx 2")?;
        let fx_tag_chunk = r#"<VST "VSTi: ReaSynth (Cockos)" reasynth.dll 0 "" 1919251321
  eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
  776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
  AAAQAAAA
  >"#;
        // When
        midi_fx_2.set_tag_chunk(fx_tag_chunk);
        // Then
        check_eq!(midi_fx_2.get_index(), 1);
        check_eq!(
            midi_fx_2.get_name().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        check_eq!(midi_fx_1.get_index(), 0);
        check_eq!(
            midi_fx_1.get_name().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        Ok(())
    })
}

fn set_fx_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.get_fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain
            .get_fx_by_index(1)
            .ok_or("Couldn't find synth fx")?;
        let synth_fx_guid_before = synth_fx.get_guid();
        // When
        synth_fx.set_chunk(midi_fx.get_chunk());
        // Then
        check_eq!(synth_fx.get_guid(), synth_fx_guid_before);
        check!(synth_fx.is_available());
        check_eq!(
            synth_fx.get_name().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        check_eq!(midi_fx.get_index(), 0);
        check_eq!(synth_fx.get_index(), 1);
        Ok(())
    })
}

fn add_fx_by_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Add FX by chunk", move |reaper, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx_chunk = r#"BYPASS 0 0 0
<VST "VSTi: ReaSynth (Cockos)" reasynth.dll 0 "" 1919251321
eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
AAAQAAAA
>
FLOATPOS 0 0 0 0
FXID {5FF5FB09-9102-4CBA-A3FB-3467BA1BFE5D}
WAK 0
"#;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .fx_added()
                .take_until(step.finished)
                .subscribe(move |fx| {
                    mock.invoke(fx);
                });
        });
        let synth_fx = fx_chain.add_fx_from_chunk(fx_chunk);
        // Then
        let synth_fx = synth_fx.ok_or("Didn't return FX")?;
        check_eq!(synth_fx.get_index(), 1);
        let guid = Guid::try_from(c_str!("{5FF5FB09-9102-4CBA-A3FB-3467BA1BFE5D}"))?;
        check_eq!(synth_fx.get_guid(), Some(guid));
        check_eq!(
            synth_fx
                .get_parameter_by_index(5)
                .get_formatted_value()
                .as_c_str(),
            c_str!("-6.00")
        );
        // TODO Detect such a programmatic FX add as well (maybe by hooking into
        // HelperControlSurface::updateMediaTrackPositions)
        check_eq!(mock.get_invocation_count(), 0);
        Ok(())
    })
}

fn remove_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Remove FX", move |reaper, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let synth_fx = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find synth fx")?;
        let midi_fx = fx_chain.get_fx_by_index(1).ok_or("Couldn't find MIDI fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .fx_removed()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        fx_chain.remove_fx(&synth_fx);
        // Then
        check!(!synth_fx.is_available());
        check!(midi_fx.is_available());
        check_eq!(midi_fx.get_index(), 0);
        midi_fx.invalidate_index();
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), synth_fx);
        Ok(())
    })
}

fn move_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Move FX", move |reaper, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.get_fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain
            .get_fx_by_index(1)
            .ok_or("Couldn't find synth fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .fx_reordered()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        fx_chain.move_fx(&synth_fx, 0);
        // Then
        check_eq!(midi_fx.get_index(), 1);
        check_eq!(synth_fx.get_index(), 0);
        if reaper.get_version() < ReaperVersion::from("5.95") {
            check_eq!(mock.get_invocation_count(), 0);
        } else {
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), fx_chain.get_track());
        }
        Ok(())
    })
}

fn fx_parameter_value_changed_with_heuristic_fail(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "fxParameterValueChanged with heuristic fail in REAPER < 5.95",
        move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx = fx_chain.get_fx_by_index(0).ok_or("Couldn't find fx")?;
            let p = fx.get_parameter_by_index(0);
            p.set_normalized_value(ReaperNormalizedValue::new(0.5));
            let other_fx_chain = if fx_chain.is_input_fx() {
                fx.get_track().get_normal_fx_chain()
            } else {
                fx.get_track().get_input_fx_chain()
            };
            let fx_on_other_fx_chain = other_fx_chain
                .add_fx_by_original_name(c_str!("ReaControlMIDI (Cockos)"))
                .expect("Couldn't find FX on other FX chain");
            let p_on_other_fx_chain = fx_on_other_fx_chain.get_parameter_by_index(0);
            // First set parameter on other FX chain to same value (confuses heuristic if
            // fxChain is input FX chain)
            p_on_other_fx_chain.set_normalized_value(ReaperNormalizedValue::new(0.5));
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_parameter_value_changed()
                    .take_until(step.finished)
                    .subscribe(move |p| {
                        mock.invoke(p);
                    });
            });
            p.set_normalized_value(ReaperNormalizedValue::new(0.5));
            // Then
            check_eq!(mock.get_invocation_count(), 2);
            if fx_chain.is_input_fx() && reaper.get_version() < ReaperVersion::from(c_str!("5.95"))
            {
                check_ne!(mock.get_last_arg(), p);
            } else {
                check_eq!(mock.get_last_arg(), p);
            }
            Ok(())
        },
    )
}

fn set_fx_parameter_value(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Set fx parameter value",
        move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx = fx_chain.get_fx_by_index(1).ok_or("Couldn't find fx")?;
            let p = fx.get_parameter_by_index(5);
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_parameter_value_changed()
                    .take_until(step.finished)
                    .subscribe(move |p| {
                        mock.invoke(p);
                    });
            });
            p.set_normalized_value(ReaperNormalizedValue::new(0.3));
            // Then
            let last_touched_fx_param = reaper.get_last_touched_fx_parameter();
            if fx_chain.is_input_fx() && reaper.get_version() < ReaperVersion::from(c_str!("5.95"))
            {
                check!(last_touched_fx_param.is_none());
            } else {
                check_eq!(last_touched_fx_param, Some(p.clone()));
            }
            check_eq!(p.get_formatted_value().as_c_str(), c_str!("-4.44"));
            check_eq!(
                p.get_normalized_value(),
                ReaperNormalizedValue::new(0.30000001192092896)
            );
            check_eq!(
                p.get_reaper_value(),
                ReaperNormalizedValue::new(0.30000001192092896)
            );
            check_eq!(
                p.format_normalized_value(p.get_normalized_value())
                    .as_c_str(),
                c_str!("-4.44 dB")
            );
            if reaper.get_version() < ReaperVersion::from("6") {
                if fx_chain.is_input_fx() {
                    // Mmh
                    check_eq!(mock.get_invocation_count(), 2);
                } else {
                    check_eq!(mock.get_invocation_count(), 1);
                }
            } else {
                // TODO-low 1 invocation would be better than 2 (in v6 it gives us 2)
                check_eq!(mock.get_invocation_count(), 2);
            }
            check_eq!(mock.get_last_arg(), p);
            Ok(())
        },
    )
}

fn check_fx_presets(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check fx presets", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // When
        // Then
        check_eq!(fx.get_preset_count(), 0);
        check!(fx.get_preset_name().is_none());
        check!(fx.preset_is_dirty());
        Ok(())
    })
}

fn check_fx_parameter(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check fx parameter", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // When
        let p = fx.get_parameter_by_index(5);
        // Then
        check!(p.is_available());
        check_eq!(p.get_name().as_c_str(), c_str!("Pitch Wheel"));
        check_eq!(p.get_index(), 5);
        check_eq!(p.get_character(), FxParameterCharacter::Continuous);
        check_eq!(p.clone(), p);
        check_eq!(p.get_formatted_value().as_c_str(), c_str!("0"));
        check_eq!(p.get_normalized_value(), ReaperNormalizedValue::new(0.5));
        check_eq!(p.get_reaper_value(), ReaperNormalizedValue::new(0.5));
        check_eq!(
            p.format_normalized_value(p.get_normalized_value())
                .as_c_str(),
            c_str!("0")
        );
        check_eq!(p.get_fx(), fx);
        check!(p.get_step_size().is_none());
        check_eq!(
            p.get_value_range(),
            FxParameterValueRange {
                min_val: 0.0,
                mid_val: 0.5,
                max_val: 1.0
            }
        );
        Ok(())
    })
}

fn check_track_fx_with_2_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check track fx with 2 fx", move |reaper, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let track = fx_chain.get_track();
        // When
        let fx_1 = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        let fx_2 = fx_chain
            .add_fx_by_original_name(c_str!("ReaSynth (Cockos)"))
            .ok_or("Couldn't add ReaSynth")?;
        // Then
        check!(fx_1.is_available());
        check!(fx_2.is_available());
        check_eq!(fx_1.get_index(), 0);
        check_eq!(fx_2.get_index(), 1);
        check_eq!(
            i32::from(fx_1.get_query_index()),
            if fx_chain.is_input_fx() { 0x1000000 } else { 0 }
        );
        check_eq!(
            i32::from(fx_2.get_query_index()),
            if fx_chain.is_input_fx() { 0x1000001 } else { 1 }
        );
        check!(fx_1.get_guid().is_some());
        check!(fx_2.get_guid().is_some());
        check_eq!(
            fx_1.get_name().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        check_eq!(
            fx_2.get_name().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        let chunk_1 = fx_1.get_chunk();
        check!(chunk_1.starts_with("BYPASS 0 0 0"));
        if reaper.get_version() < ReaperVersion::from("6") {
            check!(chunk_1.ends_with("\nWAK 0"));
        } else {
            check!(chunk_1.ends_with("\nWAK 0 0"));
        }
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
        check!(
            track
                .get_fx_by_query_index(if fx_chain.is_input_fx() { 0x1000000 } else { 0 })
                .is_some()
        );
        check!(
            track
                .get_fx_by_query_index(if fx_chain.is_input_fx() { 0x1000001 } else { 1 })
                .is_some()
        );
        check!(
            !track
                .get_fx_by_query_index(if fx_chain.is_input_fx() { 0 } else { 0x1000000 })
                .is_some()
        );
        check!(
            !track
                .get_fx_by_query_index(if fx_chain.is_input_fx() { 1 } else { 0x1000001 })
                .is_some()
        );
        if !fx_chain.is_input_fx() {
            let first_instrument_fx = fx_chain
                .get_first_instrument_fx()
                .ok_or("Couldn't find instrument FX")?;
            check_eq!(first_instrument_fx.get_index(), 1);
        }
        Ok(())
    })
}

fn enable_track_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Enable track fx", move |reaper, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx_1 = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .fx_enabled_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        fx_1.enable();
        // Then
        check!(fx_1.is_enabled());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), fx_1);
        Ok(())
    })
}

fn disable_track_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Disable track fx", move |reaper, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx_1 = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            reaper
                .fx_enabled_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        fx_1.disable();
        // Then
        check!(!fx_1.is_enabled());
        check_eq!(mock.get_invocation_count(), 1);
        check_eq!(mock.get_last_arg(), fx_1);
        Ok(())
    })
}

fn check_track_fx_with_1_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check track fx with 1 fx", move |reaper, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let track = fx_chain.get_track();
        // When
        let fx_1 = fx_chain
            .get_fx_by_index(0)
            .ok_or("Couldn't find first fx")?;
        // Then
        check!(fx_1.is_available());
        check_eq!(fx_1.get_index(), 0);
        check_eq!(
            i32::from(fx_1.get_query_index()),
            if fx_chain.is_input_fx() { 0x1000000 } else { 0 }
        );
        check!(fx_1.get_guid().is_some());
        check_eq!(
            fx_1.get_name().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        let chunk = fx_1.get_chunk();
        check!(chunk.starts_with("BYPASS 0 0 0"));
        if reaper.get_version() < ReaperVersion::from("6") {
            check!(chunk.ends_with("\nWAK 0"));
        } else {
            check!(chunk.ends_with("\nWAK 0 0"));
        }
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
    })
}

fn add_track_fx_by_original_name(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Add track fx by original name",
        move |reaper, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                reaper
                    .fx_added()
                    .take_until(step.finished)
                    .subscribe(move |t| {
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
            let non_existing_guid =
                Guid::try_from(c_str!("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}"))?;
            check!(
                !fx_chain
                    .get_fx_by_guid_and_index(&non_existing_guid, 0)
                    .is_available()
            );
            check_eq!(
                fx_chain.get_first_fx_by_name(c_str!("ReaControlMIDI (Cockos)")),
                Some(fx.clone())
            );
            let chain_chunk = fx_chain.get_chunk();
            check!(chain_chunk.is_some());
            let chain_chunk = chain_chunk.unwrap();
            check!(chain_chunk.starts_with("<FXCHAIN"));
            check!(chain_chunk.ends_with("\n>"));
            let first_tag = chain_chunk.find_first_tag(0);
            check!(first_tag.is_some());
            let first_tag = first_tag.unwrap();
            check_eq!(
                first_tag.get_content().deref(),
                chain_chunk.get_content().deref()
            );
            check_eq!(mock.get_invocation_count(), 1);
            check_eq!(mock.get_last_arg(), fx);
            Ok(())
        },
    )
}

fn get_track(index: u32) -> Result<Track, &'static str> {
    Reaper::get()
        .get_current_project()
        .get_track_by_index(index)
        .ok_or("Track not found")
}
