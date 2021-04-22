use derive_more::*;
use std::cell::UnsafeCell;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Type alias for the platform-specific REAPER mutex primitive.
#[cfg(windows)]
pub(crate) type ReaperMutexPrimitive = winapi::um::minwinbase::CRITICAL_SECTION;

/// Type alias for the platform-specific REAPER mutex primitive.
#[cfg(unix)]
pub(crate) type ReaperMutexPrimitive = libc::pthread_mutex_t;

/// Initializes the given mutex primitive.
pub(crate) fn initialize_mutex_primitive(primitive: &mut ReaperMutexPrimitive) {
    #[cfg(windows)]
    unsafe {
        winapi::um::synchapi::InitializeCriticalSection(primitive as *mut _);
    }
    #[cfg(unix)]
    unsafe {
        libc::pthread_mutex_init(primitive as *mut _ as _, std::ptr::null());
    }
}

/// Destroys the given mutex primitive.
pub(crate) fn destroy_mutex_primitive(primitive: &mut ReaperMutexPrimitive) {
    #[cfg(windows)]
    unsafe {
        winapi::um::synchapi::DeleteCriticalSection(primitive as *mut _);
    }
    #[cfg(unix)]
    unsafe {
        libc::pthread_mutex_destroy(primitive as *mut _ as _);
    }
}

/// Mutex that works on native critical sections / mutexes exposed by the REAPER API.
pub struct ReaperMutex<T: AsRef<ReaperMutexPrimitive>> {
    pub(crate) data: UnsafeCell<T>,
}

impl<T: AsRef<ReaperMutexPrimitive>> fmt::Debug for ReaperMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // In future we could use try-lock in the same fashion that Rust's std Mutex does it.
        f.debug_struct("ReaperMutex").finish()
    }
}

impl<T: AsRef<ReaperMutexPrimitive>> ReaperMutex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            data: UnsafeCell::new(inner),
        }
    }

    /// Acquires read/write access to the underlying data.
    pub fn lock(&self) -> Result<ReaperMutexGuard<T>, ReaperLockError> {
        ReaperMutexGuard::new(&self)
    }

    fn primitive_ptr(&self) -> *mut ReaperMutexPrimitive {
        let data = unsafe { &*self.data.get() };
        data.as_ref() as *const _ as *mut _
    }
}

pub struct ReaperMutexGuard<'a, T: AsRef<ReaperMutexPrimitive>> {
    lock: &'a ReaperMutex<T>,
}

impl<'a, T: AsRef<ReaperMutexPrimitive>> ReaperMutexGuard<'a, T> {
    #[allow(clippy::unnecessary_wraps)]
    fn new(mutex: &'a ReaperMutex<T>) -> Result<Self, ReaperLockError> {
        #[cfg(windows)]
        unsafe {
            winapi::um::synchapi::EnterCriticalSection(mutex.primitive_ptr());
        }
        #[cfg(unix)]
        unsafe {
            let result = libc::pthread_mutex_lock(mutex.primitive_ptr() as _);
            if result != 0 {
                return Err(ReaperLockError(()));
            }
        }
        let guard = Self { lock: mutex };
        Ok(guard)
    }
}
impl<'a, T: AsRef<ReaperMutexPrimitive>> Drop for ReaperMutexGuard<'a, T> {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            winapi::um::synchapi::LeaveCriticalSection(self.lock.primitive_ptr());
        }
        #[cfg(unix)]
        unsafe {
            libc::pthread_mutex_unlock(self.lock.primitive_ptr() as _);
        }
    }
}

impl<T: AsRef<ReaperMutexPrimitive>> Deref for ReaperMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: AsRef<ReaperMutexPrimitive>> DerefMut for ReaperMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

/// An error which can occur when trying to lock a REAPER mutex.
#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(fmt = "couldn't acquire lock")]
pub struct ReaperLockError(());

impl std::error::Error for ReaperLockError {}
