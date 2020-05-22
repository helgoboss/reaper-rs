use metered::clear::Clear;
use metered::hdr_histogram::AtomicHdrHistogram;
use metered::metric::{Advice, Histogram, Metric, OnResult};
use metered::Enter;
use serde::{Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;

/// Like the original metered-rs ResponseTime metric but on nano-second granularity.
pub struct NanoResponseTime<H: Histogram = AtomicHdrHistogram>(H);

impl<H: Histogram> Default for NanoResponseTime<H> {
    fn default() -> Self {
        // A HdrHistogram measuring latencies from 1ns to 1minute
        // All recordings will be saturating, that is, a value higher than 60 seconds
        // will be replaced by 60 seconds...
        NanoResponseTime(H::with_bound(60 * 1_000_000_000))
    }
}

impl<R, H: Histogram> Metric<R> for NanoResponseTime<H> {}

impl<H: Histogram> Enter for NanoResponseTime<H> {
    type E = std::time::Instant;

    fn enter(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

impl<H: Histogram, R> OnResult<R> for NanoResponseTime<H> {
    fn on_result(&self, enter: std::time::Instant, _: &R) -> Advice {
        let elapsed = enter.elapsed();
        self.0.record(elapsed.as_nanos() as u64);
        Advice::Return
    }
}

impl<H: Histogram> Clear for NanoResponseTime<H> {
    fn clear(&self) {
        self.0.clear();
    }
}

impl<H: Histogram> Serialize for NanoResponseTime<H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.0, serializer)
    }
}

impl<H: Histogram + Debug> Debug for NanoResponseTime<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}
