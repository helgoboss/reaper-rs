#![allow(clippy::float_cmp)]
use approx::*;

use std::iter;
use std::ops::Deref;

use c_str_macro::c_str;

use reaper_high::{
    get_media_track_guid, toggleable, ActionCharacter, ActionKind, FxChain, FxParameterCharacter,
    FxParameterValueRange, Guid, Pan, PlayRate, Reaper, SendPartnerType, Tempo, Track,
    TrackRoutePartner, Volume, Width,
};
use rxrust::prelude::*;

use crate::api::{step, Test, TestStep};

use super::invocation_mock::observe_invocations;
use crate::api::VersionRestriction::AllVersions;
use helgoboss_midi::test_util::{channel, key_number, u7};
use helgoboss_midi::{RawShortMessage, ShortMessageFactory};

use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::{
    reaper_str, AutoSeekBehavior, AutomationMode, Bpm, CommandId, Db, DurationInSeconds,
    FxPresetRef, GangBehavior, InputMonitoringMode, MasterTrackBehavior, MidiInputDeviceId,
    MidiOutputDeviceId, NormalizedPlayRate, PlaybackSpeedFactor, PositionInSeconds,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperVersion, ReaperVolumeValue,
    ReaperWidthValue, RecordingInput, SoloMode, StuffMidiMessageTarget, TrackLocation,
    UndoBehavior, ValueChange,
};

use reaper_low::{raw, Swell};
use reaper_rx::ActionRxProvider;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use std::rc::Rc;

/// Creates all integration test steps to be executed. The order matters!
pub fn create_test_steps() -> impl Iterator<Item = TestStep> {
    // In theory all steps could be declared inline. But that makes the IDE become terribly slow.
    let steps_a = vec![
        global_instances(),
        query_prefs(),
        register_api_functions(),
        strings(),
        low_plugin_context(),
        medium_plugin_context(),
        general(),
        volume_types(),
        create_empty_project_in_new_tab(),
        play_pause_stop_record(),
        change_repeat_state(),
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
        query_track_width(),
        set_track_pan(),
        set_track_width(),
        disable_all_track_fx(),
        enable_all_track_fx(),
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
        query_track_misc(),
        query_track_route_count(),
        add_track_send(),
        query_track_send(),
        set_track_send_volume(),
        set_track_send_pan(),
        set_track_send_mute(),
        query_time_ranges(),
        set_time_ranges(),
        query_action(),
        invoke_action(),
        test_action_invoked_event(),
        unmute_track(),
        mute_track(),
        solo_track(),
        unsolo_track(),
        solo_track_in_place(),
        unsolo_track(),
        generate_guid(),
        main_section_functions(),
        register_and_unregister_action(),
        register_and_unregister_toggle_action(),
    ]
    .into_iter();
    let steps_b = vec![
        insert_track_at(),
        scroll_mixer(),
        query_midi_input_devices(),
        query_midi_output_devices(),
        stuff_midi_devices(),
        use_undoable(),
        undo(),
        redo(),
        get_reaper_window(),
        mark_project_as_dirty(),
        get_project_play_rate(),
        set_project_play_rate(),
        get_project_tempo(),
        set_project_tempo(),
        swell(),
        metrics(),
    ]
    .into_iter();
    let output_fx_steps = create_fx_steps("Output FX chain", || {
        get_track(0).map(|t| t.normal_fx_chain())
    });
    let input_fx_steps = create_fx_steps("Input FX chain", || {
        get_track(1).map(|t| t.input_fx_chain())
    });
    iter::empty()
        .chain(steps_a)
        .chain(output_fx_steps)
        .chain(input_fx_steps)
        .chain(steps_b)
}

fn swell() -> TestStep {
    step(AllVersions, "SWELL", |_session, _| {
        let swell = Swell::load(*Reaper::get().medium_reaper().low().plugin_context());
        if cfg!(target_family = "windows") {
            assert!(swell.pointers().MessageBox.is_none());
            Ok(())
        } else {
            assert!(swell.pointers().MessageBox.is_some());
            Ok(())
        }
        // TODO-low At some point we might be okay with interactive tests
        // swell.MessageBox(
        //     null_mut(),
        //     c_str!("Hello world from SWELL").as_ptr(),
        //     c_str!("reaper-rs SWELL").as_ptr(),
        //     1,
        // );
    })
}

fn metrics() -> TestStep {
    step(AllVersions, "Metrics", |_session, _| {
        // TODO-low Log as yaml (and only the metrics - put it behind feature gate)
        println!(
            "reaper_medium::Reaper metrics after integration test: {:#?}",
            Reaper::get().medium_reaper()
        );
        Ok(())
    })
}

fn set_project_tempo() -> TestStep {
    step(AllVersions, "Set project tempo", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .master_tempo_changed()
                .take_until(step.finished)
                .subscribe(move |_| {
                    mock.invoke(());
                });
        });
        project.set_tempo(
            Tempo::from_bpm(Bpm::new(130.0)),
            UndoBehavior::OmitUndoPoint,
        );
        // Then
        assert_eq!(project.tempo().bpm(), Bpm::new(130.0));
        // TODO-low There should be only one event invocation
        assert_eq!(mock.invocation_count(), 2);
        Ok(())
    })
}

fn set_project_play_rate() -> TestStep {
    step(AllVersions, "Set project play rate", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .master_playrate_changed()
                .take_until(step.finished)
                .subscribe(move |_| {
                    mock.invoke(());
                });
        });
        project.set_play_rate(PlayRate::from_playback_speed_factor(
            PlaybackSpeedFactor::MAX,
        ));
        // Then
        assert_eq!(
            project.play_rate().playback_speed_factor(),
            PlaybackSpeedFactor::new(4.0)
        );
        assert_eq!(
            project.play_rate().normalized_value(),
            NormalizedPlayRate::MAX
        );
        assert_eq!(mock.invocation_count(), 1);
        Ok(())
    })
}

fn get_project_tempo() -> TestStep {
    step(AllVersions, "Get project tempo", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let tempo = project.tempo();
        // Then
        assert_eq!(tempo.bpm(), Bpm::new(120.0));
        assert!(abs_diff_eq!(tempo.normalized_value(), 119.0 / 959.0));
        Ok(())
    })
}

fn get_project_play_rate() -> TestStep {
    step(AllVersions, "Get project play rate", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let play_rate = project.play_rate();
        // Then
        assert_eq!(
            play_rate.playback_speed_factor(),
            PlaybackSpeedFactor::NORMAL
        );
        assert_eq!(play_rate.normalized_value(), NormalizedPlayRate::NORMAL);
        Ok(())
    })
}

fn mark_project_as_dirty() -> TestStep {
    step(AllVersions, "Mark project as dirty", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        project.mark_as_dirty();
        // Then
        // TODO Doesn't say very much because it has been dirty before already. Save before!?
        assert!(project.is_dirty());
        Ok(())
    })
}

fn get_reaper_window() -> TestStep {
    step(AllVersions, "Get REAPER window", |_session, _| {
        // Given
        // When
        Reaper::get().main_window();
        // Then
        Ok(())
    })
}

fn redo() -> TestStep {
    step(AllVersions, "Redo", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let successful = project.redo();
        let label = project.label_of_last_undoable_action();
        // Then
        assert!(successful);
        assert_eq!(track.name().ok_or("no track name")?.to_str(), "Renamed");
        assert_eq!(
            label.ok_or("no undo label")?.to_str(),
            "reaper-rs integration test operation"
        );
        Ok(())
    })
}

fn undo() -> TestStep {
    step(AllVersions, "Undo", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let successful = project.undo();
        let label = project.label_of_last_redoable_action();
        // Then
        assert!(successful);
        assert_eq!(track.name().ok_or("no track name")?.to_str().len(), 0);
        assert_eq!(
            label.ok_or("no redo label")?.to_str(),
            "reaper-rs integration test operation"
        );
        Ok(())
    })
}

fn use_undoable() -> TestStep {
    step(AllVersions, "Use undoable", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_name_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let track_mirror = track.clone();
        project.undoable("reaper-rs integration test operation", move || {
            track_mirror.set_name("Renamed");
        });
        let label = project.label_of_last_undoable_action();
        // Then
        assert_eq!(track.name().ok_or("no track name")?.to_str(), "Renamed");
        assert_eq!(
            label.ok_or("no undo label")?.to_str(),
            "reaper-rs integration test operation"
        );
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn stuff_midi_devices() -> TestStep {
    step(AllVersions, "Stuff MIDI messages", |_, _| {
        // Given
        let msg = RawShortMessage::note_on(channel(0), key_number(64), u7(100));
        // When
        // reaper
        //     .do_later_in_real_time_audio_thread_asap(|rt_reaper| {
        //         rt_reaper
        //             .midi_message_received()
        //             // TODO-medium This is fishy. next() will be called from main thread although
        //             //  the rest happens in audio thread. I think we need to use shared subjects.
        //             .take_until(step.finished)
        //             .subscribe(move |_evt| {
        //                 println!("MIDI event arrived");
        //                 // Right now not invoked because MIDI message arrives async.
        //                 // TODO As soon as we have an Observable which is not generic on
        // Observer,                 // introduce  steps which return an
        //                 // Observable<TestStepResult, ()> in order to test
        //                 //  asynchronously that stuffed MIDI messages arrived via
        //                 // midi_message_received().
        //             });
        //     })
        //     .map_err(|_| "couldn't schedule for execution in audio thread")?;
        Reaper::get().stuff_midi_message(StuffMidiMessageTarget::VirtualMidiKeyboardQueue, msg);
        // Then
        Ok(())
    })
}

fn query_midi_output_devices() -> TestStep {
    step(AllVersions, "Query MIDI output devices", |_session, _| {
        // Given
        // When
        Reaper::get().midi_output_devices().count();
        Reaper::get().midi_output_device_by_id(MidiOutputDeviceId::new(0));
        Ok(())
    })
}

fn query_midi_input_devices() -> TestStep {
    step(AllVersions, "Query MIDI input devices", |_session, _| {
        // Given
        // When
        let _devs = Reaper::get().midi_input_devices().count();
        let _dev_0 = Reaper::get().midi_input_device_by_id(MidiInputDeviceId::new(0));
        // Then
        // TODO There might be no MIDI input devices
        //            assert_ne!(devs.count(), 0);
        //            assert!(dev_0.is_available());
        Ok(())
    })
}

fn scroll_mixer() -> TestStep {
    step(AllVersions, "Scroll mixer", |_, _| {
        // Given
        let project = Reaper::get().current_project();
        let track = project.track_by_index(3).ok_or("Missing track 2")?;
        // When
        track.scroll_mixer();
        // Then
        Ok(())
    })
}

fn insert_track_at() -> TestStep {
    step(AllVersions, "Insert track at", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_2 = project.track_by_index(1).ok_or("Missing track 2")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_added()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let new_track = project.insert_track_at(1);
        new_track.set_name("Inserted track");
        // Then
        assert_eq!(project.track_count(), 4);
        assert_eq!(new_track.location(), TrackLocation::NormalTrack(1));
        assert_eq!(new_track.index(), Some(1));
        assert_eq!(
            new_track.name().ok_or("no track name")?.to_str(),
            "Inserted track"
        );
        assert_eq!(track_2.index(), Some(2));
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), new_track);
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
                    "reaperRsTest2",
                    "reaper-rs test toggle action",
                    move || {
                        mock.invoke(43);
                    },
                    toggleable(move || cloned_mock.invocation_count() % 2 == 1),
                )
            });
            let action = Reaper::get().action_by_command_name("reaperRsTest2");
            // Then
            let _action_index = action.index();
            let _command_id = action.command_id();
            assert!(action.is_available());
            assert_eq!(mock.invocation_count(), 0);
            assert_eq!(action.is_on(), Some(false));
            action.invoke_as_trigger(None);
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), 43);
            assert_eq!(action.is_on(), Some(true));
            assert_eq!(action.character(), ActionCharacter::Toggle);
            assert!(action.command_id() > CommandId::new(1));
            assert_eq!(action.command_name().unwrap().to_str(), "reaperRsTest2");
            assert_eq!(action.name().to_str(), "reaper-rs test toggle action");
            reg.unregister();
            assert!(!action.is_available());
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
                    "reaperRsTest",
                    "reaper-rs test action",
                    move || {
                        mock.invoke(42);
                    },
                    ActionKind::NotToggleable,
                )
            });
            let action = Reaper::get().action_by_command_name("reaperRsTest");
            // Then
            assert!(action.is_available());
            assert_eq!(mock.invocation_count(), 0);
            action.invoke_as_trigger(None);
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), 42);
            assert_eq!(action.character(), ActionCharacter::Trigger);
            assert!(action.command_id() > CommandId::new(1));
            assert_eq!(action.command_name().unwrap().to_str(), "reaperRsTest");
            assert_eq!(action.is_on(), None);
            assert_eq!(action.name().to_str(), "reaper-rs test action");
            reaper.go_to_sleep()?;
            assert!(!action.is_available());
            reaper.wake_up()?;
            assert!(action.is_available());
            reg.unregister();
            assert!(!action.is_available());
            Ok(())
        },
    )
}

