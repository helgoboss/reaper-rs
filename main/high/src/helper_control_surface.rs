use std::cell::{Cell, RefCell, RefMut};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::iter::once;

use crossbeam_channel::{Receiver, Sender};
use rxrust::prelude::*;

use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::{
    reaper_str, AutomationMode, ControlSurface, ExtSetBpmAndPlayRateArgs, ExtSetFocusedFxArgs,
    ExtSetFxChangeArgs, ExtSetFxEnabledArgs, ExtSetFxOpenArgs, ExtSetFxParamArgs,
    ExtSetInputMonitorArgs, ExtSetLastTouchedFxArgs, ExtSetSendPanArgs, ExtSetSendVolumeArgs,
    ExtTrackFxPresetChangedArgs, InputMonitoringMode, MediaTrack, ReaProject,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperStr, ReaperVersion, ReaperVolumeValue,
    SetPlayStateArgs, SetRepeatStateArgs, SetSurfaceMuteArgs, SetSurfacePanArgs,
    SetSurfaceRecArmArgs, SetSurfaceSelectedArgs, SetSurfaceSoloArgs, SetSurfaceVolumeArgs,
    SetTrackTitleArgs, TrackFxChainType, TrackLocation, VersionDependentFxLocation,
    VersionDependentTrackFxLocation,
};

use crate::run_loop_scheduler::RxTask;
use crate::{
    local_run_loop_executor, run_loop_executor, ChangeDetectionMiddleware, ChangeEvent,
    ControlSurfaceEvent, ControlSurfaceMiddleware, MainSubjects, MainThreadTask, Project, Reaper,
    MAIN_THREAD_TASK_BULK_SIZE,
};

#[derive(Debug)]
pub(crate) struct HelperMiddleware {
    // These two are for very simple scheduling. Most light-weight.
    main_thread_task_sender: Sender<MainThreadTask>,
    main_thread_task_receiver: Receiver<MainThreadTask>,
    // This is for executing futures.
    main_thread_executor: run_loop_executor::RunLoopExecutor,
    local_main_thread_executor: local_run_loop_executor::RunLoopExecutor,
    // This is for scheduling rxRust observables.
    // TODO-medium Remove, I ran into deadlocks with this thing.
    main_thread_rx_task_receiver: Receiver<RxTask>,
    change_detection_middleware: ChangeDetectionMiddleware,
    subjects: MainSubjects,
}

impl HelperMiddleware {
    pub fn new(
        version: ReaperVersion<'static>,
        last_active_project: Project,
        main_thread_task_sender: Sender<MainThreadTask>,
        main_thread_task_receiver: Receiver<MainThreadTask>,
        main_thread_rx_task_receiver: Receiver<RxTask>,
        executor: run_loop_executor::RunLoopExecutor,
        local_executor: local_run_loop_executor::RunLoopExecutor,
        subjects: MainSubjects,
    ) -> HelperMiddleware {
        HelperMiddleware {
            main_thread_task_sender,
            main_thread_task_receiver,
            main_thread_executor: executor,
            local_main_thread_executor: local_executor,
            main_thread_rx_task_receiver,
            change_detection_middleware: ChangeDetectionMiddleware::new(
                version,
                last_active_project,
            ),
            subjects,
        }
    }

    pub fn reset(&self) {
        self.discard_tasks();
    }

    fn discard_tasks(&self) {
        self.discard_main_thread_tasks();
        self.discard_main_thread_rx_tasks();
        self.discard_future_tasks();
    }

    fn discard_future_tasks(&self) {
        let shared_task_count = self.main_thread_executor.discard_tasks();
        let local_task_count = self.local_main_thread_executor.discard_tasks();
        let total_task_count = shared_task_count + local_task_count;
        if total_task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded future tasks on reactivation";
                "task_count" => total_task_count,
            );
        }
    }

    fn discard_main_thread_tasks(&self) {
        let task_count = self.main_thread_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded main thread tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }

    fn discard_main_thread_rx_tasks(&self) {
        let task_count = self.main_thread_rx_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded main thread rx tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }
}

