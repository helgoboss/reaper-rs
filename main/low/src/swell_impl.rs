use crate::{Swell, SwellFunctionPointers};

// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Swell> = None;
static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();

impl Swell {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    pub fn make_available_globally(functions: Swell) {
        unsafe {
            INIT_INSTANCE.call_once(|| INSTANCE = Some(functions));
        }
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Swell {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("call `make_available_globally()` before using `get()`")
        }
    }

    /// Gives access to the SWELL function pointers.
    pub fn pointers(&self) -> &SwellFunctionPointers {
        &self.pointers
    }
}