fn main_section_functions() -> TestStep {
    step(AllVersions, "Main section functions", |_reaper, _| {
        // Given
        let section = Reaper::get().main_section();
        // When
        let actions = unsafe { section.actions() };
        // Then
        assert_eq!(actions.count() as u32, section.action_count());
        Ok(())
    })
}

fn generate_guid() -> TestStep {
    step(AllVersions, "Generate GUID", |_session, _| {
        // Given
        // When
        let guid = Reaper::get().generate_guid();
        // Then
        assert_eq!(guid.to_string_with_braces().len(), 38);
        Ok(())
    })
}

fn unsolo_track() -> TestStep {
    step(AllVersions, "Unsolo track", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_solo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unsolo();
        // Then
        assert!(!track.is_solo());
        // Started to be 2 when making master track notification work
        assert_eq!(mock.invocation_count(), 2);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn solo_track() -> TestStep {
    step(AllVersions, "Solo track", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_solo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.solo();
        // Then
        assert!(track.is_solo());
        // Started to be 2 when making master track notification work
        assert_eq!(mock.invocation_count(), 2);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn solo_track_in_place() -> TestStep {
    step(AllVersions, "Solo track in place", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_solo_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_solo_mode(SoloMode::SoloInPlace);
        // Then
        assert!(track.is_solo());
        assert_eq!(track.solo_mode(), SoloMode::SoloInPlace);
        // Started to be 2 when making master track notification work
        assert_eq!(mock.invocation_count(), 2);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn mute_track() -> TestStep {
    step(AllVersions, "Mute track", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_mute_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.mute();
        // Then
        assert!(track.is_muted());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn unmute_track() -> TestStep {
    step(AllVersions, "Unmute track", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_mute_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unmute();
        // Then
        assert!(!track.is_muted());
        // For some reason REAPER doesn't call SetSurfaceMute on control surfaces when an action
        // caused the muting. So HelperControlSurface still thinks the track was unmuted and
        // therefore will not fire a change event!
        assert_eq!(mock.invocation_count(), 0);
        Ok(())
    })
}

fn test_action_invoked_event() -> TestStep {
    step(AllVersions, "Test actionInvoked event", |_, step| {
        // Given
        let action = Reaper::get()
            .main_section()
            .action_by_command_id(CommandId::new(1582));
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::action_rx()
                .action_invoked()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        Reaper::get()
            .medium_reaper()
            .main_on_command_ex(action.command_id(), 0, CurrentProject);
        // Then
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(*mock.last_arg(), action);
        Ok(())
    })
}

fn invoke_action() -> TestStep {
    step(AllVersions, "Invoke action", |reaper, step| {
        // Given
        let action = Reaper::get()
            .main_section()
            .action_by_command_id(CommandId::new(6));
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::action_rx()
                .action_invoked()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        action.invoke_as_trigger(None);
        // Then
        assert_eq!(action.is_on(), Some(true));
        assert!(track.is_muted());
        let reaper_version = reaper.version();
        if reaper_version >= ReaperVersion::new("6.20")
            || reaper_version.into_inner().to_str().starts_with("6.19+dev")
        {
            assert_eq!(mock.invocation_count(), 1);
            let normalized_value = action
                .normalized_value()
                .ok_or("action should be able to report normalized value")?;
            assert!(abs_diff_eq!(normalized_value, 1.0));
        } else {
            assert_eq!(mock.invocation_count(), 0);
            assert!(action.normalized_value().is_none());
        }
        Ok(())
    })
}

fn query_action() -> TestStep {
    step(AllVersions, "Query action", |_reaper, _| {
        // Given
        let track = get_track(0)?;
        track.select_exclusively();
        assert!(!track.is_muted());
        // When
        let toggle_action = Reaper::get()
            .main_section()
            .action_by_command_id(CommandId::new(6));
        let normal_action = Reaper::get()
            .main_section()
            .action_by_command_id(CommandId::new(41075));
        let normal_action_by_index = Reaper::get()
            .main_section()
            .action_by_index(normal_action.index());
        // Then
        assert!(toggle_action.is_available());
        assert!(normal_action.is_available());
        assert_eq!(toggle_action.character(), ActionCharacter::Toggle);
        assert_eq!(normal_action.character(), ActionCharacter::Trigger);
        assert_eq!(toggle_action.is_on(), Some(false));
        assert_eq!(normal_action.is_on(), None);
        assert_eq!(toggle_action.clone(), toggle_action);
        assert_eq!(toggle_action.command_id(), CommandId::new(6));
        assert!(toggle_action.command_name().is_none());
        assert_eq!(
            toggle_action.name().to_str(),
            "Track: Toggle mute for selected tracks"
        );
        assert!(toggle_action.index() > 0);
        assert_eq!(toggle_action.section(), Reaper::get().main_section());
        assert_eq!(normal_action_by_index, normal_action);
        Ok(())
    })
}

fn query_time_ranges() -> TestStep {
    step(AllVersions, "Query time ranges", |_, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let time_selection = project.time_selection();
        let loop_points = project.loop_points();
        // Then
        assert!(time_selection.is_none());
        assert!(loop_points.is_none());
        Ok(())
    })
}

fn set_time_ranges() -> TestStep {
    step(AllVersions, "Set time ranges", |_, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        project.set_time_selection(PositionInSeconds::new(5.0), PositionInSeconds::new(7.0));
        project.set_loop_points(
            PositionInSeconds::new(5.0),
            PositionInSeconds::new(7.0),
            AutoSeekBehavior::DenyAutoSeek,
        );
        // Then
        let time_selection = project.time_selection().unwrap();
        assert!(abs_diff_eq!(time_selection.start.get(), 5.0));
        assert!(abs_diff_eq!(time_selection.end.get(), 7.0));
        let loop_points = project.loop_points().unwrap();
        assert!(abs_diff_eq!(loop_points.start.get(), 5.0));
        assert!(abs_diff_eq!(loop_points.end.get(), 7.0));
        Ok(())
    })
}

fn set_track_send_pan() -> TestStep {
    step(AllVersions, "Set track send pan", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_3 = project.track_by_index(2).ok_or("Missing track 3")?;
        let send = track_1
            .find_send_by_destination_track(&track_3)
            .ok_or("missing send")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_route_pan_changed()
                .take_until(step.finished.clone())
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        send.set_pan(Pan::from_normalized_value(0.25)).unwrap();
        // Then
        assert_eq!(send.pan().reaper_value(), ReaperPanValue::new(-0.5));
        assert_eq!(send.pan().normalized_value(), 0.25);
        assert_eq!(mock.invocation_count(), 2);
        Ok(())
    })
}

fn set_track_send_mute() -> TestStep {
    step(AllVersions, "Mute track send", |_, _| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_3 = project.track_by_index(2).ok_or("Missing track 3")?;
        let send = track_1
            .find_send_by_destination_track(&track_3)
            .ok_or("missing send")?;
        // When
        send.mute();
        // Then
        assert!(send.is_muted());
        // When
        send.mute();
        // Then
        assert!(send.is_muted());
        Ok(())
    })
}

fn set_track_send_volume() -> TestStep {
    step(AllVersions, "Set track send volume", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_3 = project.track_by_index(2).ok_or("Missing track 3")?;
        let send = track_1
            .find_send_by_destination_track(&track_3)
            .ok_or("missing send")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_route_volume_changed()
                .take_until(step.finished.clone())
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        send.set_volume(Volume::try_from_soft_normalized_value(0.25).unwrap())
            .unwrap();
        // Then
        assert!(abs_diff_eq!(
            send.volume().db().get(),
            -30.009_531_739_774_296,
            epsilon = 0.000_000_000_000_1
        ));
        assert_eq!(mock.invocation_count(), 2);
        Ok(())
    })
}

fn query_track_send() -> TestStep {
    step(AllVersions, "Query track send", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_2 = project.track_by_index(1).ok_or("Missing track 2")?;
        let track_3 = project.add_track();
        // When
        let send_to_track_2 = track_1
            .find_send_by_destination_track(&track_2)
            .ok_or("missing send")?;
        let send_to_track_3 = track_1.add_send_to(&track_3);
        // Then
        assert!(send_to_track_2.is_available());
        assert!(send_to_track_3.is_available());
        assert_eq!(send_to_track_2.index(), 0);
        assert_eq!(send_to_track_3.index(), 1);
        assert_eq!(send_to_track_2.track(), &track_1);
        assert_eq!(send_to_track_3.track(), &track_1);
        assert_eq!(
            send_to_track_2.partner(),
            Some(TrackRoutePartner::Track(track_2))
        );
        assert_eq!(send_to_track_2.name().to_str(), "Track 2");
        assert_eq!(
            send_to_track_3.partner(),
            Some(TrackRoutePartner::Track(track_3))
        );
        assert_eq!(send_to_track_2.volume().db(), Db::ZERO_DB);
        assert_eq!(send_to_track_3.volume().db(), Db::ZERO_DB);
        assert!(!send_to_track_2.is_muted());
        assert!(!send_to_track_3.is_muted());
        Ok(())
    })
}

fn add_track_send() -> TestStep {
    step(AllVersions, "Add track send", |_session, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_2 = project.track_by_index(1).ok_or("Missing track 2")?;
        // When
        let (send_mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_send_count_changed()
                .take_until(step.finished.clone())
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let (receive_mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .receive_count_changed()
                .take_until(step.finished.clone())
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let send = track_1.add_send_to(&track_2);
        // Then
        assert_eq!(track_1.send_count(), 1);
        assert_eq!(track_1.typed_send_count(SendPartnerType::Track), 1);
        assert_eq!(track_1.typed_send_count(SendPartnerType::HardwareOutput), 0);
        assert_eq!(track_1.receive_count(), 0);
        assert_eq!(track_2.receive_count(), 1);
        assert_eq!(track_1.send_by_index(0).unwrap(), send);
        assert_eq!(
            track_1
                .typed_send_by_index(SendPartnerType::Track, 0)
                .unwrap(),
            send
        );
        assert_eq!(
            track_1.typed_send_by_index(SendPartnerType::HardwareOutput, 0),
            None
        );
        assert_eq!(track_1.receive_by_index(0), None);
        assert!(track_2.receive_by_index(0).unwrap().is_available());
        assert!(
            track_1
                .find_send_by_destination_track(&track_2)
                .ok_or("missing send")?
                .is_available()
        );
        assert!(track_2.find_send_by_destination_track(&track_1).is_none());
        assert!(
            track_2
                .find_receive_by_source_track(&track_1)
                .ok_or("missing receive")?
                .is_available()
        );
        assert!(track_1.find_receive_by_source_track(&track_2).is_none());
        assert_eq!(track_1.sends().count(), 1);
        assert_eq!(track_2.sends().count(), 0);
        assert_eq!(track_1.typed_sends(SendPartnerType::Track).count(), 1);
        assert_eq!(
            track_1.typed_sends(SendPartnerType::HardwareOutput).count(),
            0
        );
        assert_eq!(track_2.receives().count(), 1);
        assert_eq!(track_1.receives().count(), 0);
        assert_eq!(send_mock.invocation_count(), 1);
        assert_eq!(send_mock.last_arg(), track_1);
        assert_eq!(receive_mock.invocation_count(), 1);
        assert_eq!(receive_mock.last_arg(), track_2);
        Ok(())
    })
}

fn query_track_route_count() -> TestStep {
    step(AllVersions, "Query track route count", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let track_send_count = track.typed_send_count(SendPartnerType::Track);
        let hw_output_send_count = track.typed_send_count(SendPartnerType::HardwareOutput);
        let receive_count = track.receive_count();
        // Then
        assert_eq!(track_send_count, 0);
        assert_eq!(hw_output_send_count, 0);
        assert_eq!(receive_count, 0);
        assert!(
            track
                .typed_send_by_index(SendPartnerType::Track, 0)
                .is_none()
        );
        assert!(
            track
                .typed_send_by_index(SendPartnerType::HardwareOutput, 0)
                .is_none()
        );
        assert!(track.receive_by_index(0).is_none());
        assert!(track.find_send_by_destination_track(&track).is_none());
        assert!(track.send_by_index(0).is_none());
        assert!(
            track
                .typed_send_by_index(SendPartnerType::Track, 0)
                .is_none()
        );
        assert!(
            track
                .typed_send_by_index(SendPartnerType::HardwareOutput, 0)
                .is_none()
        );
        assert!(track.receive_by_index(0).is_none());
        assert_eq!(track.sends().count(), 0);
        assert_eq!(track.typed_sends(SendPartnerType::Track).count(), 0);
        assert_eq!(
            track.typed_sends(SendPartnerType::HardwareOutput).count(),
            0
        );
        assert_eq!(track.receives().count(), 0);
        Ok(())
    })
}

fn query_track_automation_mode() -> TestStep {
    step(AllVersions, "Query track automation mode", |_session, _| {
        // Given
        let track = get_track(0)?;
        // When
        let automation_mode = track.automation_mode();
        let global_automation_override = Reaper::get().global_automation_override();
        let effective_automation_mode = track.effective_automation_mode();
        // Then
        assert_eq!(automation_mode, AutomationMode::TrimRead);
        assert_eq!(global_automation_override, None);
        assert_eq!(effective_automation_mode, Some(AutomationMode::TrimRead));
        Ok(())
    })
}

fn query_track_misc() -> TestStep {
    step(AllVersions, "Query track misc", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        // Then
        assert_eq!(track.folder_depth_change(), 0);
        Ok(())
    })
}

