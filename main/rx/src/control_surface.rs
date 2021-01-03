use crate::{EventStreamSubject, ReactiveEvent};
use reaper_high::{ChangeEvent, Fx, FxParameter, Project, Track, TrackSend};
use rxrust::prelude::*;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug)]
pub struct ControlSurfaceRxDriver {
    rx: ControlSurfaceRx,
}

impl ControlSurfaceRxDriver {
    pub fn new(rx: ControlSurfaceRx) -> ControlSurfaceRxDriver {
        ControlSurfaceRxDriver { rx }
    }

    pub fn run(&self) {
        self.rx.main_thread_idle.borrow_mut().next(());
    }

    pub fn handle_change(&self, event: ChangeEvent) {
        use ChangeEvent::*;
        match event {
            ProjectSwitched(p) => self.rx.project_switched.borrow_mut().next(p),
            TrackVolumeChanged(t) => self.rx.track_volume_changed.borrow_mut().next(t),
            TrackVolumeTouched(t) => self.rx.track_volume_touched.borrow_mut().next(t),
            TrackPanChanged(t) => self.rx.track_pan_changed.borrow_mut().next(t),
            TrackPanTouched(t) => self.rx.track_pan_touched.borrow_mut().next(t),
            TrackSendVolumeChanged(ts) => self.rx.track_send_volume_changed.borrow_mut().next(ts),
            TrackSendVolumeTouched(ts) => self.rx.track_send_volume_touched.borrow_mut().next(ts),
            TrackSendPanChanged(ts) => self.rx.track_send_pan_changed.borrow_mut().next(ts),
            TrackSendPanTouched(ts) => self.rx.track_send_pan_touched.borrow_mut().next(ts),
            TrackAdded(t) => self.rx.track_added.borrow_mut().next(t),
            TrackRemoved(t) => self.rx.track_removed.borrow_mut().next(t),
            TracksReordered(p) => self.rx.tracks_reordered.borrow_mut().next(p),
            TrackNameChanged(t) => self.rx.track_name_changed.borrow_mut().next(t),
            TrackInputChanged(t) => self.rx.track_input_changed.borrow_mut().next(t),
            TrackInputMonitoringChanged(t) => {
                self.rx.track_input_monitoring_changed.borrow_mut().next(t)
            }
            TrackArmChanged(t) => self.rx.track_arm_changed.borrow_mut().next(t),
            TrackMuteChanged(t) => self.rx.track_mute_changed.borrow_mut().next(t),
            TrackMuteTouched(t) => self.rx.track_mute_touched.borrow_mut().next(t),
            TrackSoloChanged(t) => self.rx.track_solo_changed.borrow_mut().next(t),
            TrackSelectedChanged(t) => self.rx.track_selected_changed.borrow_mut().next(t),
            FxAdded(f) => self.rx.fx_added.borrow_mut().next(f),
            FxRemoved(f) => self.rx.fx_removed.borrow_mut().next(f),
            FxEnabledChanged(f) => self.rx.fx_enabled_changed.borrow_mut().next(f),
            FxOpened(f) => self.rx.fx_opened.borrow_mut().next(f),
            FxClosed(f) => self.rx.fx_closed.borrow_mut().next(f),
            FxFocused(f) => self.rx.fx_focused.borrow_mut().next(f),
            FxReordered(t) => self.rx.fx_reordered.borrow_mut().next(t),
            FxParameterValueChanged(p) => self.rx.fx_parameter_value_changed.borrow_mut().next(p),
            FxParameterTouched(p) => self.rx.fx_parameter_touched.borrow_mut().next(p),
            FxPresetChanged(f) => self.rx.fx_preset_changed.borrow_mut().next(f),
            MasterTempoChanged => self.rx.master_tempo_changed.borrow_mut().next(()),
            MasterTempoTouched => self.rx.master_tempo_touched.borrow_mut().next(()),
            MasterPlayrateChanged => self.rx.master_playrate_changed.borrow_mut().next(()),
            MasterPlayrateTouched => self.rx.master_playrate_touched.borrow_mut().next(()),
            PlayStateChanged => self.rx.play_state_changed.borrow_mut().next(()),
            RepeatStateChanged => self.rx.repeat_state_changed.borrow_mut().next(()),
            ProjectClosed(p) => self.rx.project_closed.borrow_mut().next(p),
        };
    }
}

