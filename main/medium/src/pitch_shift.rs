use crate::ReaperPitchShift;
use reaper_low::raw;
use ref_cast::RefCast;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// Case 3: Internals exposed: no | vtable: yes
// ===========================================

/// Owned REAPER pitch shift instance.
///
/// This one automatically destroys the associated C++ `IReaperPitchShift` when dropped.
#[derive(Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedReaperPitchShift(ReaperPitchShift);

unsafe impl Send for OwnedReaperPitchShift {}

/// Borrowed (reference-only) REAPER pitch shift instance.
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedReaperPitchShift(raw::IReaperPitchShift);

impl OwnedReaperPitchShift {
    /// Takes ownership of the given pitch shift instance.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given instance is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: ReaperPitchShift) -> Self {
        Self(raw)
    }
}

impl Drop for OwnedReaperPitchShift {
    fn drop(&mut self) {
        unsafe {
            reaper_low::delete_cpp_reaper_pitch_shift(self.0);
        }
    }
}

impl AsRef<BorrowedReaperPitchShift> for OwnedReaperPitchShift {
    fn as_ref(&self) -> &BorrowedReaperPitchShift {
        BorrowedReaperPitchShift::from_raw(unsafe { self.0.as_ref() })
    }
}

impl AsMut<BorrowedReaperPitchShift> for OwnedReaperPitchShift {
    fn as_mut(&mut self) -> &mut BorrowedReaperPitchShift {
        BorrowedReaperPitchShift::from_raw_mut(unsafe { self.0.as_mut() })
    }
}

impl Deref for OwnedReaperPitchShift {
    type Target = BorrowedReaperPitchShift;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for OwnedReaperPitchShift {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl BorrowedReaperPitchShift {
    /// Creates a medium-level representation from the given low-level reference.
    pub fn from_raw(raw: &raw::IReaperPitchShift) -> &Self {
        Self::ref_cast(raw)
    }

    /// Creates a mutable medium-level representation from the given low-level reference.
    pub fn from_raw_mut(raw: &mut raw::IReaperPitchShift) -> &mut Self {
        Self::ref_cast_mut(raw)
    }

    /// Returns the pointer to this pitch shift instance.
    pub fn as_ptr(&self) -> ReaperPitchShift {
        NonNull::from(self.as_ref())
    }
}

impl AsRef<raw::IReaperPitchShift> for BorrowedReaperPitchShift {
    fn as_ref(&self) -> &raw::IReaperPitchShift {
        &self.0
    }
}

impl AsMut<raw::IReaperPitchShift> for BorrowedReaperPitchShift {
    fn as_mut(&mut self) -> &mut raw::IReaperPitchShift {
        &mut self.0
    }
}
