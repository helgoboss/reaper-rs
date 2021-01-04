use crate::ControlSurfaceEvent;
use metered::hdr_histogram::HdrHistogram;
use metered::measure;
use metered::metric::Histogram;
use metered::time_source::{Instant, StdInstantMicros};
use metered::ResponseTime;
use serde::{Serialize, Serializer};
use std::cell::RefCell;

type CustomResponseTime = ResponseTime<RefCell<HdrHistogram>, StdInstantMicros>;

#[derive(Debug, Default)]
pub struct MeterControlSurfaceMiddleware {
    metrics: ControlSurfaceMiddlewareMetrics,
}

#[derive(Debug, Default, Serialize)]
pub struct ControlSurfaceMiddlewareMetrics {
    run: CustomResponseTime,
    close_no_reset: CustomResponseTime,
    set_track_list_change: CustomResponseTime,
    set_surface_volume: CustomResponseTime,
    set_surface_pan: CustomResponseTime,
    set_surface_mute: CustomResponseTime,
    set_surface_selected: CustomResponseTime,
    set_surface_solo: CustomResponseTime,
    set_surface_rec_arm: CustomResponseTime,
    set_play_state: CustomResponseTime,
    set_repeat_state: CustomResponseTime,
    set_track_title: CustomResponseTime,
    set_auto_mode: CustomResponseTime,
    reset_cached_vol_pan_states: CustomResponseTime,
    on_track_selection: CustomResponseTime,
    ext_set_input_monitor: CustomResponseTime,
    ext_set_fx_param: CustomResponseTime,
    ext_set_fx_param_rec_fx: CustomResponseTime,
    ext_set_fx_enabled: CustomResponseTime,
    ext_set_send_volume: CustomResponseTime,
    ext_set_send_pan: CustomResponseTime,
    ext_set_focused_fx: CustomResponseTime,
    ext_set_last_touched_fx: CustomResponseTime,
    ext_set_fx_open: CustomResponseTime,
    ext_set_fx_change: CustomResponseTime,
    ext_set_bpm_and_play_rate: CustomResponseTime,
    ext_track_fx_preset_changed: CustomResponseTime,
}

