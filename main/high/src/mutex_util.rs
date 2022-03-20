use std::sync::{LockResult, Mutex, MutexGuard};

/// Locks the given mutex even it has been poisoned.
///
/// That's mostly okay and even desired in our case because we catch panics anyway at a fault
/// barrier and log them. Logging the poisoning panic would just hide the original panic and
/// makes things stop working.
pub fn lock_ignoring_poisoning<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    match mutex.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    }
}
