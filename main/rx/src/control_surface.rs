use crate::{EventStreamSubject, ReactiveEvent};
use reaper_high::{AvailablePanValue, ChangeEvent, Fx, FxParameter, Project, Track, TrackRoute};
use reaper_medium::Pan;
use rxrust::prelude::*;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug)]
pub struct ControlSurfaceRxMiddleware {
    rx: ControlSurfaceRx,
}

impl ControlSurfaceRxMiddleware {
    pub fn new(rx: ControlSurfaceRx) -> ControlSurfaceRxMiddleware {
        ControlSurfaceRxMiddleware { rx }
    }

    pub fn run(&self) {
        self.rx.main_thread_idle.borrow_mut().next(());
    }

    pub fn handle_change(&self, event: ChangeEvent) {
        use ChangeEvent::*;
        match event {
            ProjectSwitched(e) => self.rx.project_switched.borrow_mut().next(e.new_project),
            TrackVolumeChanged(e) => {
                self.rx
                    .track_volume_changed
                    .borrow_mut()
                    .next(e.track.clone());
                if e.touched {
                    self.rx.track_volume_touched.borrow_mut().next(e.track);
                }
            }
            TrackPanChanged(e) => {
                self.rx.track_pan_changed.borrow_mut().next(e.track.clone());
                if e.touched {
                    // When it's touched, it should always be complete.
                    if let AvailablePanValue::Complete(new_value) = e.new_value {
                        self.rx.track_pan_touched.borrow_mut().next((
                            e.track,
                            e.old_value,
                            new_value,
                        ));
                    }
                }
            }
            TrackRouteVolumeChanged(e) => {
                self.rx
                    .track_route_volume_changed
                    .borrow_mut()
                    .next(e.route.clone());
                if e.touched {
                    self.rx
                        .track_route_volume_touched
                        .borrow_mut()
                        .next(e.route);
                }
            }
            TrackRoutePanChanged(e) => {
                self.rx
                    .track_route_pan_changed
                    .borrow_mut()
                    .next(e.route.clone());
                if e.touched {
                    self.rx.track_route_pan_touched.borrow_mut().next(e.route);
                }
            }
            TrackAdded(e) => self.rx.track_added.borrow_mut().next(e.track),
            TrackRemoved(e) => self.rx.track_removed.borrow_mut().next(e.track),
            TracksReordered(e) => self.rx.tracks_reordered.borrow_mut().next(e.project),
            TrackNameChanged(e) => self.rx.track_name_changed.borrow_mut().next(e.track),
            TrackInputChanged(e) => self.rx.track_input_changed.borrow_mut().next(e.track),
            TrackInputMonitoringChanged(e) => self
                .rx
                .track_input_monitoring_changed
                .borrow_mut()
                .next(e.track),
            TrackAutomationModeChanged(e) => self
                .rx
                .track_automation_mode_changed
                .borrow_mut()
                .next(e.track),
            TrackArmChanged(e) => self.rx.track_arm_changed.borrow_mut().next(e.track),
            TrackMuteChanged(e) => {
                self.rx
                    .track_mute_changed
                    .borrow_mut()
                    .next(e.track.clone());
                if e.touched {
                    self.rx.track_mute_touched.borrow_mut().next(e.track);
                }
            }
            TrackSoloChanged(e) => self.rx.track_solo_changed.borrow_mut().next(e.track),
            TrackSelectedChanged(e) => self
                .rx
                .track_selected_changed
                .borrow_mut()
                .next((e.track, e.new_value)),
            FxAdded(e) => self.rx.fx_added.borrow_mut().next(e.fx),
            FxRemoved(e) => self.rx.fx_removed.borrow_mut().next(e.fx),
            FxEnabledChanged(e) => self.rx.fx_enabled_changed.borrow_mut().next(e.fx),
            FxOpened(e) => self.rx.fx_opened.borrow_mut().next(e.fx),
            FxClosed(e) => self.rx.fx_closed.borrow_mut().next(e.fx),
            FxFocused(e) => self.rx.fx_focused.borrow_mut().next(e.fx),
            FxReordered(e) => self.rx.fx_reordered.borrow_mut().next(e.track),
            FxParameterValueChanged(e) => {
                self.rx
                    .fx_parameter_value_changed
                    .borrow_mut()
                    .next(e.parameter.clone());
                if e.touched {
                    self.rx.fx_parameter_touched.borrow_mut().next(e.parameter);
                }
            }
            FxPresetChanged(e) => self.rx.fx_preset_changed.borrow_mut().next(e.fx),
            MasterTempoChanged(e) => {
                self.rx.master_tempo_changed.borrow_mut().next(());
                if e.touched {
                    self.rx.master_tempo_touched.borrow_mut().next(());
                }
            }
            MasterPlayrateChanged(e) => {
                self.rx.master_playrate_changed.borrow_mut().next(());
                if e.touched {
                    self.rx.master_playrate_touched.borrow_mut().next(());
                }
            }
            PlayStateChanged(_) => self.rx.play_state_changed.borrow_mut().next(()),
            RepeatStateChanged(_) => self.rx.repeat_state_changed.borrow_mut().next(()),
            ProjectClosed(e) => self.rx.project_closed.borrow_mut().next(e.project),
            GlobalAutomationOverrideChanged(_) => self
                .rx
                .global_automation_override_changed
                .borrow_mut()
                .next(()),
            BookmarksChanged(_) => self.rx.bookmarks_changed.borrow_mut().next(()),
            ReceiveCountChanged(e) => self.rx.receive_count_changed.borrow_mut().next(e.track),
            HardwareOutputSendCountChanged(e) => self
                .rx
                .hardware_output_send_count_changed
                .borrow_mut()
                .next(e.track),
            TrackSendCountChanged(e) => self.rx.track_send_count_changed.borrow_mut().next(e.track),
        };
    }
}

