use metered::hdr_histogram::{AtomicHdrHistogram, HdrHistogram};
use metered::time_source::StdInstantMicros;
use metered::ResponseTime;
use std::cell::RefCell;

pub type ResponseTimeSingleThreaded = ResponseTime<RefCell<HdrHistogram>, StdInstantMicros>;
pub type ResponseTimeMultiThreaded = ResponseTime<AtomicHdrHistogram, StdInstantMicros>;
