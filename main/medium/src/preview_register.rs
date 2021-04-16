use crate::mutex::ReaperMutex;
use crate::{
    destroy_mutex_primitive, initialize_mutex_primitive, ReaperLockError, ReaperMutexGuard,
    ReaperMutexPrimitive, ReaperVolumeValue,
};
use reaper_low::raw;
use std::fmt;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;

/// An owned preview register.
///
/// It owns the mutex or critical section (and manages its lifetime) but it doesn't own the PCM
/// source and track.
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
//
// A borrowed version is not necessary for now because as I see it, preview registers are *always*
// created by the consumer and never returned by REAPER itself. The only use case case would be
// interoperation with another extension but that would probably look differently anyway. If one
// day we have the need, we can introduce a borrowed version, move most methods to it and at a
// Deref implementation from owned to borrowed.
// TODO-high Clone should probably implemented manually (mutex initialization!)
#[derive(Clone)]
pub struct OwnedPreviewRegister(raw::preview_register_t);

impl fmt::Debug for OwnedPreviewRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedPreviewRegister").finish()
    }
}

impl OwnedPreviewRegister {
    /// Creates a preview register.
    ///
    /// Also takes care of initializing the mutex.
    pub fn new() -> OwnedPreviewRegister {
        Default::default()
    }

    pub fn src(&self) -> Option<NonNull<raw::PCM_source>> {
        NonNull::new(self.0.src)
    }

    pub fn set_src(&mut self, src: Option<NonNull<raw::PCM_source>>) {
        self.0.src = src.map(NonNull::as_ptr).unwrap_or(null_mut());
    }

    pub fn volume(&self) -> ReaperVolumeValue {
        ReaperVolumeValue::new(self.0.volume)
    }

    pub fn set_volume(&mut self, volume: ReaperVolumeValue) {
        self.0.volume = volume.get()
    }

    /// TODO-high Position in seconds?
    pub fn cur_pos(&self) -> f64 {
        self.0.curpos
    }

    /// TODO-high Position in seconds?
    pub fn set_cur_pos(&mut self, pos: f64) {
        self.0.curpos = pos;
    }
}

impl Default for OwnedPreviewRegister {
    fn default() -> Self {
        let mut inner = raw::preview_register_t {
            #[cfg(windows)]
            cs: Default::default(),
            #[cfg(unix)]
            mutex: Default::default(),
            ..Default::default()
        };
        #[cfg(windows)]
        initialize_mutex_primitive(&mut inner.cs);
        #[cfg(unix)]
        initialize_mutex_primitive(&mut inner.mutex);
        Self(inner)
    }
}

impl Drop for OwnedPreviewRegister {
    fn drop(&mut self) {
        #[cfg(windows)]
        destroy_mutex_primitive(&mut self.0.cs);
        #[cfg(unix)]
        destroy_mutex_primitive(&mut self.0.mutex);
    }
}

impl AsRef<raw::preview_register_t> for OwnedPreviewRegister {
    fn as_ref(&self) -> &raw::preview_register_t {
        &self.0
    }
}

// We want to have access to the raw register pointer even without locking the mutex. Necessary
// because of our `SharedKeeper` type bounds.
impl AsRef<raw::preview_register_t> for ReaperMutex<OwnedPreviewRegister> {
    fn as_ref(&self) -> &raw::preview_register_t {
        let data = unsafe { &*self.data.get() };
        data.as_ref()
    }
}

impl AsRef<ReaperMutexPrimitive> for OwnedPreviewRegister {
    fn as_ref(&self) -> &ReaperMutexPrimitive {
        #[cfg(windows)]
        {
            &self.0.cs
        }
        #[cfg(unix)]
        {
            &self.0.mutex
        }
    }
}

/// Preview register that's in control of REAPER because it's being played.
pub struct PlayingPreviewRegister {
    pub(crate) register: Rc<ReaperMutex<OwnedPreviewRegister>>,
    pub(crate) handle: NonNull<raw::preview_register_t>,
}

impl PlayingPreviewRegister {
    /// TODO-high Are there preview register fields that shouldn't be set while playing? In that
    ///  case it would be better to encapsulate it using a delegate. If not, we could also expose
    ///  the guard directly without using continuation passing style.
    pub fn lock<R>(
        &self,
        f: impl FnOnce(Result<&mut OwnedPreviewRegister, ReaperLockError>) -> R,
    ) -> R {
        match self.register.lock() {
            Ok(mut r) => f(Ok(&mut *r)),
            Err(e) => f(Err(e)),
        }
    }

    pub fn handle(&self) -> NonNull<raw::preview_register_t> {
        self.handle
    }
}