#[derive(Clone, Default)]
pub struct ControlSurfaceRx {
    pub main_thread_idle: EventStreamSubject<()>,
    pub project_switched: EventStreamSubject<Project>,
    pub global_automation_override_changed: EventStreamSubject<()>,
    pub track_volume_changed: EventStreamSubject<Track>,
    pub track_volume_touched: EventStreamSubject<Track>,
    pub track_pan_changed: EventStreamSubject<Track>,
    /// Old, New.
    pub track_pan_touched: EventStreamSubject<(Track, Pan, Pan)>,
    pub track_route_volume_changed: EventStreamSubject<TrackRoute>,
    pub track_route_volume_touched: EventStreamSubject<TrackRoute>,
    pub track_route_pan_changed: EventStreamSubject<TrackRoute>,
    pub track_route_pan_touched: EventStreamSubject<TrackRoute>,
    pub track_added: EventStreamSubject<Track>,
    pub track_removed: EventStreamSubject<Track>,
    pub tracks_reordered: EventStreamSubject<Project>,
    pub receive_count_changed: EventStreamSubject<Track>,
    pub track_send_count_changed: EventStreamSubject<Track>,
    pub hardware_output_send_count_changed: EventStreamSubject<Track>,
    pub track_name_changed: EventStreamSubject<Track>,
    pub track_input_changed: EventStreamSubject<Track>,
    pub track_input_monitoring_changed: EventStreamSubject<Track>,
    pub track_automation_mode_changed: EventStreamSubject<Track>,
    pub track_arm_changed: EventStreamSubject<Track>,
    pub track_mute_changed: EventStreamSubject<Track>,
    pub track_mute_touched: EventStreamSubject<Track>,
    pub track_solo_changed: EventStreamSubject<Track>,
    pub track_selected_changed: EventStreamSubject<(Track, bool)>,
    pub fx_added: EventStreamSubject<Fx>,
    pub fx_removed: EventStreamSubject<Fx>,
    pub fx_enabled_changed: EventStreamSubject<Fx>,
    pub fx_opened: EventStreamSubject<Fx>,
    pub fx_closed: EventStreamSubject<Fx>,
    pub fx_focused: EventStreamSubject<Option<Fx>>,
    pub fx_reordered: EventStreamSubject<Track>,
    pub fx_parameter_value_changed: EventStreamSubject<FxParameter>,
    pub fx_parameter_touched: EventStreamSubject<FxParameter>,
    pub fx_preset_changed: EventStreamSubject<Fx>,
    pub master_tempo_changed: EventStreamSubject<()>,
    pub master_tempo_touched: EventStreamSubject<()>,
    pub master_playrate_changed: EventStreamSubject<()>,
    pub master_playrate_touched: EventStreamSubject<()>,
    pub play_state_changed: EventStreamSubject<()>,
    pub repeat_state_changed: EventStreamSubject<()>,
    pub project_closed: EventStreamSubject<Project>,
    pub bookmarks_changed: EventStreamSubject<()>,
}