fn remove_track() -> TestStep {
    step(AllVersions, "Remove track", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_count_before = project.track_count();
        let track_1 = project
            .track_by_ref(TrackLocation::NormalTrack(0))
            .ok_or("Missing track 1")?;
        let track_2 = project
            .track_by_ref(TrackLocation::NormalTrack(1))
            .ok_or("Missing track 2")?;
        let track_2_guid = track_2.guid();
        assert!(track_1.is_available());
        assert_eq!(track_2.index(), Some(1));
        assert!(track_2.is_available());
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_removed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        project.remove_track(&track_1);
        // Then
        assert_eq!(project.track_count(), track_count_before - 1);
        assert!(!track_1.is_available());
        assert_eq!(track_2.index(), Some(0));
        assert_eq!(track_2.guid(), track_2_guid);
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track_1);
        Ok(())
    })
}

fn select_track_exclusively() -> TestStep {
    step(AllVersions, "Select track exclusively", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track_1 = project.track_by_index(0).ok_or("Missing track 1")?;
        let track_2 = project.track_by_index(1).ok_or("Missing track 2")?;
        let track_3 = project.track_by_index(2).ok_or("Missing track 3")?;
        let master_track = project.master_track();
        assert!(master_track.is_selected());
        track_1.unselect();
        track_2.select();
        track_3.select();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track_1.select_exclusively();
        // Then
        assert!(track_1.is_selected());
        assert!(!track_2.is_selected());
        assert!(!track_3.is_selected());
        assert!(!master_track.is_selected());
        assert_eq!(
            project.selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            1
        );
        assert!(
            project
                .first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
                .is_some()
        );
        assert_eq!(
            project
                .selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            1
        );
        // 4 because master track is unselected, too
        assert_eq!(mock.invocation_count(), 4);
        Ok(())
    })
}

fn arm_track_in_auto_arm_mode_ignoring_auto_arm() -> TestStep {
    step(
        AllVersions,
        "Arm track in auto-arm mode (ignoring auto-arm)",
        |_, step| {
            // Given
            let track = get_track(0)?;
            track.enable_auto_arm()?;
            assert!(track.has_auto_arm_enabled());
            assert!(!track.is_armed(true));
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.arm(false);
            // Then
            assert!(track.is_armed(true));
            assert!(track.is_armed(false));
            assert!(!track.has_auto_arm_enabled());
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), track);
            Ok(())
        },
    )
}

fn disarm_track_in_auto_arm_mode_ignoring_auto_arm() -> TestStep {
    step(
        AllVersions,
        "Disarm track in auto-arm mode (ignoring auto-arm)",
        |_, step| {
            // Given
            let track = get_track(0)?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .track_arm_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.disarm(false);
            // Then
            assert!(!track.is_armed(true));
            assert!(!track.is_armed(false));
            assert!(!track.has_auto_arm_enabled());
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), track);
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
            track.enable_auto_arm()?;
            // Then
            assert!(track.has_auto_arm_enabled());
            assert!(track.is_armed(true));
            assert!(track.is_armed(false));
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
            assert!(track.is_armed(true));
            // When
            track.disable_auto_arm()?;
            // Then
            assert!(!track.has_auto_arm_enabled());
            assert!(track.is_armed(true));
            assert!(track.is_armed(false));
            Ok(())
        },
    )
}

fn disable_track_auto_arm_mode() -> TestStep {
    step(AllVersions, "Disable track auto-arm mode", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.disable_auto_arm()?;
        // Then
        assert!(!track.has_auto_arm_enabled());
        assert!(!track.is_armed(true));
        assert!(!track.is_armed(false));
        Ok(())
    })
}

