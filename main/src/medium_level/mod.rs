//! Provides all functions from `reaper_plugin_functions.h` with the following small improvements:
//! TODO
//! The medium-level API offers much more type safety and convenience. Still stays close to
//! original REAPER API and the ultimate goal is to expose every function with every possible
//! calling style of the low-level API, just
//! in a bit nicer and type-safe manner - so that at the end you don't have to resort to the
//! low-level API anymore and this gets a complete replacement. Some low-level functions can't be
//! rewritten in a type-safe way. In this case, new convenience functions are introduced.
//! Quite likely that someone who uses Rust (instead of e.g. Lua) does it also because of
//! performance reasons. medium-level API can be considered the first API. The low-level one is not
//! really supposed to be used directly. It's important that the "first" API still is sensitive
//! about performance and doesn't do too much extra.
//!
//! - Note about strings (both return and parameter)!
//! - When I say "index", I always mean zero-based
//!
//! - Snake-case function and parameter names
//! - Use bool instead of i32 as return value type for functions with obvious "yes or no" result
//! - Use ReaperStringPtr instead of raw c_char pointers as return value type (offers convenience
//!   functions)
//! - Use ReaperVoidPtr instead of raw c_void pointers as return value type (offers convenience
//!   functions)
//! - Use return values instead of output parameters
//! - When there are string output parameters which can be passed a null pointer, trigger this null
//!   pointer case when a buffer size of 0 is passed, also use Cow in this case in order to have a
//!   cheap empty string in null-pointer case
//! - When there are both return values and output parameters, return a tuple if there's just one
//!   output parameter and a struct if there are many output parameters
//! - In all REAPER functions which can fail (mostly indicated by returning false or -1), return
//!   Result
//! - In all REAPER functions which return things that might not be present, return Option
//! - Panics if function not available (we should make sure on plug-in load that all necessary
//!   functions are available)
//! - More restrictive number types where safely applicable (for increased safety, e.g. u32 instead
//!   of i32). In the unlikely case that the value range has to be extended in future, it's just a
//!   matter of removing safe casts on user-side code.

mod constants;
pub use constants::*;

mod control_surface;
pub use control_surface::*;

mod reaper;
pub use reaper::*;

mod util;
pub use util::*;

mod string_types;
pub use string_types::*;

mod recording_input;
pub use recording_input::*;