impl fmt::Debug for ControlSurfaceRx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlSurfaceRx").finish()
    }
}

impl ControlSurfaceRx {
    pub fn new() -> ControlSurfaceRx {
        fn default<T>() -> EventStreamSubject<T> {
            RefCell::new(LocalSubject::new())
        }
        ControlSurfaceRx {
            main_thread_idle: default(),
            project_switched: default(),
            global_automation_override_changed: default(),
            track_volume_changed: default(),
            track_volume_touched: default(),
            track_pan_changed: default(),
            track_pan_touched: default(),
            track_route_volume_changed: default(),
            track_route_volume_touched: default(),
            track_route_pan_changed: default(),
            track_route_pan_touched: default(),
            track_added: default(),
            track_removed: default(),
            tracks_reordered: default(),
            receive_count_changed: default(),
            track_send_count_changed: default(),
            hardware_output_send_count_changed: default(),
            track_name_changed: default(),
            track_input_changed: default(),
            track_input_monitoring_changed: default(),
            track_automation_mode_changed: default(),
            track_arm_changed: default(),
            track_mute_changed: default(),
            track_mute_touched: default(),
            track_solo_changed: default(),
            track_selected_changed: default(),
            fx_added: default(),
            fx_removed: default(),
            fx_enabled_changed: default(),
            fx_opened: default(),
            fx_closed: default(),
            fx_focused: default(),
            fx_reordered: default(),
            fx_parameter_value_changed: default(),
            fx_parameter_touched: default(),
            fx_preset_changed: default(),
            master_tempo_changed: default(),
            master_tempo_touched: default(),
            master_playrate_changed: default(),
            master_playrate_touched: default(),
            play_state_changed: default(),
            repeat_state_changed: default(),
            project_closed: default(),
            bookmarks_changed: default(),
        }
    }

    pub fn project_switched(&self) -> impl ReactiveEvent<Project> {
        self.project_switched.borrow().clone()
    }

    pub fn global_automation_override_changed(&self) -> impl ReactiveEvent<()> {
        self.global_automation_override_changed.borrow().clone()
    }

    pub fn bookmarks_changed(&self) -> impl ReactiveEvent<()> {
        self.bookmarks_changed.borrow().clone()
    }

    pub fn fx_opened(&self) -> impl ReactiveEvent<Fx> {
        self.fx_opened.borrow().clone()
    }

    pub fn fx_closed(&self) -> impl ReactiveEvent<Fx> {
        self.fx_closed.borrow().clone()
    }

    pub fn fx_focused(&self) -> impl ReactiveEvent<Option<Fx>> {
        self.fx_focused.borrow().clone()
    }

    pub fn track_added(&self) -> impl ReactiveEvent<Track> {
        self.track_added.borrow().clone()
    }

    // Delivers a GUID-based track (to still be able to identify it even it is deleted)
    pub fn track_removed(&self) -> impl ReactiveEvent<Track> {
        self.track_removed.borrow().clone()
    }

    pub fn tracks_reordered(&self) -> impl ReactiveEvent<Project> {
        self.tracks_reordered.borrow().clone()
    }

    pub fn receive_count_changed(&self) -> impl ReactiveEvent<Track> {
        self.receive_count_changed.borrow().clone()
    }

