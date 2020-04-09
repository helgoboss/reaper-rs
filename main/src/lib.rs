#![cfg_attr(feature = "high-level", feature(fn_traits, clamp, backtrace))]

// The high-level API currently requires nightly features:
// - fn_traits: For calling hook commands. I think this can be avoided somehow.
// - clamp: Could easily be replaced with clamp from num crate.
// - backtrace: For logging. Could be replaced with crate.
//
// For now let's leave things as they are. It's impossible anyway to make the high-level API work on
// stable channel as long as rxRust still relies on nightly features. Moreover I don't consider it
// as a showstopper that it doesn't work on stable channel yet. reaper-rs high-level API will most
// likely not be used by other universal Rust libraries, but only in final plugins. So its
// nightly-nature is not contaminating.

#[cfg(feature = "high-level")]
pub mod high_level;

#[cfg(feature = "medium-level")]
pub mod medium_level;

pub mod low_level;
