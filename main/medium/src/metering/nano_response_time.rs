use crate::metering::SingleThreadedHdrHistogram;
use metered::clear::Clear;
use metered::metric::{Advice, Histogram, Metric, OnResult};
use metered::Enter;
use serde::{Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;

/// Like the original metered-rs ResponseTime metric but on nano-second granularity and not
/// thread-safe.
pub struct NanoResponseTime(SingleThreadedHdrHistogram);

impl Default for NanoResponseTime {
    fn default() -> Self {
        // A HdrHistogram measuring latencies from 1ms to 5minutes
        // All recordings will be saturating, that is, a value higher than 60 seconds
        // will be replace by 60 seconds...
        NanoResponseTime(SingleThreadedHdrHistogram::with_bound(60 * 1_000_000_000))
    }
}

impl<R> Metric<R> for NanoResponseTime {}

impl Enter for NanoResponseTime {
    type E = std::time::Instant;

    fn enter(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

impl<R> OnResult<R> for NanoResponseTime {
    fn on_result(&self, enter: std::time::Instant, _: &R) -> Advice {
        let elapsed = enter.elapsed();
        self.0.record(elapsed.as_nanos() as u64);
        Advice::Return
    }
}

impl Clear for NanoResponseTime {
    fn clear(&self) {
        self.0.clear();
    }
}

impl Serialize for NanoResponseTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

impl Debug for NanoResponseTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}