impl ControlSurfaceMiddleware for HelperMiddleware {
    fn run(&mut self) {
        // Invoke custom idle code
        Reaper::get()
            .subjects
            .main_thread_idle
            .borrow_mut()
            .next(());
        // Process plain main thread tasks in queue
        for task in self
            .main_thread_task_receiver
            .try_iter()
            .take(MAIN_THREAD_TASK_BULK_SIZE)
        {
            match task.desired_execution_time {
                None => (task.op)(),
                Some(t) => {
                    if std::time::SystemTime::now() < t {
                        self.main_thread_task_sender
                            .send(task)
                            .expect("couldn't reschedule main thread task");
                    } else {
                        (task.op)()
                    }
                }
            }
        }
        // Execute futures
        self.main_thread_executor.run();
        self.local_main_thread_executor.run();
        // Execute observables
        for task in self
            .main_thread_rx_task_receiver
            .try_iter()
            .take(MAIN_THREAD_TASK_BULK_SIZE)
        {
            task();
        }
    }

    fn handle_event(&self, event: ControlSurfaceEvent) {
        self.change_detection_middleware.process(event, |event| {
            use ChangeEvent::*;
            match event {
                ProjectSwitched(p) => self.subjects.project_switched.borrow_mut().next(p),
                TrackVolumeChanged(t) => self.subjects.track_volume_changed.borrow_mut().next(t),
                TrackVolumeTouched(t) => self.subjects.track_volume_touched.borrow_mut().next(t),
                TrackPanChanged(t) => self.subjects.track_pan_changed.borrow_mut().next(t),
                TrackPanTouched(t) => self.subjects.track_pan_touched.borrow_mut().next(t),
                TrackSendVolumeChanged(ts) => self
                    .subjects
                    .track_send_volume_changed
                    .borrow_mut()
                    .next(ts),
                TrackSendVolumeTouched(ts) => self
                    .subjects
                    .track_send_volume_touched
                    .borrow_mut()
                    .next(ts),
                TrackSendPanChanged(ts) => {
                    self.subjects.track_send_pan_changed.borrow_mut().next(ts)
                }
                TrackSendPanTouched(ts) => {
                    self.subjects.track_send_pan_touched.borrow_mut().next(ts)
                }
                TrackAdded(t) => self.subjects.track_added.borrow_mut().next(t),
                TrackRemoved(t) => self.subjects.track_removed.borrow_mut().next(t),
                TracksReordered(p) => self.subjects.tracks_reordered.borrow_mut().next(p),
                TrackNameChanged(t) => self.subjects.track_name_changed.borrow_mut().next(t),
                TrackInputChanged(t) => self.subjects.track_input_changed.borrow_mut().next(t),
                TrackInputMonitoringChanged(t) => self
                    .subjects
                    .track_input_monitoring_changed
                    .borrow_mut()
                    .next(t),
                TrackArmChanged(t) => self.subjects.track_arm_changed.borrow_mut().next(t),
                TrackMuteChanged(t) => self.subjects.track_mute_changed.borrow_mut().next(t),
                TrackMuteTouched(t) => self.subjects.track_mute_touched.borrow_mut().next(t),
                TrackSoloChanged(t) => self.subjects.track_solo_changed.borrow_mut().next(t),
                TrackSelectedChanged(t) => {
                    self.subjects.track_selected_changed.borrow_mut().next(t)
                }
                FxAdded(f) => self.subjects.fx_added.borrow_mut().next(f),
                FxRemoved(f) => self.subjects.fx_removed.borrow_mut().next(f),
                FxEnabledChanged(f) => self.subjects.fx_enabled_changed.borrow_mut().next(f),
                FxOpened(f) => self.subjects.fx_opened.borrow_mut().next(f),
                FxClosed(f) => self.subjects.fx_closed.borrow_mut().next(f),
                FxFocused(f) => self.subjects.fx_focused.borrow_mut().next(f),
                FxReordered(t) => self.subjects.fx_reordered.borrow_mut().next(t),
                FxParameterValueChanged(p) => self
                    .subjects
                    .fx_parameter_value_changed
                    .borrow_mut()
                    .next(p),
                FxParameterTouched(p) => self.subjects.fx_parameter_touched.borrow_mut().next(p),
                FxPresetChanged(f) => self.subjects.fx_preset_changed.borrow_mut().next(f),
                MasterTempoChanged => self.subjects.master_tempo_changed.borrow_mut().next(()),
                MasterTempoTouched => self.subjects.master_tempo_touched.borrow_mut().next(()),
                MasterPlayrateChanged => {
                    self.subjects.master_playrate_changed.borrow_mut().next(())
                }
                MasterPlayrateTouched => {
                    self.subjects.master_playrate_touched.borrow_mut().next(())
                }
                PlayStateChanged => self.subjects.play_state_changed.borrow_mut().next(()),
                RepeatStateChanged => self.subjects.repeat_state_changed.borrow_mut().next(()),
                ProjectClosed(p) => self.subjects.project_closed.borrow_mut().next(p),
            };
        });
    }
}