fn disarm_track_in_auto_arm_mode() -> TestStep {
    step(AllVersions, "Disarm track in auto-arm mode", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.disarm(true);
        // Then
        assert!(!track.is_armed(true));
        assert!(!track.is_armed(false));
        assert!(track.has_auto_arm_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn arm_track_in_auto_arm_mode() -> TestStep {
    step(AllVersions, "Arm track in auto-arm mode", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.arm(true);
        // Then
        assert!(track.is_armed(true));
        // TODO Interesting! GetMediaTrackInfo_Value read with I_RECARM seems to support
        // auto-arm already! So maybe we should remove the chunk check and the
        // parameter supportAutoArm
        assert!(track.is_armed(false));
        assert!(track.has_auto_arm_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn enable_track_in_auto_arm_mode() -> TestStep {
    step(AllVersions, "Enable track auto-arm mode", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.enable_auto_arm()?;
        // Then
        assert!(track.has_auto_arm_enabled());
        assert!(!track.is_armed(true));
        assert!(!track.is_armed(false));
        Ok(())
    })
}

fn disarm_track_in_normal_mode() -> TestStep {
    step(AllVersions, "Disarm track in normal mode", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.disarm(true);
        // Then
        assert!(!track.is_armed(true));
        assert!(!track.is_armed(false));
        assert!(!track.has_auto_arm_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn arm_track_in_normal_mode() -> TestStep {
    step(AllVersions, "Arm track in normal mode", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_arm_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.arm(true);
        // Then
        assert!(track.is_armed(true));
        assert!(track.is_armed(false));
        assert!(!track.has_auto_arm_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
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
        assert!(!is_armed);
        assert!(!is_armed_ignoring_auto_arm);
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
        assert!(!is_in_auto_arm_mode);
        Ok(())
    })
}

fn select_master_track() -> TestStep {
    step(AllVersions, "Select master track", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let master_track = project.master_track();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        project.unselect_all_tracks();
        master_track.select();
        // Then
        assert!(master_track.is_selected());
        assert_eq!(
            project.selected_track_count(MasterTrackBehavior::IncludeMasterTrack),
            1
        );
        let first_selected_track = project
            .first_selected_track(MasterTrackBehavior::IncludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        assert!(first_selected_track.is_master_track());
        assert_eq!(
            project
                .selected_tracks(MasterTrackBehavior::IncludeMasterTrack)
                .count(),
            1
        );
        assert_eq!(mock.invocation_count(), 2);
        assert_eq!(mock.last_arg().0.index(), None);
        Ok(())
    })
}

fn unselect_track() -> TestStep {
    step(AllVersions, "Unselect track", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.unselect();
        // Then
        assert!(!track.is_selected());
        assert_eq!(
            project.selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            1
        );
        let first_selected_track = project
            .first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        assert_eq!(first_selected_track.index(), Some(2));
        assert_eq!(
            project
                .selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            1
        );
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg().0, track);
        Ok(())
    })
}

fn select_track() -> TestStep {
    step(AllVersions, "Select track", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        let track2 = project.track_by_index(2).ok_or("No track at index 2")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_selected_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.select();
        track2.select();
        // Then
        assert!(track.is_selected());
        assert!(track2.is_selected());
        assert_eq!(
            project.selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            2
        );
        let first_selected_track = project
            .first_selected_track(MasterTrackBehavior::ExcludeMasterTrack)
            .ok_or("Couldn't get first selected track")?;
        assert_eq!(first_selected_track.index(), Some(0));
        assert_eq!(
            project
                .selected_tracks(MasterTrackBehavior::ExcludeMasterTrack)
                .count(),
            2
        );
        assert_eq!(mock.invocation_count(), 2);
        assert_eq!(mock.last_arg().0, track2);
        Ok(())
    })
}

fn query_track_selection_state() -> TestStep {
    step(AllVersions, "Query track selection state", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let is_selected = track.is_selected();
        // Then
        assert!(!is_selected);
        assert_eq!(
            project.selected_track_count(MasterTrackBehavior::ExcludeMasterTrack),
            0
        );
        Ok(())
    })
}

fn set_track_pan() -> TestStep {
    step(AllVersions, "Set track pan", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_pan_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_pan(Pan::from_normalized_value(0.25));
        // Then
        let pan = track.pan();
        assert_eq!(pan.reaper_value(), ReaperPanValue::new(-0.5));
        assert_eq!(pan.normalized_value(), 0.25);
        assert_eq!(pan.to_string().as_str(), "50%L");
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        let parsed_pan: Pan = "20%L".parse()?;
        assert!(abs_diff_eq!(parsed_pan.reaper_value().get(), -0.2));
        Ok(())
    })
}

fn set_track_width() -> TestStep {
    step(AllVersions, "Set track width", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_pan_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_width(Width::from_normalized_value(0.25));
        // Then
        let width = track.width();
        assert_eq!(width.reaper_value(), ReaperWidthValue::new(-0.5));
        assert_eq!(width.normalized_value(), 0.25);
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn disable_all_track_fx() -> TestStep {
    step(AllVersions, "Disable all track FX", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.disable_fx();
        // Then
        assert!(!track.fx_is_enabled());
        Ok(())
    })
}

fn enable_all_track_fx() -> TestStep {
    step(AllVersions, "Enable all track FX", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        track.enable_fx();
        // Then
        assert!(track.fx_is_enabled());
        Ok(())
    })
}

fn query_track_pan() -> TestStep {
    step(AllVersions, "Query track pan", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let pan = track.pan();
        // Then
        assert_eq!(pan.reaper_value(), ReaperPanValue::CENTER);
        assert_eq!(pan.normalized_value(), 0.5);
        assert_eq!(pan.to_string().as_str(), "center");
        Ok(())
    })
}

fn query_track_width() -> TestStep {
    step(AllVersions, "Query track width", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let width = track.width();
        // Then
        assert_eq!(width.reaper_value(), ReaperWidthValue::MAX);
        assert_eq!(width.normalized_value(), 1.0);
        Ok(())
    })
}

fn set_track_volume() -> TestStep {
    step(AllVersions, "Set track volume", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_volume_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_volume(Volume::try_from_soft_normalized_value(0.25).unwrap());
        // Then
        let volume = track.volume();
        assert!(abs_diff_eq!(
            volume.reaper_value().get(),
            0.031_588_093_366_685_01,
            epsilon = 0.000_000_000_000_1
        ));
        let db = volume.db().get();
        assert!(abs_diff_eq!(
            db,
            -30.009_531_739_774_296,
            epsilon = 0.000_000_000_000_1
        ));
        assert!(abs_diff_eq!(
            volume.soft_normalized_value(),
            0.250_000_000_000_034_97,
            epsilon = 0.000_000_000_000_1
        ));
        assert_eq!(volume.to_string().as_str(), "-30.0dB");
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn set_track_volume_extreme_values() -> TestStep {
    step(
        AllVersions,
        "Set track volume extreme values",
        |_session, _| {
            // Given
            let track_1 = get_track(0)?;
            let track_2 = get_track(1)?;
            // When
            let track_1_result = unsafe {
                Reaper::get().medium_reaper().csurf_on_volume_change_ex(
                    track_1.raw(),
                    ValueChange::Absolute(ReaperVolumeValue::new(1.0 / 0.0)),
                    GangBehavior::DenyGang,
                );
                Reaper::get()
                    .medium_reaper()
                    .get_track_ui_vol_pan(track_1.raw())
                    .unwrap()
            };
            let track_2_result = unsafe {
                Reaper::get().medium_reaper().csurf_on_volume_change_ex(
                    track_2.raw(),
                    ValueChange::Absolute(ReaperVolumeValue::new(f64::NAN)),
                    GangBehavior::DenyGang,
                );
                Reaper::get()
                    .medium_reaper()
                    .get_track_ui_vol_pan(track_2.raw())
                    .unwrap()
            };
            // Then
            assert_eq!(track_1_result.volume, ReaperVolumeValue::new(1.0 / 0.0));
            let track_1_volume = Volume::from_reaper_value(track_1_result.volume);
            assert_eq!(track_1_volume.db(), Db::new(1.0 / 0.0));
            assert_eq!(track_1_volume.soft_normalized_value(), 1.0 / 0.0);
            assert_eq!(
                track_1_volume.reaper_value(),
                ReaperVolumeValue::new(1.0 / 0.0)
            );
            #[cfg(target_family = "windows")]
            assert_eq!(track_1_volume.to_string().as_str(), "+1.#dB");
            #[cfg(target_family = "unix")]
            assert_eq!(track_1_volume.to_string().as_str(), "+indB");

            assert!(track_2_result.volume.get().is_nan());
            let track_2_volume = Volume::from_reaper_value(track_2_result.volume);
            assert!(track_2_volume.db().get().is_nan());
            assert!(track_2_volume.soft_normalized_value().is_nan());
            assert!(track_2_volume.reaper_value().get().is_nan());
            #[cfg(target_family = "windows")]
            assert_eq!(track_2_volume.to_string().as_str(), "1.#RdB");
            #[cfg(target_family = "unix")]
            assert_eq!(track_2_volume.to_string().as_str(), "nandB");
            Ok(())
        },
    )
}

fn query_track_volume() -> TestStep {
    step(AllVersions, "Query track volume", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let volume = track.volume();
        // Then
        assert_eq!(volume.reaper_value(), ReaperVolumeValue::ZERO_DB);
        assert_eq!(volume.db(), Db::ZERO_DB);
        assert_eq!(volume.to_string().as_str(), "0.00dB");
        assert!(abs_diff_eq!(volume.soft_normalized_value(), 0.716));
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
            assert_eq!(track.recording_input(), given_input);
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
            assert_eq!(track.recording_input(), given_input);
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
        assert_eq!(track.recording_input(), given_input);
        Ok(())
    })
}

fn set_track_recording_input_midi_all_all() -> TestStep {
    step(
        AllVersions,
        "Set track recording input MIDI all/all",
        |_, step| {
            // Given
            let track = get_track(0)?;
            let given_input = Some(RecordingInput::Midi {
                device_id: None,
                channel: None,
            });
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .track_input_changed()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            track.set_recording_input(given_input);
            // Then
            let input = track.recording_input();
            assert_eq!(input, given_input);
            let input = input.unwrap();
            assert_eq!(input.to_raw(), 6112);
            assert_eq!(RecordingInput::from_raw(6112), input);
            // TODO-high Search in project for 5198273 for a hacky way to solve this
            assert_eq!(mock.invocation_count(), 0);
            // assert_eq!(mock.last_arg(), track);
            Ok(())
        },
    )
}

fn query_track_recording_input() -> TestStep {
    step(AllVersions, "Query track recording input", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let input = track.recording_input();
        // Then
        match input {
            Some(RecordingInput::Mono(0)) => Ok(()),
            _ => Err("Expected MidiRecordingInput".into()),
        }
    })
}

fn set_track_input_monitoring() -> TestStep {
    step(AllVersions, "Set track input monitoring", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_input_monitoring_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_input_monitoring_mode(InputMonitoringMode::NotWhenPlaying);
        // Then
        assert_eq!(
            track.input_monitoring_mode(),
            InputMonitoringMode::NotWhenPlaying
        );
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn query_track_input_monitoring() -> TestStep {
    step(
        AllVersions,
        "Query track input monitoring",
        |_session, _| {
            // Given
            let track = get_track(0)?;
            // When
            let mode = track.input_monitoring_mode();
            // Then
            use InputMonitoringMode::*;
            if Reaper::get().version() < ReaperVersion::new("6") {
                assert_eq!(mode, Off);
            } else {
                assert_eq!(mode, Normal);
            }
            Ok(())
        },
    )
}

fn set_track_name() -> TestStep {
    step(AllVersions, "Set track name", |_, step| {
        // Given
        let track = get_track(0)?;
        // When
        // TODO Factor this state pattern out
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_name_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        track.set_name("Foo Bla");
        // Then
        assert_eq!(track.name().ok_or("no track name")?.to_str(), "Foo Bla");
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), track);
        Ok(())
    })
}

fn query_track_name() -> TestStep {
    step(AllVersions, "Query track name", |_, _| {
        // Given
        let track = get_track(0)?;
        // When
        let track_name = track.name();
        // Then
        assert_eq!(track_name.ok_or("no track name")?.to_str().len(), 0);
        Ok(())
    })
}

fn query_track_project() -> TestStep {
    step(AllVersions, "Query track project", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let track = get_track(0)?;
        // When
        let track_project = track.project();
        // Then
        assert_eq!(track_project, project);
        Ok(())
    })
}

