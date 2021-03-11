use crate::ControlSurfaceEvent;
use metered::hdr_histogram::HdrHistogram;
use metered::metric::Histogram;
use metered::time_source::{Instant, StdInstantMicros};
use metered::ResponseTime;
use serde::Serialize;
use slog::Logger;
use std::cell::RefCell;
use std::fmt;

type CustomResponseTime = ResponseTime<RefCell<HdrHistogram>, StdInstantMicros>;

#[derive(Debug)]
pub struct MeterMiddleware {
    logger: Logger,
    metrics: MeterMiddlewareMetrics,
    descriptors: ControlSurfaceResponseTimeDescriptors,
}

#[derive(Debug, Default, Serialize)]
pub struct MeterMiddlewareMetrics {
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
    ext_set_recv_volume: CustomResponseTime,
    ext_set_recv_pan: CustomResponseTime,
    ext_set_pan_ex: CustomResponseTime,
    ext_set_focused_fx: CustomResponseTime,
    ext_set_last_touched_fx: CustomResponseTime,
    ext_set_fx_open: CustomResponseTime,
    ext_set_fx_change: CustomResponseTime,
    ext_set_bpm_and_play_rate: CustomResponseTime,
    ext_track_fx_preset_changed: CustomResponseTime,
    ext_reset: CustomResponseTime,
    ext_set_project_marker_change: CustomResponseTime,
}

impl MeterMiddlewareMetrics {
    pub fn response_time_descriptors() -> ControlSurfaceResponseTimeDescriptors {
        [
            MetricDescriptor::new("run", |m| &m.run, is_critical_default),
            MetricDescriptor::new("close_no_reset", |m| &m.close_no_reset, is_critical_default),
            MetricDescriptor::new(
                "set_track_list_change",
                |m| &m.set_track_list_change,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_volume",
                |m| &m.set_surface_volume,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_pan",
                |m| &m.set_surface_pan,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_mute",
                |m| &m.set_surface_mute,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_selected",
                |m| &m.set_surface_selected,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_solo",
                |m| &m.set_surface_solo,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_surface_rec_arm",
                |m| &m.set_surface_rec_arm,
                is_critical_default,
            ),
            MetricDescriptor::new("set_play_state", |m| &m.set_play_state, is_critical_default),
            MetricDescriptor::new(
                "set_repeat_state",
                |m| &m.set_repeat_state,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "set_track_title",
                |m| &m.set_track_title,
                is_critical_default,
            ),
            MetricDescriptor::new("set_auto_mode", |m| &m.set_auto_mode, is_critical_default),
            MetricDescriptor::new(
                "reset_cached_vol_pan_states",
                |m| &m.reset_cached_vol_pan_states,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "on_track_selection",
                |m| &m.on_track_selection,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_input_monitor",
                |m| &m.ext_set_input_monitor,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_fx_param",
                |m| &m.ext_set_fx_param,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_fx_param_rec_fx",
                |m| &m.ext_set_fx_param_rec_fx,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_fx_enabled",
                |m| &m.ext_set_fx_enabled,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_send_volume",
                |m| &m.ext_set_send_volume,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_send_pan",
                |m| &m.ext_set_send_pan,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_recv_volume",
                |m| &m.ext_set_recv_volume,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_recv_pan",
                |m| &m.ext_set_recv_pan,
                is_critical_default,
            ),
            MetricDescriptor::new("ext_set_pan_ex", |m| &m.ext_set_pan_ex, is_critical_default),
            MetricDescriptor::new(
                "ext_set_focused_fx",
                |m| &m.ext_set_focused_fx,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_last_touched_fx",
                |m| &m.ext_set_last_touched_fx,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_fx_open",
                |m| &m.ext_set_fx_open,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_fx_change",
                |m| &m.ext_set_fx_change,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_set_bpm_and_play_rate",
                |m| &m.ext_set_bpm_and_play_rate,
                is_critical_default,
            ),
            MetricDescriptor::new(
                "ext_track_fx_preset_changed",
                |m| &m.ext_track_fx_preset_changed,
                is_critical_default,
            ),
            MetricDescriptor::new("ext_reset", |m| &m.ext_reset, is_critical_default),
            MetricDescriptor::new(
                "ext_set_project_marker_change",
                |m| &m.ext_set_project_marker_change,
                is_critical_default,
            ),
        ]
    }
}

