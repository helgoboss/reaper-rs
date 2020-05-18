// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Reaper> = None;
static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();

#[derive(Clone, Debug, Default)]
pub struct Reaper {
    medium: reaper_medium::ReaperFunctions,
}

impl Reaper {
    pub(crate) fn new(medium: reaper_medium::ReaperFunctions) -> Reaper {
        Reaper { medium }
    }

    pub(crate) fn make_available_globally(reaper: Reaper) {
        unsafe {
            INIT_INSTANCE.call_once(|| INSTANCE = Some(reaper));
        }
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Reaper {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("call `make_available_globally()` before using `get()`")
        }
    }

    /// Gives access to the medium-level Reaper instance.
    pub fn medium(&self) -> &reaper_medium::ReaperFunctions {
        &self.medium
    }
}