fn query_non_existent_track_by_guid() -> TestStep {
    step(
        AllVersions,
        "Query non-existent track by GUID",
        |_session, _| {
            // Given
            let project = Reaper::get().current_project();
            // When
            let guid = Guid::from_string_with_braces("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}")?;
            let found_track = project.track_by_guid(&guid);
            // Then
            assert!(!found_track.is_available());
            Ok(())
        },
    )
}

fn query_track_by_guid() -> TestStep {
    step(AllVersions, "Query track by GUID", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        let first_track = get_track(0)?;
        let new_track = project.add_track();
        // When
        let found_track = project.track_by_guid(new_track.guid());
        // Then
        assert!(found_track.is_available());
        assert_eq!(&found_track, &new_track);
        assert_ne!(&found_track, &first_track);
        assert_eq!(new_track.guid(), &get_media_track_guid(new_track.raw()));
        Ok(())
    })
}

fn query_all_tracks() -> TestStep {
    step(AllVersions, "Query all tracks", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        project.add_track();
        // When
        let tracks = project.tracks();
        // Then
        assert_eq!(tracks.count(), 2);
        Ok(())
    })
}

fn query_master_track() -> TestStep {
    step(AllVersions, "Query master track", |_session, _| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let master_track = project.master_track();
        // Then
        assert_eq!(master_track.location(), TrackLocation::MasterTrack);
        assert!(master_track.is_master_track());
        Ok(())
    })
}

fn fn_mut_action() -> TestStep {
    #[allow(unreachable_code)]
    step(AllVersions, "FnMut action", |_session, _| {
        // TODO-low Add this as new test
        return Ok(());
        let mut i = 0;
        let _action1 = _session.register_action(
            "reaperRsCounter",
            "reaper-rs counter",
            move || {
                Reaper::get().show_console_msg(format!("Hello from Rust number {}\0", i));
                i += 1;
            },
            ActionKind::NotToggleable,
        );
        Ok(())
    })
}

fn play_pause_stop_record() -> TestStep {
    step(AllVersions, "Play/pause/stop/record", |reaper, step| {
        // Given
        let project = Reaper::get().current_project();
        // When
        // Then
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .play_state_changed()
                .take_until(step.finished)
                .subscribe(move |_| {
                    mock.invoke(());
                });
        });
        assert!(!project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(project.is_stopped());
        assert_eq!(mock.invocation_count(), 0);
        project.play();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 1);
        project.play();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 2);
        project.pause();
        assert!(!project.is_playing());
        assert!(project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 3);
        project.pause();
        assert!(!project.is_playing());
        assert!(project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 3);
        project.play();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 4);
        project.stop();
        assert!(!project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(project.is_stopped());
        assert_eq!(mock.invocation_count(), 6);
        reaper.enable_record_in_current_project();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 7);
        reaper.enable_record_in_current_project();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 7);
        reaper.disable_record_in_current_project();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 8);
        reaper.disable_record_in_current_project();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 8);
        reaper.enable_record_in_current_project();
        assert!(project.is_playing());
        assert!(!project.is_paused());
        assert!(project.is_recording());
        assert!(!project.is_stopped());
        assert_eq!(mock.invocation_count(), 9);
        project.stop();
        assert!(!project.is_playing());
        assert!(!project.is_paused());
        assert!(!project.is_recording());
        assert!(project.is_stopped());
        assert_eq!(mock.invocation_count(), 11);
        Ok(())
    })
}

fn change_repeat_state() -> TestStep {
    step(AllVersions, "Repeat", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        // When
        // Then
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .repeat_state_changed()
                .take_until(step.finished)
                .subscribe(move |_| {
                    mock.invoke(());
                });
        });
        assert!(!project.repeat_is_enabled());
        assert_eq!(mock.invocation_count(), 0);
        project.enable_repeat();
        assert!(project.repeat_is_enabled());
        assert_eq!(mock.invocation_count(), 1);
        project.disable_repeat();
        assert!(!project.repeat_is_enabled());
        assert_eq!(mock.invocation_count(), 2);
        Ok(())
    })
}

fn add_track() -> TestStep {
    step(AllVersions, "Add track", |_, step| {
        // Given
        let project = Reaper::get().current_project();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .track_added()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        let new_track = project.add_track();
        // Then
        assert_eq!(project.track_count(), 1);
        assert_eq!(new_track.index(), Some(0));
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), new_track);
        Ok(())
    })
}

fn general() -> TestStep {
    step(AllVersions, "General", |reaper, _| {
        // Given
        // When
        let resource_path = reaper.resource_path();
        // Then
        assert!(resource_path.is_dir());
        assert!(
            resource_path
                .to_str()
                .ok_or("invalid resource path")?
                .to_lowercase()
                .contains("reaper")
        );
        Ok(())
    })
}

fn volume_types() -> TestStep {
    step(AllVersions, "Volume types", |reaper, _| {
        // Given
        let input_values = vec![
            0.0,
            0.00000000000001,
            0.2,
            0.5,
            0.9,
            1.0,
            1.0000001,
            1.5,
            2.0,
            12.0,
            20.0,
            100_000.0,
            std::f64::NAN,
            // std::f64::MIN,
            std::f64::MAX,
            std::f64::EPSILON,
            std::f64::INFINITY,
            /* std::f64::MIN_POSITIVE,
             * std::f64::NEG_INFINITY, */
        ]
        .into_iter()
        .map(ReaperVolumeValue::new)
        .chain(vec![
            ReaperVolumeValue::MIN,
            ReaperVolumeValue::MINUS_150_DB,
            ReaperVolumeValue::NAN,
            ReaperVolumeValue::TWELVE_DB,
            ReaperVolumeValue::ZERO_DB,
        ]);
        // When
        // Then
        for input_value in input_values {
            let output_value = Volume::from_reaper_value(input_value);
            reaper.show_console_msg(format!("{:?} => {:?}\n", input_value, output_value));
        }
        Ok(())
    })
}

fn create_empty_project_in_new_tab() -> TestStep {
    step(AllVersions, "Create empty project in new tab", |_, step| {
        // Given
        let current_project_before = Reaper::get().current_project();
        let project_count_before = Reaper::get().project_count();
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .project_switched()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        let new_project = Reaper::get().create_empty_project_in_new_tab();
        // Then
        assert_eq!(current_project_before, current_project_before);
        assert_eq!(Reaper::get().project_count(), project_count_before + 1);
        assert_eq!(
            Reaper::get().projects().count() as u32,
            project_count_before + 1
        );
        assert_ne!(Reaper::get().current_project(), current_project_before);
        assert_eq!(Reaper::get().current_project(), new_project);
        assert_ne!(Reaper::get().projects().next(), Some(new_project));
        //
        // assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().first() ==
        // newProject);
        // assertTrue(Reaper::instance().projectsWithCurrentOneFirst().as_blocking().count() ==
        // projectCountBefore + 1);
        assert_eq!(new_project.track_count(), 0);
        assert!(new_project.index() > 0);
        assert!(new_project.file().is_none());
        assert_eq!(new_project.length(), DurationInSeconds::new(0.0));
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), new_project);
        Ok(())
    })
}

fn strings() -> TestStep {
    step(AllVersions, "Strings", |_session, _| {
        assert!(Guid::from_string_with_braces("{hey}").is_err());
        Reaper::get().show_console_msg(reaper_str!("- &ReaperStr: 范例文字äöüß\n"));
        Reaper::get().show_console_msg("- &str: 范例文字äöüß\n");
        Reaper::get().show_console_msg(String::from("- String: 范例文字äöüß\n"));
        Ok(())
    })
}

fn query_prefs() -> TestStep {
    step(AllVersions, "Query preferences", |_, _| {
        fn query_track_sel_on_mouse_is_enabled() -> bool {
            if let Some(res) = Reaper::get()
                .medium_reaper()
                .get_config_var("trackselonmouse")
            {
                if res.size != 4 {
                    // Shouldn't be.
                    return false;
                }
                let ptr = res.value.as_ptr() as *const u32;
                let value = unsafe { *ptr };
                // The second flag corresponds to that setting.
                (value & 2) != 0
            } else {
                false
            }
        }
        // When
        let is_enabled = query_track_sel_on_mouse_is_enabled();
        // Then
        if is_enabled {
            return Err(
                "\"Mouse click on volume/pan faders and track buttons changes track selection\" seems to be enabled. Maybe you are not using the REAPER default preferences?".into(),
            );
        }
        Ok(())
    })
}

fn global_instances() -> TestStep {
    step(AllVersions, "Global instances", |_, _| {
        // Sizes
        use std::mem::size_of_val;
        let medium_session = Reaper::get().medium_session();
        let medium_reaper = Reaper::get().medium_reaper();
        let metrics_size = size_of_val(medium_reaper.metrics());
        Reaper::get().show_console_msg(format!(
            "\
            Struct sizes in byte:\n\
            - reaper_high::Reaper: {high_reaper} ({high_reaper_no_metrics} without metrics)\n\
            - reaper_medium::ReaperSession: {medium_session} ({medium_session_no_metrics} without metrics)\n\
            - reaper_medium::Reaper: {medium_reaper} ({medium_reaper_no_metrics} without metrics)\n\
            - reaper_low::Reaper: {low_reaper}\n\
            ",
            high_reaper = size_of_val(Reaper::get()),
            high_reaper_no_metrics = size_of_val(Reaper::get()) - metrics_size,
            medium_session = size_of_val(medium_session.deref()),
            medium_session_no_metrics = size_of_val(medium_session.deref()) - metrics_size,
            medium_reaper = size_of_val(medium_reaper),
            medium_reaper_no_metrics = size_of_val(medium_reaper) - metrics_size,
            low_reaper = size_of_val(medium_reaper.low()),
        ));
        // Low-level REAPER
        reaper_low::Reaper::make_available_globally(*medium_reaper.low());
        reaper_low::Reaper::make_available_globally(*medium_reaper.low());
        let low = reaper_low::Reaper::get();
        println!("reaper_low::Reaper {:?}", &low);
        unsafe {
            low.ShowConsoleMsg(c_str!("- Hello from low-level API\n").as_ptr());
        }
        // Low-level SWELL
        let swell = Swell::load(*medium_reaper.low().plugin_context());
        println!("reaper_low::Swell {:?}", &swell);
        Swell::make_available_globally(swell);
        let _ = Swell::get();
        // Medium-level REAPER
        reaper_medium::Reaper::make_available_globally(medium_reaper.clone());
        reaper_medium::Reaper::make_available_globally(medium_reaper.clone());
        medium_reaper.show_console_msg("- Hello from medium-level API\n");
        Ok(())
    })
}

