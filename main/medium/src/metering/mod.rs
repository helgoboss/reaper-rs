use core::fmt;
use metered::hdr_histogram::{AtomicHdrHistogram, HdrHistogram};
use metered::time_source::StdInstantMicros;
use metered::ResponseTime;
use serde::export::Formatter;
use std::cell::RefCell;

pub type ResponseTimeSingleThreaded = ResponseTime<RefCell<HdrHistogram>, StdInstantMicros>;
pub type ResponseTimeMultiThreaded = ResponseTime<AtomicHdrHistogram, StdInstantMicros>;

pub type ResponseTimeDescriptor<R> = MetricDescriptor<R, ResponseTimeSingleThreaded>;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
