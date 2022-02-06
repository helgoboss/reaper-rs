use crate::PcmSink;
use reaper_low::raw;
use ref_cast::RefCast;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// Case 3: Internals exposed: no | vtable: yes
// ===========================================

/// Owned PCM sink.
///
/// This one automatically destroys the associated C++ `PCM_sink` when dropped.
#[derive(Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedPcmSink(PcmSink);

/// Borrowed (reference-only) PCM sink.
#[derive(PartialEq, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedPcmSink(raw::PCM_sink);

impl OwnedPcmSink {
    /// Takes ownership of the given PCM sink.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given sink is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: PcmSink) -> Self {
        Self(raw)
    }
}

impl Drop for OwnedPcmSink {
    fn drop(&mut self) {
        unsafe {
            reaper_low::delete_cpp_pcm_sink(self.0);
        }
    }
}

impl AsRef<BorrowedPcmSink> for OwnedPcmSink {
    fn as_ref(&self) -> &BorrowedPcmSink {
        BorrowedPcmSink::from_raw(unsafe { self.0.as_ref() })
    }
}

impl AsMut<BorrowedPcmSink> for OwnedPcmSink {
    fn as_mut(&mut self) -> &mut BorrowedPcmSink {
        BorrowedPcmSink::from_raw_mut(unsafe { self.0.as_mut() })
    }
}

impl Deref for OwnedPcmSink {
    type Target = BorrowedPcmSink;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for OwnedPcmSink {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl BorrowedPcmSink {
    /// Creates a medium-level representation from the given low-level reference.
    pub fn from_raw(raw: &raw::PCM_sink) -> &Self {
        Self::ref_cast(raw)
    }

    /// Creates a mutable medium-level representation from the given low-level reference.
    pub fn from_raw_mut(raw: &mut raw::PCM_sink) -> &mut Self {
        Self::ref_cast_mut(raw)
    }

    /// Returns the pointer to this sink.
    pub fn as_ptr(&self) -> PcmSink {
        NonNull::from(self.as_ref())
    }
}

impl AsRef<raw::PCM_sink> for BorrowedPcmSink {
    fn as_ref(&self) -> &raw::PCM_sink {
        &self.0
    }
}

impl AsMut<raw::PCM_sink> for BorrowedPcmSink {
    fn as_mut(&mut self) -> &mut raw::PCM_sink {
        &mut self.0
    }
}