fn register_api_functions() -> TestStep {
    step(AllVersions, "Register API functions", |reaper, _| {
        // Given
        let mut session = reaper.medium_session();
        // When
        unsafe {
            session
                .plugin_register_add_api_and_def(
                    "ReaperRs_HeyThere",
                    hey_there as _,
                    hey_there_vararg,
                    "void",
                    "",
                    "",
                    "Just says hey there.",
                )
                // TODO-low This will fail on second test run. Unregister after usage as soon
                //  as possible!
                .map_err(|_| "couldn't register API function")?;
        }
        // Then
        let ptr = session
            .reaper()
            .plugin_context()
            .get_func("ReaperRs_HeyThere");
        assert!(!ptr.is_null());
        let restored_function: Option<extern "C" fn()> = unsafe { std::mem::transmute(ptr) };
        let restored_function =
            restored_function.ok_or("couldn't restore API function from ptr")?;
        restored_function();
        Ok(())
    })
}

extern "C" fn hey_there() {
    Reaper::get().show_console_msg("Hey there!\n");
}

unsafe extern "C" fn hey_there_vararg(_arglist: *mut *mut c_void, _numparms: c_int) -> *mut c_void {
    hey_there();
    null_mut()
}

#[allow(overflowing_literals)]
fn low_plugin_context() -> TestStep {
    step(AllVersions, "Low plugin context", |_session, _| {
        // Given
        let medium = Reaper::get().medium_reaper();
        let plugin_context = medium.low().plugin_context();
        // When
        // Then
        // GetSwellFunc
        let swell_function_provider = plugin_context.swell_function_provider();
        if cfg!(target_family = "windows") {
            assert!(swell_function_provider.is_none());
        } else {
            let swell_function_provider =
                swell_function_provider.ok_or("SWELL function provider not available")?;
            let swell_func = unsafe { swell_function_provider(c_str!("DefWindowProc").as_ptr()) };
            assert!(!swell_func.is_null());
        }
        use reaper_low::TypeSpecificPluginContext::*;
        match plugin_context.type_specific() {
            Extension(ctx) => unsafe {
                let result = ctx.Register(
                    c_str!("command_id").as_ptr(),
                    c_str!("REAPER_RS_foo").as_ptr() as *mut c_void,
                );
                assert!(result > 0);
            },
            Vst(_ctx) => {}
        };
        Ok(())
    })
}

#[allow(overflowing_literals)]
fn medium_plugin_context() -> TestStep {
    step(AllVersions, "Medium plugin context", |_session, _| {
        // Given
        let medium = Reaper::get().medium_reaper();
        let plugin_context = medium.plugin_context();
        // When
        // Then
        // GetFunc
        let show_console_msg_func = plugin_context.get_func("ShowConsoleMsg");
        assert!(!show_console_msg_func.is_null());
        let bla_func = plugin_context.get_func("Bla");
        assert!(bla_func.is_null());
        use reaper_medium::TypeSpecificPluginContext::*;
        match plugin_context.type_specific() {
            Extension(ctx) => {
                assert!(plugin_context.h_instance().is_some());
                assert_eq!(ctx.caller_version(), raw::REAPER_PLUGIN_VERSION);
                assert_eq!(ctx.hwnd_main(), medium.get_main_hwnd());
            }
            Vst(_ctx) => {
                if cfg!(target_family = "windows") {
                    assert!(plugin_context.h_instance().is_some());
                } else {
                    assert!(plugin_context.h_instance().is_none());
                }
                // TODO-medium We must pass the AEffect for this to work. Refactor test step API
                //  a bit so that it only takes one parameter which also contains passed AEffect.
            }
        };
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
        assert_eq!(fx_chain.fx_count(), 0);
        assert_eq!(fx_chain.fxs().count(), 0);
        assert!(fx_chain.fx_by_index(0).is_none());
        assert!(!fx_chain.fx_by_index_untracked(0).is_available());
        assert!(fx_chain.first_fx().is_none());
        assert!(fx_chain.last_fx().is_none());
        let non_existing_guid =
            Guid::from_string_without_braces("E64BB283-FB17-4702-ACFA-2DDB7E38F14F")?;
        assert!(!fx_chain.fx_by_guid(&non_existing_guid).is_available());
        assert!(
            !fx_chain
                .fx_by_guid_and_index(&non_existing_guid, 0)
                .is_available()
        );
        assert!(fx_chain.first_fx_by_name("bla").is_none());
        assert!(fx_chain.chunk().unwrap().is_none());
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
        change_fx_preset(get_fx_chain),
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
        move |_session, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.track();
            // When
            let fx = fx_chain.fx_by_index(2);
            // Then
            let fx = fx.ok_or("No FX found")?;
            assert!(fx.is_available());
            assert_eq!(fx.index(), 2);
            assert_eq!(
                fx.query_index().to_raw(),
                if fx_chain.is_input_fx() {
                    0x0100_0002
                } else {
                    2
                }
            );
            assert!(fx.guid().is_some());
            assert_eq!(fx.name().into_inner().as_c_str(), c_str!("JS: phaser"));
            let fx_chunk = fx.chunk()?;
            assert!(fx_chunk.starts_with("BYPASS 0 0 0"));
            if Reaper::get().version() < ReaperVersion::new("6") {
                assert!(fx_chunk.ends_with("\nWAK 0"));
            } else {
                assert!(fx_chunk.ends_with("\nWAK 0 0"));
            }
            let tag_chunk = fx.tag_chunk()?;
            assert!(tag_chunk.starts_with(r#"<JS phaser """#));
            assert!(tag_chunk.ends_with("\n>"));
            let state_chunk = fx.state_chunk()?;
            assert!(!state_chunk.contains("<"));
            assert!(!state_chunk.contains(">"));
            assert_eq!(fx.track(), track);
            assert_eq!(fx.is_input_fx(), fx_chain.is_input_fx());
            assert_eq!(fx.chain(), &fx_chain);
            assert_eq!(fx.parameter_count(), 7);
            assert_eq!(fx.parameters().count(), 7);
            let param1 = fx.parameter_by_index(0);
            assert!(param1.is_available());
            // TODO-low Fix for input FX (there it's 1.0 for some reason)
            // assert_eq!(param1.step_size(), Some(0.01));
            assert_eq!(
                param1.value_range(),
                FxParameterValueRange {
                    min_val: 0.0,
                    mid_val: 5.0,
                    max_val: 10.0
                }
            );
            assert!(fx.parameter_by_index(6).is_available());
            assert!(!fx.parameter_by_index(7).is_available());
            let fx_info = fx.info()?;
            let stem = fx_info.file_name.file_stem().ok_or("No stem")?;
            assert_eq!(stem, "phaser");
            Ok(())
        },
    )
}

fn add_track_js_fx_by_original_name(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Add track JS fx by original name",
        move |_, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .fx_added()
                    .take_until(step.finished.clone())
                    .subscribe(move |fx| {
                        mock.invoke(fx);
                    });
            });
            let fx = fx_chain.add_fx_by_original_name("phaser");
            // Then
            let fx = fx.ok_or("No FX added")?;
            assert_eq!(fx_chain.fx_count(), 3);
            assert_eq!(fx_chain.fx_by_index(2), Some(fx.clone()));
            assert_eq!(fx_chain.last_fx(), Some(fx.clone()));
            let fx_guid = fx.guid().ok_or("No GUID")?;
            assert!(fx_chain.fx_by_guid(&fx_guid).is_available());
            let guid: Guid = "{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}".parse()?;
            assert!(!fx_chain.fx_by_guid_and_index(&guid, 0).is_available());
            assert!(
                fx_chain
                    .first_fx_by_name("ReaControlMIDI (Cockos)")
                    .is_some()
            );
            assert_eq!(
                fx_chain.first_fx_by_name(reaper_str!("phaser")),
                Some(fx.clone())
            );
            if Reaper::get().version() < ReaperVersion::new("6") {
                // Mmh
                if fx_chain.is_input_fx() {
                    assert_eq!(mock.invocation_count(), 2);
                } else {
                    assert_eq!(mock.invocation_count(), 3);
                }
            } else {
                assert_eq!(mock.invocation_count(), 1);
                assert_eq!(mock.last_arg(), fx);
            }
            Ok(())
        },
    )
}

fn show_fx_in_floating_window(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Show fx in floating window", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
        // When
        let (fx_opened_mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_opened()
                .take_until(step.finished.clone())
                .subscribe(move |fx| {
                    mock.invoke(fx);
                });
        });
        let (fx_focused_mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_focused()
                .take_until(step.finished)
                .subscribe(move |fx| {
                    mock.invoke(fx);
                });
        });
        fx.show_in_floating_window();
        // Then
        assert!(fx.floating_window().is_some());
        assert!(fx.window_is_open());
        // TODO-low Not correctly implemented right now? Should maybe have focus!
        assert!(!fx.window_has_focus());
        assert!(fx_opened_mock.invocation_count() >= 1);
        if !fx_chain.is_input_fx() || Reaper::get().version() >= ReaperVersion::new("5.95") {
            // In previous versions it wrongly reports as normal FX
            assert_eq!(fx_opened_mock.last_arg(), fx);
        }
        assert_eq!(fx_focused_mock.invocation_count(), 0);
        if cfg!(target_os = "windows") {
            // Should be > 0 but doesn't work
            assert!(Reaper::get().focused_fx().is_none()); // Should be Some but doesn't work
        }
        Ok(())
    })
}

fn query_fx_floating_window(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Query fx floating window",
        move |_session, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
            // When
            // Then
            assert!(fx.floating_window().is_none());
            assert!(!fx.window_is_open());
            assert!(!fx.window_has_focus());
            if cfg!(target_os = "windows") {
                assert!(Reaper::get().focused_fx().is_none());
            };
            Ok(())
        },
    )
}

fn set_fx_chain_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx chain chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let track = fx_chain.track().ok_or("no track")?;
        let other_fx_chain = if fx_chain.is_input_fx() {
            track.normal_fx_chain()
        } else {
            track.input_fx_chain()
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
        other_fx_chain.set_chunk(fx_chain_chunk.as_str())?;
        // Then
        assert_eq!(other_fx_chain.fx_count(), 2);
        assert_eq!(fx_chain.fx_count(), 2);
        Ok(())
    })
}

