/// This is like c_str_macro but directly creating `ReaperStr`.
macro_rules! reaper_str {
    ($lit:expr) => {
        $crate::ReaperStr::from_ptr(concat!($lit, "\0").as_ptr() as *const std::os::raw::c_char)
    };
}