impl MeterMiddleware {
    pub fn new(logger: Logger) -> MeterMiddleware {
        MeterMiddleware {
            logger,
            metrics: Default::default(),
            descriptors: MeterMiddlewareMetrics::response_time_descriptors(),
        }
    }

    pub fn metrics(&self) -> &MeterMiddlewareMetrics {
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

    pub fn record_event(&self, event: ControlSurfaceEvent, elapsed: u64) -> bool {
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
            ExtSetRecvVolume(_) => &self.metrics.ext_set_recv_volume,
            ExtSetRecvPan(_) => &self.metrics.ext_set_recv_pan,
            ExtSetFocusedFx(_) => &self.metrics.ext_set_focused_fx,
            ExtSetLastTouchedFx(_) => &self.metrics.ext_set_last_touched_fx,
            ExtSetFxOpen(_) => &self.metrics.ext_set_fx_open,
            ExtSetFxChange(_) => &self.metrics.ext_set_fx_change,
            ExtSetBpmAndPlayRate(_) => &self.metrics.ext_set_bpm_and_play_rate,
            ExtTrackFxPresetChanged(_) => &self.metrics.ext_track_fx_preset_changed,
            ExtSetPanExt(_) => &self.metrics.ext_set_pan_ex,
            ExtReset(_) => &self.metrics.ext_reset,
            ExtSetProjectMarkerChange(_) => &self.metrics.ext_set_project_marker_change,
        };
        response_time.record(elapsed);
        true
    }

    pub fn warn_about_critical_metrics(&self) {
        for desc in &self.descriptors {
            self.warn_if_critical(desc);
        }
    }

    pub fn log_metrics(&self) {
        slog::info!(self.logger, "{}", format_pretty(&self.metrics));
    }

    fn warn_if_critical(&self, descriptor: &ResponseTimeDescriptor<MeterMiddlewareMetrics>) {
        let response_time = descriptor.get_metric(&self.metrics);
        if descriptor.is_critical(response_time) {
            slog::warn!(
                self.logger,
                "Encountered slow control surface execution";
                "method" => descriptor.name(),
                "response_time" => format_pretty(response_time)
            );
        }
    }
}

fn format_pretty(value: &impl serde::Serialize) -> String {
    serde_yaml::to_string(value).unwrap()
}

pub type ResponseTimeDescriptor<R> = MetricDescriptor<R, CustomResponseTime>;

/// Type parameters
///
/// * `R` - Metrics registry
/// * `M` - Metric
pub struct MetricDescriptor<R, M> {
    name: &'static str,
    get_metric: fn(&R) -> &M,
    is_critical: fn(&M) -> bool,
}

impl<R, M> fmt::Debug for MetricDescriptor<R, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetricDescriptor")
            .field("name", &self.name)
            .finish()
    }
}

impl<R, M> MetricDescriptor<R, M> {
    pub fn new(name: &'static str, get_metric: fn(&R) -> &M, is_critical: fn(&M) -> bool) -> Self {
        Self {
            name,
            get_metric,
            is_critical,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get_metric<'a>(&self, registry: &'a R) -> &'a M {
        (self.get_metric)(registry)
    }

    pub fn is_critical(&self, metric: &M) -> bool {
        (self.is_critical)(metric)
    }
}

type ControlSurfaceResponseTimeDescriptors = [ResponseTimeDescriptor<MeterMiddlewareMetrics>; 32];

fn is_critical_default(response_time: &CustomResponseTime) -> bool {
    response_time.borrow().max() > 10000
}
