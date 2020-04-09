//! Provides all functions from `reaper_plugin_functions.h` with the following small improvements:
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
mod control_surface;
mod reaper;
mod util;
pub use util::*;

pub use constants::*;
pub use control_surface::{ControlSurface, DelegatingControlSurface};
pub use reaper::*;
