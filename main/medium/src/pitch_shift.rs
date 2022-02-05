use reaper_low::raw;
use reaper_low::raw::IReaperPitchShift;
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
pub struct OwnedReaperPitchShift(NonNull<raw::IReaperPitchShift>);

/// Borrowed (reference-only) REAPER pitch shift instance.
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct ReaperPitchShift(raw::IReaperPitchShift);

impl OwnedReaperPitchShift {
    /// Takes ownership of the given pitch shift instance.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given instance is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: NonNull<raw::IReaperPitchShift>) -> Self {
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

impl AsRef<ReaperPitchShift> for OwnedReaperPitchShift {
    fn as_ref(&self) -> &ReaperPitchShift {
        ReaperPitchShift::ref_cast(unsafe { self.0.as_ref() })
    }
}

impl AsMut<ReaperPitchShift> for OwnedReaperPitchShift {
    fn as_mut(&mut self) -> &mut ReaperPitchShift {
        ReaperPitchShift::ref_cast_mut(unsafe { self.0.as_mut() })
    }
}

impl Deref for OwnedReaperPitchShift {
    type Target = ReaperPitchShift;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for OwnedReaperPitchShift {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl AsRef<raw::IReaperPitchShift> for ReaperPitchShift {
    fn as_ref(&self) -> &IReaperPitchShift {
        &self.0
    }
}

impl AsMut<raw::IReaperPitchShift> for ReaperPitchShift {
    fn as_mut(&mut self) -> &mut IReaperPitchShift {
        &mut self.0
    }
}
