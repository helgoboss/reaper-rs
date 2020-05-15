use metered::clear::Clear;
use metered::hdr_histogram::HdrHistogram;
use metered::metric::Histogram;
use serde::{Serialize, Serializer};
use std::cell::RefCell;

/// A single-threaded implementation of HdrHistogram
#[derive(Debug)]
pub struct SingleThreadedHdrHistogram {
    inner: RefCell<HdrHistogram>,
}

impl Histogram for SingleThreadedHdrHistogram {
    fn with_bound(max_bound: u64) -> Self {
        let histo = HdrHistogram::with_bound(max_bound);
        let inner = RefCell::new(histo);
        SingleThreadedHdrHistogram { inner }
    }

    fn record(&self, value: u64) {
        self.inner.borrow_mut().record(value);
    }
}

impl Clear for SingleThreadedHdrHistogram {
    fn clear(&self) {
        self.inner.borrow_mut().clear();
    }
}

impl Serialize for SingleThreadedHdrHistogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use std::ops::Deref;
        let inner = self.inner.borrow_mut();
        let inner = inner.deref();
        Serialize::serialize(inner, serializer)
    }
}