    pub fn track_send_count_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_send_count_changed.borrow().clone()
    }

    pub fn hardware_output_send_count_changed(&self) -> impl ReactiveEvent<Track> {
        self.hardware_output_send_count_changed.borrow().clone()
    }

    pub fn track_name_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_name_changed.borrow().clone()
    }

    pub fn master_tempo_changed(&self) -> impl ReactiveEvent<()> {
        self.master_tempo_changed.borrow().clone()
    }

    pub fn master_tempo_touched(&self) -> impl ReactiveEvent<()> {
        self.master_tempo_touched.borrow().clone()
    }

    pub fn master_playrate_changed(&self) -> impl ReactiveEvent<()> {
        self.master_playrate_changed.borrow().clone()
    }

    pub fn master_playrate_touched(&self) -> impl ReactiveEvent<()> {
        self.master_playrate_touched.borrow().clone()
    }

    pub fn play_state_changed(&self) -> impl ReactiveEvent<()> {
        self.play_state_changed.borrow().clone()
    }

    pub fn repeat_state_changed(&self) -> impl ReactiveEvent<()> {
        self.repeat_state_changed.borrow().clone()
    }

    pub fn fx_added(&self) -> impl ReactiveEvent<Fx> {
        self.fx_added.borrow().clone()
    }

    pub fn fx_enabled_changed(&self) -> impl ReactiveEvent<Fx> {
        self.fx_enabled_changed.borrow().clone()
    }

    pub fn fx_reordered(&self) -> impl ReactiveEvent<Track> {
        self.fx_reordered.borrow().clone()
    }

    pub fn fx_removed(&self) -> impl ReactiveEvent<Fx> {
        self.fx_removed.borrow().clone()
    }

    pub fn fx_parameter_value_changed(&self) -> impl ReactiveEvent<FxParameter> {
        self.fx_parameter_value_changed.borrow().clone()
    }

    pub fn fx_parameter_touched(&self) -> impl ReactiveEvent<FxParameter> {
        self.fx_parameter_touched.borrow().clone()
    }

    pub fn fx_preset_changed(&self) -> impl ReactiveEvent<Fx> {
        self.fx_preset_changed.borrow().clone()
    }

    pub fn track_input_monitoring_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_input_monitoring_changed.borrow().clone()
    }

    pub fn track_automation_mode_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_automation_mode_changed.borrow().clone()
    }

    pub fn track_input_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_input_changed.borrow().clone()
    }

    pub fn track_volume_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_volume_changed.borrow().clone()
    }

    pub fn track_volume_touched(&self) -> impl ReactiveEvent<Track> {
        self.track_volume_touched.borrow().clone()
    }

    pub fn track_pan_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_pan_changed.borrow().clone()
    }

    /// Old, new
    pub fn track_pan_touched(&self) -> impl ReactiveEvent<(Track, Pan, Pan)> {
        self.track_pan_touched.borrow().clone()
    }

    /// New
    pub fn track_selected_changed(&self) -> impl ReactiveEvent<(Track, bool)> {
        self.track_selected_changed.borrow().clone()
    }

    pub fn track_mute_changed(&self) -> impl ReactiveEvent<Track> {
        // TODO-medium Use try_borrow() and emit a helpful error message, e.g.
        //  "Don't subscribe to an event x while this event is raised! Defer the subscription."
        self.track_mute_changed.borrow().clone()
    }

    pub fn track_mute_touched(&self) -> impl ReactiveEvent<Track> {
        self.track_mute_touched.borrow().clone()
    }

    pub fn track_solo_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_solo_changed.borrow().clone()
    }

    pub fn track_arm_changed(&self) -> impl ReactiveEvent<Track> {
        self.track_arm_changed.borrow().clone()
    }

    pub fn track_route_volume_changed(&self) -> impl ReactiveEvent<TrackRoute> {
        self.track_route_volume_changed.borrow().clone()
    }

    pub fn track_route_volume_touched(&self) -> impl ReactiveEvent<TrackRoute> {
        self.track_route_volume_touched.borrow().clone()
    }

    pub fn track_route_pan_changed(&self) -> impl ReactiveEvent<TrackRoute> {
        self.track_route_pan_changed.borrow().clone()
    }

    pub fn track_route_pan_touched(&self) -> impl ReactiveEvent<TrackRoute> {
        self.track_route_pan_touched.borrow().clone()
    }

    /// Only fires if `run()` is called on the driver.
    pub fn main_thread_idle(&self) -> impl ReactiveEvent<()> {
        self.main_thread_idle.borrow().clone()
    }
}
