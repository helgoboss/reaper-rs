use std::borrow::Cow;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// This type is used for all medium-level API function parameters which are strings. The C++
/// REAPER SDK generally expects UTF-8 encoded C-Strings (`*const c_char`). It wouldn't be
/// convenient if we just keep it that way in Rust.
///
/// At the very least we should provide the possibility to pass `&CStr`. That gives us type safety:
/// It's a reference instead of a pointer so we can assume it's not stale. Also, the `CStr` type
/// gives some guarantees, for example that there are no intermediate zero bytes (which would make
/// the string end abruptly in C world). Because `&CStr` is the closest thing to a `*const c_char`
/// (infact it's the same + some type safety guarantees), `ReaperStringArg` is just a wrapper around
/// it. That answers the medium-level API's claim to be still as close to the original REAPER SDK as
/// possible.
///
/// Well, actually it's a wrapper around a `Cow<CStr>` because it also needs to be able to hold
/// conversion results. For improved convenience, `ReaperStringArg` offers conversions from regular
/// Rust string types. Because medium-level API functions take string parameters as `impl
/// Into<ReaperStringArg>`, they *just work* with regular Rust strings. This conversion is not
/// entirely without cost because it needs to check for intermediate zero bytes and append a zero
/// byte (which demands a copy if a borrowed string is passed)! Therefore, if you want to be sure to
/// not waste any performance, just pass a `&CStr`, then there's no extra cost involved. In many
/// scenarios this is probably over optimization, but the point is, you *can* go the zero-cost way,
/// if you want. Fortunately, the string encoding itself doesn't need to be converted because REAPER
/// expects UTF-8 encoding as well.
///
/// In the *reaper-rs*  code base you will find many examples that pass `c_str!("...")` to string
/// parameters. This macro from the [c_str_macro crate](https://crates.io/crates/c_str_macro)
/// creates static (UTF-8 encoded) `&CStr` literals, just as `"..."` creates `&str` literals.
/// Because those literals are embedded in the plug-in code, no heap-space allocation or conversion
/// is necessary at all. If you want, you can do the same with literals.
pub struct ReaperStringArg<'a>(Cow<'a, CStr>);

impl<'a> ReaperStringArg<'a> {
    /// Returns a raw pointer to the string. Used by code in this crate only.
    pub(super) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }

    /// Consumes this string and spits out the contained cow. Used by code in this crate only.
    pub(super) fn into_inner(self) -> Cow<'a, CStr> {
        self.0
    }
}

// This is the most important conversion because it's the ideal case (zero-cost). For now we don't
// offer a conversion from `CString` (owned) because it could confuse consumers. They might start to
// think that string arguments are always consumed, which is not the case. If there's much demand,
// we can still add that later.
impl<'a> From<&'a CStr> for ReaperStringArg<'a> {
    fn from(s: &'a CStr) -> Self {
        ReaperStringArg(s.into())
    }
}

// This is the second most important conversion because we want consumers to be able to just pass a
// normal string literal.
impl<'a> From<&'a str> for ReaperStringArg<'a> {
    fn from(s: &'a str) -> Self {
        // Requires copying
        ReaperStringArg(CString::new(s).unwrap().into())
    }
}

// This conversion might appear somewhat unusual because it takes something *owned*. But that has a
// good reason: If there's a `String` which the consumer is okay to give away (move), this is
// good for performance because no copy needs to be made in order to convert this into a C string.
// By introducing this conversion, we want to encourage this scenario.
impl<'a> From<String> for ReaperStringArg<'a> {
    fn from(s: String) -> Self {
        // Doesn't require copying because we own the string now
        ReaperStringArg(CString::new(s).unwrap().into())
    }
}
