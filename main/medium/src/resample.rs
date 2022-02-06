use crate::ReaperResample;
use reaper_low::raw;
use ref_cast::RefCast;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// Case 3: Internals exposed: no | vtable: yes
// ===========================================

/// Owned REAPER resample instance.
///
/// This one automatically destroys the associated C++ `REAPER_Resample_Interface` when dropped.
#[derive(Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedReaperResample(ReaperResample);

/// Borrowed (reference-only) REAPER resample instance.
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedReaperResample(raw::REAPER_Resample_Interface);

impl OwnedReaperResample {
    /// Takes ownership of the given resample instance.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given instance is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: ReaperResample) -> Self {
        Self(raw)
    }
}

impl Drop for OwnedReaperResample {
    fn drop(&mut self) {
        unsafe {
            reaper_low::delete_cpp_reaper_resample_interface(self.0);
        }
    }
}

impl AsRef<BorrowedReaperResample> for OwnedReaperResample {
    fn as_ref(&self) -> &BorrowedReaperResample {
        BorrowedReaperResample::from_raw(unsafe { self.0.as_ref() })
    }
}

impl AsMut<BorrowedReaperResample> for OwnedReaperResample {
    fn as_mut(&mut self) -> &mut BorrowedReaperResample {
        BorrowedReaperResample::from_raw_mut(unsafe { self.0.as_mut() })
    }
}

impl Deref for OwnedReaperResample {
    type Target = BorrowedReaperResample;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for OwnedReaperResample {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl BorrowedReaperResample {
    /// Creates a medium-level representation from the given low-level reference.
    pub fn from_raw(raw: &raw::REAPER_Resample_Interface) -> &Self {
        Self::ref_cast(raw)
    }

    /// Creates a mutable medium-level representation from the given low-level reference.
    pub fn from_raw_mut(raw: &mut raw::REAPER_Resample_Interface) -> &mut Self {
        Self::ref_cast_mut(raw)
    }

    /// Returns the pointer to this resample instance.
    pub fn as_ptr(&self) -> ReaperResample {
        NonNull::from(self.as_ref())
    }
}

impl AsRef<raw::REAPER_Resample_Interface> for BorrowedReaperResample {
    fn as_ref(&self) -> &raw::REAPER_Resample_Interface {
        &self.0
    }
}

impl AsMut<raw::REAPER_Resample_Interface> for BorrowedReaperResample {
    fn as_mut(&mut self) -> &mut raw::REAPER_Resample_Interface {
        &mut self.0
    }
}
