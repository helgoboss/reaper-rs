use reaper_medium::metering::ResponseTimeDescriptor;
use reaper_medium::{ControlSurfaceMetrics, ControlSurfaceResponseTimeDescriptors};
use slog::Logger;
use std::time::Duration;

#[derive(Debug)]
pub struct ControlSurfacePerformanceMonitor {
    logger: Logger,
    check_interval: Duration,
    counter: u64,
    descriptors: ControlSurfaceResponseTimeDescriptors,
}

impl ControlSurfacePerformanceMonitor {
    pub fn new(logger: Logger, check_interval: Duration) -> ControlSurfacePerformanceMonitor {
        ControlSurfacePerformanceMonitor {
            logger,
            check_interval,
            counter: 0,
            descriptors: ControlSurfaceMetrics::response_time_descriptors(),
        }
    }

    pub fn handle_metrics(&mut self, metrics: &reaper_medium::ControlSurfaceMetrics) {
        // We know it's called roughly 30 times a second.
        if self.counter == 30 * self.check_interval.as_secs() {
            for desc in &self.descriptors {
                self.warn_if_critical(metrics, desc);
            }
            self.counter = 0;
        } else {
            self.counter += 1;
        }
    }

    pub fn log_metrics(&self, metrics: &ControlSurfaceMetrics) {
        slog::info!(self.logger, "{}", format_pretty(metrics));
    }

    fn warn_if_critical(
        &self,
        metrics: &ControlSurfaceMetrics,
        descriptor: &ResponseTimeDescriptor<ControlSurfaceMetrics>,
    ) {
        let response_time = descriptor.get_metric(metrics);
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
