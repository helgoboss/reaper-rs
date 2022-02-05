use reaper_low::raw;
use reaper_low::raw::REAPER_Resample_Interface;
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
pub struct OwnedReaperResample(NonNull<raw::REAPER_Resample_Interface>);

/// Borrowed (reference-only) REAPER resample instance.
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct ReaperResample(raw::REAPER_Resample_Interface);

impl OwnedReaperResample {
    /// Takes ownership of the given resample instance.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given instance is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn from_raw(raw: NonNull<raw::REAPER_Resample_Interface>) -> Self {
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

impl AsRef<ReaperResample> for OwnedReaperResample {
    fn as_ref(&self) -> &ReaperResample {
        ReaperResample::ref_cast(unsafe { self.0.as_ref() })
    }
}

impl AsMut<ReaperResample> for OwnedReaperResample {
    fn as_mut(&mut self) -> &mut ReaperResample {
        ReaperResample::ref_cast_mut(unsafe { self.0.as_mut() })
    }
}

impl Deref for OwnedReaperResample {
    type Target = ReaperResample;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for OwnedReaperResample {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl AsRef<raw::REAPER_Resample_Interface> for ReaperResample {
    fn as_ref(&self) -> &REAPER_Resample_Interface {
        &self.0
    }
}

impl AsMut<raw::REAPER_Resample_Interface> for ReaperResample {
    fn as_mut(&mut self) -> &mut REAPER_Resample_Interface {
        &mut self.0
    }
}
