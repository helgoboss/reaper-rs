/// This creates a static `ReaperStr` string literal embedded in the binary.
///
/// If you pass this to REAPER functions, no string conversion has to be done at runtime.
///
/// # Example
///
/// ```
/// use reaper_medium::{ReaperStr, reaper_str};
///
/// let text: &'static ReaperStr = reaper_str!("Hello REAPER!");
/// ```
#[macro_export]
macro_rules! reaper_str {
    ($lit:expr) => {{
        #[allow(unused_unsafe)]
        let result = unsafe {
            $crate::ReaperStr::from_ptr(concat!($lit, "\0").as_ptr() as *const std::os::raw::c_char)
        };
        result
    }};
}
