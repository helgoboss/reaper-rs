#![cfg_attr(feature = "high-level", feature(fn_traits, clamp, backtrace))]
//!
//! Currently required nightly features:
//! - fn_traits: In high-level API for calling hook commands. I think there must be an easy
//!   workaround.
//! - clamp: In high-level API. Could be easily replaced with clamp from num crate.
//! - backtrace: In high-level API for logging. Could be replaced with crate.
//!
//! For now leave things as they are. It's impossible anyway to make the high-level API work on
//! stable channel as long as rxRust still relies on nightly features. Moreover I don't consider it
//! as a showstopper that it doesn't work on stable channel yet. reaper-rs will most likely not be
//! used by other universal Rust libraries, but only in final plugins. So its nightly-nature is not
//! very contaminating.
//! TODO Wise rustfmt settings
#[cfg(feature = "high-level")]
pub mod high_level;
#[cfg(feature = "medium-level")]
pub mod medium_level;
pub mod low_level;