fn set_fx_state_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx state chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain.fx_by_index(1).ok_or("Couldn't find synth fx")?;
        let synth_param_5 = synth_fx.parameter_by_index(5);
        synth_param_5
            .set_reaper_normalized_value(ReaperNormalizedFxParamValue::new(0.0))
            .map_err(|_| "couldn't set parameter value")?;
        assert_ne!(
            synth_param_5.formatted_value().into_inner().as_c_str(),
            c_str!("-6.00")
        );
        let fx_state_chunk = r#"eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
  776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
  AAAQAAAA"#;
        // When
        synth_fx.set_state_chunk(fx_state_chunk)?;
        // Then
        assert_eq!(synth_fx.index(), 1);
        assert_eq!(
            synth_fx.name().into_inner().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        assert_eq!(
            synth_param_5.formatted_value().into_inner().as_c_str(),
            c_str!("-6.00")
        );
        assert_eq!(midi_fx.index(), 0);
        assert_eq!(
            midi_fx.name().into_inner().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        Ok(())
    })
}

fn set_fx_tag_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx tag chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx_1 = fx_chain.fx_by_index(0).ok_or("Couldn't find MIDI fx 1")?;
        let midi_fx_2 = fx_chain.fx_by_index(1).ok_or("Couldn't find MIDI fx 2")?;
        let fx_tag_chunk = r#"<VST "VSTi: ReaSynth (Cockos)" reasynth.dll 0 "" 1919251321
  eXNlcu9e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAAAAAAAAEAA=
  776t3g3wrd6mm8Q7F7fROgAAAAAAAAAAAAAAAM5NAD/pZ4g9AAAAAAAAAD8AAIA/AACAPwAAAD8AAAAA
  AAAQAAAA
  >"#;
        // When
        midi_fx_2.set_tag_chunk(fx_tag_chunk)?;
        // Then
        assert_eq!(midi_fx_2.index(), 1);
        assert_eq!(
            midi_fx_2.name().into_inner().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        assert_eq!(midi_fx_1.index(), 0);
        assert_eq!(
            midi_fx_1.name().into_inner().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        Ok(())
    })
}

fn set_fx_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx chunk", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain.fx_by_index(1).ok_or("Couldn't find synth fx")?;
        let synth_fx_guid_before = synth_fx.guid();
        // When
        synth_fx.set_chunk(midi_fx.chunk()?)?;
        // Then
        assert_eq!(synth_fx.guid(), synth_fx_guid_before);
        assert!(synth_fx.is_available());
        assert_eq!(
            synth_fx.name().into_inner().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        assert_eq!(midi_fx.index(), 0);
        assert_eq!(synth_fx.index(), 1);
        Ok(())
    })
}

fn add_fx_by_chunk(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Add FX by chunk", move |_, step| {
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
            Test::control_surface_rx()
                .fx_added()
                .take_until(step.finished)
                .subscribe(move |fx| {
                    mock.invoke(fx);
                });
        });
        let synth_fx = fx_chain.add_fx_from_chunk(fx_chunk)?;
        // Then
        assert_eq!(synth_fx.index(), 1);
        let guid = Guid::from_string_with_braces("{5FF5FB09-9102-4CBA-A3FB-3467BA1BFE5D}")?;
        assert_eq!(synth_fx.guid(), Some(guid));
        assert_eq!(
            synth_fx
                .parameter_by_index(5)
                .formatted_value()
                .into_inner()
                .as_c_str(),
            c_str!("-6.00")
        );
        // TODO Detect such a programmatic FX add as well (maybe by hooking into
        // HelperControlSurface::updateMediaTrackPositions)
        assert_eq!(mock.invocation_count(), 0);
        Ok(())
    })
}

fn remove_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Remove FX", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let synth_fx = fx_chain.fx_by_index(0).ok_or("Couldn't find synth fx")?;
        let midi_fx = fx_chain.fx_by_index(1).ok_or("Couldn't find MIDI fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_removed()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        fx_chain.remove_fx(&synth_fx)?;
        // Then
        assert!(!synth_fx.is_available());
        assert!(midi_fx.is_available());
        assert_eq!(midi_fx.index(), 0);
        midi_fx.invalidate_index();
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), synth_fx);
        Ok(())
    })
}

fn move_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Move FX", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let midi_fx = fx_chain.fx_by_index(0).ok_or("Couldn't find MIDI fx")?;
        let synth_fx = fx_chain.fx_by_index(1).ok_or("Couldn't find synth fx")?;
        let fx_at_index_1 = fx_chain.fx_by_index_untracked(1);
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_reordered()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        fx_chain.move_fx(&synth_fx, 0)?;
        // Then
        assert_eq!(midi_fx.index(), 1);
        assert_eq!(
            midi_fx.name().into_inner().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        assert_eq!(synth_fx.index(), 0);
        assert_eq!(
            synth_fx.name().into_inner().as_c_str(),
            c_str!("VSTi: ReaSynth (Cockos)")
        );
        assert_eq!(fx_at_index_1.index(), 1);
        assert_eq!(
            fx_at_index_1.name().into_inner().as_c_str(),
            c_str!("VST: ReaControlMIDI (Cockos)")
        );
        if Reaper::get().version() < ReaperVersion::new("5.95") {
            assert_eq!(mock.invocation_count(), 0);
        } else {
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), *fx_chain.track().ok_or("no track")?);
        }
        Ok(())
    })
}

fn fx_parameter_value_changed_with_heuristic_fail(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "fxParameterValueChanged with heuristic fail in REAPER < 5.95",
        move |_, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            let fx = fx_chain.fx_by_index(0).ok_or("Couldn't find fx")?;
            let p = fx.parameter_by_index(0);
            p.set_reaper_normalized_value(ReaperNormalizedFxParamValue::new(0.5))
                .map_err(|_| "couldn't set parameter value")?;
            let track = fx.track().ok_or("no track")?;
            let other_fx_chain = if fx_chain.is_input_fx() {
                track.normal_fx_chain()
            } else {
                track.input_fx_chain()
            };
            let fx_on_other_fx_chain = other_fx_chain
                .add_fx_by_original_name("ReaControlMIDI (Cockos)")
                .expect("Couldn't find FX on other FX chain");
            let p_on_other_fx_chain = fx_on_other_fx_chain.parameter_by_index(0);
            // First set parameter on other FX chain to same value (confuses heuristic if
            // fxChain is input FX chain)
            p_on_other_fx_chain
                .set_reaper_normalized_value(ReaperNormalizedFxParamValue::new(0.5))
                .map_err(|_| "couldn't set parameter value")?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .fx_parameter_value_changed()
                    .take_until(step.finished)
                    .subscribe(move |p| {
                        mock.invoke(p);
                    });
            });
            p.set_reaper_normalized_value(ReaperNormalizedFxParamValue::new(0.5))
                .map_err(|_| "couldn't set parameter value")?;
            // Then
            assert_eq!(mock.invocation_count(), 2);
            if fx_chain.is_input_fx() && Reaper::get().version() < ReaperVersion::new("5.95") {
                assert_ne!(mock.last_arg(), p);
            } else {
                assert_eq!(mock.last_arg(), p);
            }
            Ok(())
        },
    )
}

fn set_fx_parameter_value(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Set fx parameter value", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain.fx_by_index(1).ok_or("Couldn't find fx")?;
        let p = fx.parameter_by_index(5);
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_parameter_value_changed()
                .take_until(step.finished)
                .subscribe(move |p| {
                    mock.invoke(p);
                });
        });
        p.set_reaper_normalized_value(ReaperNormalizedFxParamValue::new(0.3))
            .map_err(|_| "couldn't set parameter value")?;
        // Then
        let last_touched_fx_param = Reaper::get().last_touched_fx_parameter();
        if fx_chain.is_input_fx() && Reaper::get().version() < ReaperVersion::new("5.95") {
            assert!(last_touched_fx_param.is_none());
        } else {
            assert_eq!(last_touched_fx_param, Some(p.clone()));
        }
        assert_eq!(p.formatted_value().into_inner().as_c_str(), c_str!("-4.44"));
        assert!(abs_diff_eq!(
            p.reaper_normalized_value().get(),
            0.300_000_011_920_928_96
        ));
        assert!(abs_diff_eq!(
            p.reaper_normalized_value().get(),
            0.300_000_011_920_928_96
        ));
        assert_eq!(
            p.format_reaper_normalized_value(p.reaper_normalized_value())
                .map_err(|_| "Cockos plug-ins should be able to do that")?
                .into_inner()
                .as_c_str(),
            c_str!("-4.44 dB")
        );
        if Reaper::get().version() < ReaperVersion::new("6") {
            if fx_chain.is_input_fx() {
                // Mmh
                assert_eq!(mock.invocation_count(), 2);
            } else {
                assert_eq!(mock.invocation_count(), 1);
            }
        } else {
            // TODO-low 1 invocation would be better than 2 (in v6 it gives us 2)
            assert_eq!(mock.invocation_count(), 2);
        }
        assert_eq!(mock.last_arg(), p);
        Ok(())
    })
}

fn check_fx_presets(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check fx presets", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
        // When
        // Then
        assert_eq!(fx.preset_count(), Ok(0));
        assert_eq!(fx.preset_index(), Ok(None));
        assert!(fx.preset_name().is_none());
        assert!(fx.preset_is_dirty());
        Ok(())
    })
}

fn change_fx_preset(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Change FX preset", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain
            .add_fx_by_original_name("ReaEq (Cockos)")
            .ok_or("Couldn't add ReaEq")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_preset_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        fx.activate_preset(FxPresetRef::Preset(2));
        // Then
        // Should notify since REAPER v6.12+dev0617 ... but maybe not if set programmatically?
        assert_eq!(mock.invocation_count(), 0);
        Ok(())
    })
}

