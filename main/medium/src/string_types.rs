use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// A string parameter.
///
/// Medium-level API functions with string parameters accept all kinds of strings which can be
/// converted into this type, most notably `&CStr` and `&str`.
///
/// # Design
///
/// This is a wrapper around a `Cow<CStr>`.
///
/// ## Why C strings and not regular Rust strings?
///
/// We use a sort of C string because that perfectly accounts for the medium-level API's design goal
/// to be still as close to the original REAPER API as possible (while at the same time introducing
/// Rust's type safety). The C++ REAPER API generally expects C strings (`*const c_char`).
/// Fortunately UTF-8 encoded ones - which makes a character set conversion unnecessary.
///
/// ## Why `&CStr` and not `*const c_char`?
///
/// We don't use `*const c_char` directly because we want more type safety. We use `&CStr` instead
/// because in Rust that's the closest thing to a `*const c_char` (infact it's the same + some
/// additional guarantees). It's a reference instead of a pointer so we can assume it's neither
/// stale nor `null`. Also, the `&CStr` type gives us important guarantees, for example that there
/// are no intermediate zero bytes - which would make the string end abruptly in C world.
///
/// ## Why `Cow` and `ReaperStringArg`?
///
/// We don't use just a plain `&CStr` as parameter type because `&CStr` is not the regular string
/// type in Rust. It's much harder to create and use than `&str`. We want the API to be a pleasure
/// to use! That's the reason for adding `ReaperStringArg` and `Cow` to the mix. `Cow`is necessary
/// because we might need to own a possible conversion result (e.g. from `&str`). `ReaperStringArg`
/// is necessary to offer implicit conversions from regular Rust string types. Because medium-level
/// API functions take string parameters as `impl Into<ReaperStringArg>`, they *just work* with
/// regular Rust strings.
///
/// ## Performance considerations
///
/// A conversion from a regular Rust string is not entirely without cost because we need to check
/// for intermediate zero bytes and append a zero byte (which demands a copy if a borrowed string is
/// passed)! Therefore, if you want to be sure to not waste any performance and you can get cheap
/// access to a C string, just pass that one directly. Then there's no extra cost involved. In many
/// scenarios this is probably over optimization, but the point is, you *can* go the zero-cost way,
/// if you want.
///
/// In the *reaper-rs*  code base you will find many examples that pass `c_str!("...")` to string
/// parameters. This macro from the [c_str_macro crate](https://crates.io/crates/c_str_macro)
/// creates static (UTF-8 encoded) `&CStr` literals, just as `"..."` creates static `&str` literals.
/// Because those literals are embedded in the binary itself, no heap-space allocation or conversion
/// is necessary at all. If you want, you can do the same with your literals.
//
// This type doesn't need to derive common traits because the consumer never interacts with it
// directly.
pub struct ReaperStringArg<'a>(Cow<'a, CStr>);

impl<'a> ReaperStringArg<'a> {
    /// Returns a raw pointer to the string. Used by code in this crate only.
    pub(super) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }

    /// Consumes this value and spits out the contained cow. Used by code in this crate only.
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
        ReaperStringArg(
            CString::new(s)
                .expect("Rust string too exotic for REAPER")
                .into(),
        )
    }
}

// This conversion might appear somewhat unusual because it takes something *owned*. But that has a
// good reason: If there's a `String` which the consumer is okay to give away (move), this is
// good for performance because no copy needs to be made in order to convert this into a C string.
// By introducing this conversion, we want to encourage this scenario.
impl<'a> From<String> for ReaperStringArg<'a> {
    fn from(s: String) -> Self {
        // Doesn't require copying because we own the string now
        ReaperStringArg(
            CString::new(s)
                .expect("Rust string too exotic for REAPER")
                .into(),
        )
    }
}

/// An owned string created by REAPER.
///
/// This type is used in return positions of _reaper-rs_ functions. It wraps a `CString` because
/// REAPER creates C strings. The benefit over just returning `CString` is that this type provides
/// convenience methods for converting to Rust strings directly. Whereas arbitrary `CString`s can
/// have all kinds of encodings, we know that REAPER uses UTF-8, so this type can be optimistic and
/// converts without returning a `Result`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ReaperString(CString);

impl ReaperString {
    pub(crate) fn new(inner: CString) -> ReaperString {
        ReaperString(inner)
    }

    /// Consumes this value and spits out the contained C string.
    pub fn into_inner(self) -> CString {
        self.0
    }

    /// Converts this to a Rust string slice.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not properly UTF-8 encoded.
    pub fn to_str(&self) -> &str {
        self.0
            .to_str()
            .expect("REAPER string should be UTF-8 encoded")
    }

    /// Consumes this and converts it to an owned Rust string.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not properly UTF-8 encoded.
    pub fn into_string(self) -> String {
        self.0
            .into_string()
            .expect("REAPER string should be UTF-8 encoded")
    }
}

/// A borrowed string owned by REAPER.
///
/// _reaper-rs_ functions pass this type to consumer-provided closures.
///
/// See [`ReaperString`] for further details.
///
/// [`ReaperString`]: struct.ReaperString.html
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ReaperStr<'a>(&'a CStr);

impl<'a> ReaperStr<'a> {
    pub(crate) fn new(inner: &'a CStr) -> ReaperStr<'a> {
        ReaperStr(inner)
    }

    /// Spits out the contained borrowed C string.
    pub fn into_inner(self) -> &'a CStr {
        self.0
    }

    /// Converts this to a Rust string slice.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not properly UTF-8 encoded.
    pub fn to_str(&self) -> &str {
        self.0
            .to_str()
            .expect("REAPER string should be UTF-8 encoded")
    }
}