#[derive(Clone, Default)]
pub struct ControlSurfaceRx {
    pub(crate) main_thread_idle: EventStreamSubject<()>,
    pub(crate) project_switched: EventStreamSubject<Project>,
    pub(crate) track_volume_changed: EventStreamSubject<Track>,
    pub(crate) track_volume_touched: EventStreamSubject<Track>,
    pub(crate) track_pan_changed: EventStreamSubject<Track>,
    pub(crate) track_pan_touched: EventStreamSubject<Track>,
    pub(crate) track_send_volume_changed: EventStreamSubject<TrackSend>,
    pub(crate) track_send_volume_touched: EventStreamSubject<TrackSend>,
    pub(crate) track_send_pan_changed: EventStreamSubject<TrackSend>,
    pub(crate) track_send_pan_touched: EventStreamSubject<TrackSend>,
    pub(crate) track_added: EventStreamSubject<Track>,
    pub(crate) track_removed: EventStreamSubject<Track>,
    pub(crate) tracks_reordered: EventStreamSubject<Project>,
    pub(crate) track_name_changed: EventStreamSubject<Track>,
    pub(crate) track_input_changed: EventStreamSubject<Track>,
    pub(crate) track_input_monitoring_changed: EventStreamSubject<Track>,
    pub(crate) track_arm_changed: EventStreamSubject<Track>,
    pub(crate) track_mute_changed: EventStreamSubject<Track>,
    pub(crate) track_mute_touched: EventStreamSubject<Track>,
    pub(crate) track_solo_changed: EventStreamSubject<Track>,
    pub(crate) track_selected_changed: EventStreamSubject<Track>,
    pub(crate) fx_added: EventStreamSubject<Fx>,
    pub(crate) fx_removed: EventStreamSubject<Fx>,
    pub(crate) fx_enabled_changed: EventStreamSubject<Fx>,
    pub(crate) fx_opened: EventStreamSubject<Fx>,
    pub(crate) fx_closed: EventStreamSubject<Fx>,
    pub(crate) fx_focused: EventStreamSubject<Option<Fx>>,
    pub(crate) fx_reordered: EventStreamSubject<Track>,
    pub(crate) fx_parameter_value_changed: EventStreamSubject<FxParameter>,
    pub(crate) fx_parameter_touched: EventStreamSubject<FxParameter>,
    pub(crate) fx_preset_changed: EventStreamSubject<Fx>,
    pub(crate) master_tempo_changed: EventStreamSubject<()>,
    pub(crate) master_tempo_touched: EventStreamSubject<()>,
    pub(crate) master_playrate_changed: EventStreamSubject<()>,
    pub(crate) master_playrate_touched: EventStreamSubject<()>,
    pub(crate) play_state_changed: EventStreamSubject<()>,
    pub(crate) repeat_state_changed: EventStreamSubject<()>,
    pub(crate) project_closed: EventStreamSubject<Project>,
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
            track_volume_changed: default(),
            track_volume_touched: default(),
            track_pan_changed: default(),
            track_pan_touched: default(),
            track_send_volume_changed: default(),
            track_send_volume_touched: default(),
            track_send_pan_changed: default(),
            track_send_pan_touched: default(),
            track_added: default(),
            track_removed: default(),
            tracks_reordered: default(),
            track_name_changed: default(),
            track_input_changed: default(),
            track_input_monitoring_changed: default(),
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
        }
    }

    pub fn project_switched(&self) -> impl ReactiveEvent<Project> {
        self.project_switched.borrow().clone()
    }

    pub fn fx_opened(&self) -> impl ReactiveEvent<Fx> {
        self.fx_opened.borrow().clone()
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

    pub fn track_pan_touched(&self) -> impl ReactiveEvent<Track> {
        self.track_pan_touched.borrow().clone()
    }

    pub fn track_selected_changed(&self) -> impl ReactiveEvent<Track> {
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

    pub fn track_send_volume_changed(&self) -> impl ReactiveEvent<TrackSend> {
        self.track_send_volume_changed.borrow().clone()
    }

    pub fn track_send_volume_touched(&self) -> impl ReactiveEvent<TrackSend> {
        self.track_send_volume_touched.borrow().clone()
    }

    pub fn track_send_pan_changed(&self) -> impl ReactiveEvent<TrackSend> {
        self.track_send_pan_changed.borrow().clone()
    }

    pub fn track_send_pan_touched(&self) -> impl ReactiveEvent<TrackSend> {
        self.track_send_pan_touched.borrow().clone()
    }

    /// Only fires if `run()` is called on the driver.
    pub fn main_thread_idle(&self) -> impl ReactiveEvent<()> {
        self.main_thread_idle.borrow().clone()
    }
}