fn check_fx_parameter(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Check fx parameter", move |_, _| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
        // When
        let p = fx.parameter_by_index(5);
        // Then
        assert!(p.is_available());
        assert_eq!(p.name().into_inner().as_c_str(), c_str!("Pitch Wheel"));
        assert_eq!(p.index(), 5);
        assert_eq!(p.character(), FxParameterCharacter::Continuous);
        assert_eq!(p.clone(), p);
        assert_eq!(p.formatted_value().into_inner().as_c_str(), c_str!("0"));
        assert_eq!(
            p.reaper_normalized_value(),
            ReaperNormalizedFxParamValue::new(0.5)
        );
        assert_eq!(
            p.format_reaper_normalized_value(p.reaper_normalized_value())
                .map_err(|_| "Cockos plug-ins should be able to do that")?
                .into_inner()
                .as_c_str(),
            c_str!("0")
        );
        assert_eq!(p.fx(), &fx);
        assert!(p.step_size().is_none());
        assert_eq!(
            p.value_range(),
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
    #[allow(clippy::cognitive_complexity)]
    step(
        AllVersions,
        "Check track fx with 2 fx",
        move |_session, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.track().ok_or("no track")?;
            // When
            let fx_1 = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
            let fx_2 = fx_chain
                .add_fx_by_original_name("ReaSynth (Cockos)")
                .ok_or("Couldn't add ReaSynth")?;
            // Then
            assert!(fx_1.is_available());
            assert!(fx_2.is_available());
            assert_eq!(fx_1.index(), 0);
            assert_eq!(fx_2.index(), 1);
            assert_eq!(
                fx_1.query_index().to_raw(),
                if fx_chain.is_input_fx() {
                    0x0100_0000
                } else {
                    0
                }
            );
            assert_eq!(
                fx_2.query_index().to_raw(),
                if fx_chain.is_input_fx() {
                    0x0100_0001
                } else {
                    1
                }
            );
            assert!(fx_1.guid().is_some());
            assert!(fx_2.guid().is_some());
            assert_eq!(
                fx_1.name().into_inner().as_c_str(),
                c_str!("VST: ReaControlMIDI (Cockos)")
            );
            assert_eq!(
                fx_2.name().into_inner().as_c_str(),
                c_str!("VSTi: ReaSynth (Cockos)")
            );
            let chunk_1 = fx_1.chunk()?;
            assert!(chunk_1.starts_with("BYPASS 0 0 0"));
            if Reaper::get().version() < ReaperVersion::new("6") {
                assert!(chunk_1.ends_with("\nWAK 0"));
            } else {
                assert!(chunk_1.ends_with("\nWAK 0 0"));
            }
            let tag_chunk_1 = fx_1.tag_chunk()?;
            assert!(tag_chunk_1.starts_with(r#"<VST "VST: ReaControlMIDI (Cockos)" reacontrol"#));
            assert!(tag_chunk_1.ends_with("\n>"));
            let state_chunk_1 = fx_1.state_chunk()?;
            assert!(!state_chunk_1.contains("<"));
            assert!(!state_chunk_1.contains(">"));
            let fx_1_info = fx_1.info()?;
            let fx_2_info = fx_2.info()?;
            let fx_1_file_name = fx_1_info
                .file_name
                .file_name()
                .ok_or("FX 1 has no file name")?;
            let fx_2_file_name = fx_2_info
                .file_name
                .file_name()
                .ok_or("FX 2 has no file name")?;
            assert!(matches!(
                fx_1_file_name
                    .to_str()
                    .expect("FX 1 file name is not valid unicode"),
                "reacontrolmidi.dll" | "reacontrolmidi.vst.so" | "reacontrolMIDI.vst.dylib"
            ));
            assert!(matches!(
                fx_2_file_name
                    .to_str()
                    .expect("FX 1 file name is not valid unicode"),
                "reasynth.dll" | "reasynth.vst.so" | "reasynth.vst.dylib"
            ));
            assert_eq!(fx_1.track().ok_or("no track")?, track);
            assert_eq!(fx_2.track().ok_or("no track")?, track);
            assert_eq!(fx_1.is_input_fx(), fx_chain.is_input_fx());
            assert_eq!(fx_2.is_input_fx(), fx_chain.is_input_fx());
            assert_eq!(fx_1.chain(), &fx_chain);
            assert_eq!(fx_2.chain(), &fx_chain);
            assert!(fx_1.parameter_count() >= 17);
            assert!(fx_2.parameter_count() >= 15);
            assert!(fx_1.parameters().count() >= 17);
            assert!(fx_2.parameters().count() >= 15);
            assert!(fx_1.parameter_by_index(15).is_available());
            assert!(!fx_1.parameter_by_index(17).is_available());
            assert!(
                track
                    .fx_by_query_index(if fx_chain.is_input_fx() {
                        0x0100_0000
                    } else {
                        0
                    })
                    .is_some()
            );
            assert!(
                track
                    .fx_by_query_index(if fx_chain.is_input_fx() {
                        0x0100_0001
                    } else {
                        1
                    })
                    .is_some()
            );
            assert!(
                !track
                    .fx_by_query_index(if fx_chain.is_input_fx() {
                        0
                    } else {
                        0x0100_0000
                    })
                    .is_some()
            );
            assert!(
                !track
                    .fx_by_query_index(if fx_chain.is_input_fx() {
                        1
                    } else {
                        0x0100_0001
                    })
                    .is_some()
            );
            if !fx_chain.is_input_fx() {
                let first_instrument_fx = fx_chain
                    .first_instrument_fx()
                    .ok_or("Couldn't find instrument FX")?;
                assert_eq!(first_instrument_fx.index(), 1);
            }
            Ok(())
        },
    )
}

fn enable_track_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Enable track fx", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx_1 = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_enabled_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        fx_1.enable();
        // Then
        assert!(fx_1.is_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), fx_1);
        Ok(())
    })
}

fn disable_track_fx(get_fx_chain: GetFxChain) -> TestStep {
    step(AllVersions, "Disable track fx", move |_, step| {
        // Given
        let fx_chain = get_fx_chain()?;
        let fx_1 = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
        // When
        let (mock, _) = observe_invocations(|mock| {
            Test::control_surface_rx()
                .fx_enabled_changed()
                .take_until(step.finished)
                .subscribe(move |t| {
                    mock.invoke(t);
                });
        });
        fx_1.disable();
        // Then
        assert!(!fx_1.is_enabled());
        assert_eq!(mock.invocation_count(), 1);
        assert_eq!(mock.last_arg(), fx_1);
        Ok(())
    })
}

fn check_track_fx_with_1_fx(get_fx_chain: GetFxChain) -> TestStep {
    #[allow(clippy::cognitive_complexity)]
    step(
        AllVersions,
        "Check track fx with 1 fx",
        move |_session, _| {
            // Given
            let fx_chain = get_fx_chain()?;
            let track = fx_chain.track();
            // When
            let fx_1 = fx_chain.fx_by_index(0).ok_or("Couldn't find first fx")?;
            // Then
            assert!(fx_1.is_available());
            assert_eq!(fx_1.index(), 0);
            assert_eq!(
                fx_1.query_index().to_raw(),
                if fx_chain.is_input_fx() {
                    0x0100_0000
                } else {
                    0
                }
            );
            assert!(fx_1.guid().is_some());
            assert_eq!(
                fx_1.name().into_inner().as_c_str(),
                c_str!("VST: ReaControlMIDI (Cockos)")
            );
            let chunk = fx_1.chunk()?;
            assert!(chunk.starts_with("BYPASS 0 0 0"));
            if Reaper::get().version() < ReaperVersion::new("6") {
                assert!(chunk.ends_with("\nWAK 0"));
            } else {
                assert!(chunk.ends_with("\nWAK 0 0"));
            }
            let tag_chunk = fx_1.tag_chunk()?;
            assert!(tag_chunk.starts_with(r#"<VST "VST: ReaControlMIDI (Cockos)" reacontrol"#));
            assert!(tag_chunk.ends_with("\n>"));
            let state_chunk = fx_1.state_chunk()?;
            assert!(!state_chunk.contains("<"));
            assert!(!state_chunk.contains(">"));

            let fx_1_info = fx_1.info()?;
            let file_name = fx_1_info.file_name.file_name().ok_or("No FX file name")?;
            assert!(matches!(
                file_name
                    .to_str()
                    .expect("FX 1 file name is not valid unicode"),
                "reacontrolmidi.dll" | "reacontrolmidi.vst.so" | "reacontrolMIDI.vst.dylib"
            ));
            assert_eq!(fx_1_info.type_expression, "VST");
            assert_eq!(fx_1_info.sub_type_expression, "VST");
            assert_eq!(fx_1_info.effect_name, "ReaControlMIDI (Cockos)");

            assert_eq!(fx_1.track(), track);
            assert_eq!(fx_1.is_input_fx(), fx_chain.is_input_fx());
            assert_eq!(fx_1.chain(), &fx_chain);
            assert_eq!(fx_1.parameter_count(), 17);
            assert_eq!(fx_1.parameters().count(), 17);
            assert!(fx_1.parameter_by_index(15).is_available());
            assert!(!fx_1.parameter_by_index(17).is_available());
            assert!(fx_1.is_enabled());
            Ok(())
        },
    )
}

fn add_track_fx_by_original_name(get_fx_chain: GetFxChain) -> TestStep {
    step(
        AllVersions,
        "Add track fx by original name",
        move |_, step| {
            // Given
            let fx_chain = get_fx_chain()?;
            // When
            let (mock, _) = observe_invocations(|mock| {
                Test::control_surface_rx()
                    .fx_added()
                    .take_until(step.finished)
                    .subscribe(move |t| {
                        mock.invoke(t);
                    });
            });
            let fx = fx_chain.add_fx_by_original_name("ReaControlMIDI (Cockos)");
            // Then
            assert!(fx.is_some());
            assert_eq!(fx_chain.fx_count(), 1);
            assert_eq!(fx_chain.fxs().count(), 1);
            assert_eq!(fx_chain.fx_by_index(0), fx);
            assert_eq!(fx_chain.first_fx(), fx);
            assert_eq!(fx_chain.last_fx(), fx);
            let fx = fx.unwrap();
            assert_eq!(fx_chain.fx_by_index_untracked(0), fx);
            let guid = fx.guid();
            assert!(guid.is_some());
            let guid = guid.unwrap();
            let guid_string = guid.to_string_without_braces();
            assert_eq!(guid_string.len(), 36);
            assert!(guid_string.find(|c| c == '{' || c == '}').is_none());
            assert!(fx_chain.fx_by_guid(&guid).is_available());
            assert_eq!(fx_chain.fx_by_guid(&guid), fx);
            assert!(fx_chain.fx_by_guid_and_index(&guid, 0).is_available());
            // If this doesn't work, then the index hasn't automatically corrected itself
            assert!(fx_chain.fx_by_guid_and_index(&guid, 1).is_available());
            let non_existing_guid =
                Guid::from_string_with_braces("{E64BB283-FB17-4702-ACFA-2DDB7E38F14F}")?;
            assert!(
                !fx_chain
                    .fx_by_guid_and_index(&non_existing_guid, 0)
                    .is_available()
            );
            assert_eq!(
                fx_chain.first_fx_by_name("ReaControlMIDI (Cockos)"),
                Some(fx.clone())
            );
            let chain_chunk = fx_chain.chunk()?;
            assert!(chain_chunk.is_some());
            let chain_chunk = chain_chunk.unwrap();
            assert!(chain_chunk.starts_with("<FXCHAIN"));
            assert!(chain_chunk.ends_with("\n>"));
            let first_tag = chain_chunk.find_first_tag(0);
            assert!(first_tag.is_some());
            let first_tag = first_tag.unwrap();
            assert_eq!(first_tag.content().deref(), chain_chunk.content().deref());
            assert_eq!(mock.invocation_count(), 1);
            assert_eq!(mock.last_arg(), fx);
            Ok(())
        },
    )
}

fn get_track(index: u32) -> Result<Track, &'static str> {
    Reaper::get()
        .current_project()
        .track_by_index(index)
        .ok_or("Track not found")
}