impl ControlSurfaceMiddlewareMetrics {
    pub fn take_snapshot(&self) -> ControlSurfaceMiddlewareMetricsSnapshot {
        ControlSurfaceMiddlewareMetricsSnapshot {
            run: self.run.take_snapshot(),
            close_no_reset: self.close_no_reset.take_snapshot(),
            set_track_list_change: self.set_track_list_change.take_snapshot(),
            set_surface_volume: self.set_surface_volume.take_snapshot(),
            set_surface_pan: self.set_surface_pan.take_snapshot(),
            set_surface_mute: self.set_surface_mute.take_snapshot(),
            set_surface_selected: self.set_surface_selected.take_snapshot(),
            set_surface_solo: self.set_surface_solo.take_snapshot(),
            set_surface_rec_arm: self.set_surface_rec_arm.take_snapshot(),
            set_play_state: self.set_play_state.take_snapshot(),
            set_repeat_state: self.set_repeat_state.take_snapshot(),
            set_track_title: self.set_track_title.take_snapshot(),
            set_auto_mode: self.set_auto_mode.take_snapshot(),
            reset_cached_vol_pan_states: self.reset_cached_vol_pan_states.take_snapshot(),
            on_track_selection: self.on_track_selection.take_snapshot(),
            ext_set_input_monitor: self.ext_set_input_monitor.take_snapshot(),
            ext_set_fx_param: self.ext_set_fx_param.take_snapshot(),
            ext_set_fx_param_rec_fx: self.ext_set_fx_param_rec_fx.take_snapshot(),
            ext_set_fx_enabled: self.ext_set_fx_enabled.take_snapshot(),
            ext_set_send_volume: self.ext_set_send_volume.take_snapshot(),
            ext_set_send_pan: self.ext_set_send_pan.take_snapshot(),
            ext_set_focused_fx: self.ext_set_focused_fx.take_snapshot(),
            ext_set_last_touched_fx: self.ext_set_last_touched_fx.take_snapshot(),
            ext_set_fx_open: self.ext_set_fx_open.take_snapshot(),
            ext_set_fx_change: self.ext_set_fx_change.take_snapshot(),
            ext_set_bpm_and_play_rate: self.ext_set_bpm_and_play_rate.take_snapshot(),
            ext_track_fx_preset_changed: self.ext_track_fx_preset_changed.take_snapshot(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ControlSurfaceMiddlewareMetricsSnapshot {
    run: ResponseTimeSnapshot,
    close_no_reset: ResponseTimeSnapshot,
    set_track_list_change: ResponseTimeSnapshot,
    set_surface_volume: ResponseTimeSnapshot,
    set_surface_pan: ResponseTimeSnapshot,
    set_surface_mute: ResponseTimeSnapshot,
    set_surface_selected: ResponseTimeSnapshot,
    set_surface_solo: ResponseTimeSnapshot,
    set_surface_rec_arm: ResponseTimeSnapshot,
    set_play_state: ResponseTimeSnapshot,
    set_repeat_state: ResponseTimeSnapshot,
    set_track_title: ResponseTimeSnapshot,
    set_auto_mode: ResponseTimeSnapshot,
    reset_cached_vol_pan_states: ResponseTimeSnapshot,
    on_track_selection: ResponseTimeSnapshot,
    ext_set_input_monitor: ResponseTimeSnapshot,
    ext_set_fx_param: ResponseTimeSnapshot,
    ext_set_fx_param_rec_fx: ResponseTimeSnapshot,
    ext_set_fx_enabled: ResponseTimeSnapshot,
    ext_set_send_volume: ResponseTimeSnapshot,
    ext_set_send_pan: ResponseTimeSnapshot,
    ext_set_focused_fx: ResponseTimeSnapshot,
    ext_set_last_touched_fx: ResponseTimeSnapshot,
    ext_set_fx_open: ResponseTimeSnapshot,
    ext_set_fx_change: ResponseTimeSnapshot,
    ext_set_bpm_and_play_rate: ResponseTimeSnapshot,
    ext_track_fx_preset_changed: ResponseTimeSnapshot,
}

trait TakeSnapshot {
    fn take_snapshot(&self) -> ResponseTimeSnapshot;
}

impl TakeSnapshot for CustomResponseTime {
    fn take_snapshot(&self) -> ResponseTimeSnapshot {
        let hist = self.0.borrow();
        ResponseTimeSnapshot {
            min: hist.min(),
            max: hist.max(),
            mean: hist.mean(),
            stdev: hist.stdev(),
            p90: hist.p90(),
            p95: hist.p95(),
            p99: hist.p99(),
            p999: hist.p999(),
            p9999: hist.p9999(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ResponseTimeSnapshot {
    min: u64,
    max: u64,
    mean: f64,
    stdev: f64,
    p90: u64,
    p95: u64,
    p99: u64,
    p999: u64,
    p9999: u64,
}

/// See `hdr_histogram.rs` in `metered-rs`for explanation.
impl Serialize for ResponseTimeSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        macro_rules! ile {
            ($e:expr, $r:expr) => {
                &MetricAlias(concat!("!|quantile=", $e), $r)
            };
        }
        macro_rules! qual {
            ($e:expr) => {
                &MetricAlias("<|", $e)
            };
        }
        use serde::ser::SerializeMap;
        let mut tup = serializer.serialize_map(Some(9))?;
        tup.serialize_entry("min", qual!(self.min))?;
        tup.serialize_entry("max", qual!(self.max))?;
        tup.serialize_entry("mean", qual!(self.mean))?;
        tup.serialize_entry("stdev", qual!(self.stdev))?;
        tup.serialize_entry("90%ile", ile!(0.9, self.p90))?;
        tup.serialize_entry("95%ile", ile!(0.95, self.p95))?;
        tup.serialize_entry("99%ile", ile!(0.99, self.p99))?;
        tup.serialize_entry("99.9%ile", ile!(0.999, self.p999))?;
        tup.serialize_entry("99.99%ile", ile!(0.9999, self.p9999))?;
        tup.end()
    }
}

/// See `hdr_histogram.rs` in `metered-rs`for explanation.
struct MetricAlias<T: Serialize>(&'static str, T);
impl<T: Serialize> Serialize for MetricAlias<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct(self.0, &self.1)
    }
}

impl MeterControlSurfaceMiddleware {
    pub fn new() -> MeterControlSurfaceMiddleware {
        Default::default()
    }

    pub fn metrics(&self) -> &ControlSurfaceMiddlewareMetrics {
        &self.metrics
    }

    pub fn measure(f: impl FnOnce()) -> u64 {
        let now = StdInstantMicros::now();
        f();
        now.elapsed_time()
    }

    pub fn record_run(&self, elapsed: u64) {
        self.metrics.run.record(elapsed);
    }

    pub fn record_event(&self, event: ControlSurfaceEvent, elapsed: u64) {
        use ControlSurfaceEvent::*;
        let response_time = match event {
            CloseNoReset => &self.metrics.close_no_reset,
            SetTrackListChange => &self.metrics.set_track_list_change,
            SetSurfaceVolume(_) => &self.metrics.set_surface_volume,
            SetSurfacePan(_) => &self.metrics.set_surface_pan,
            SetSurfaceMute(_) => &self.metrics.set_surface_mute,
            SetSurfaceSelected(_) => &self.metrics.set_surface_selected,
            SetSurfaceSolo(_) => &self.metrics.set_surface_solo,
            SetSurfaceRecArm(_) => &self.metrics.set_surface_rec_arm,
            SetPlayState(_) => &self.metrics.set_play_state,
            SetRepeatState(_) => &self.metrics.set_repeat_state,
            SetTrackTitle(_) => &self.metrics.set_track_title,
            SetAutoMode(_) => &self.metrics.set_auto_mode,
            ResetCachedVolPanStates => &self.metrics.reset_cached_vol_pan_states,
            OnTrackSelection(_) => &self.metrics.on_track_selection,
            ExtSetInputMonitor(_) => &self.metrics.ext_set_input_monitor,
            ExtSetFxParam(_) => &self.metrics.ext_set_fx_param,
            ExtSetFxParamRecFx(_) => &self.metrics.ext_set_fx_param_rec_fx,
            ExtSetFxEnabled(_) => &self.metrics.ext_set_fx_enabled,
            ExtSetSendVolume(_) => &self.metrics.ext_set_send_volume,
            ExtSetSendPan(_) => &self.metrics.ext_set_send_pan,
            ExtSetFocusedFx(_) => &self.metrics.ext_set_focused_fx,
            ExtSetLastTouchedFx(_) => &self.metrics.ext_set_last_touched_fx,
            ExtSetFxOpen(_) => &self.metrics.ext_set_fx_open,
            ExtSetFxChange(_) => &self.metrics.ext_set_fx_change,
            ExtSetBpmAndPlayRate(_) => &self.metrics.ext_set_bpm_and_play_rate,
            ExtTrackFxPresetChanged(_) => &self.metrics.ext_track_fx_preset_changed,
        };
        response_time.record(elapsed);
    }
}
