#![doc(html_root_url = "https://docs.rs/reaper-low/0.1.0")]
#![deny(broken_intra_doc_links)]

//! This crate contains the low-level API of [reaper-rs](https://github.com/helgoboss/reaper-rs).
//!
//! It is not recommended to use this API directly because it just exposes the raw REAPER C++
//! functions, types and constants one to one in Rust. If you want idiomatic Rust, type safety and
//! convenience, please use the [medium-level] or high-level API instead.
//!
//! At times it can still be useful to access the low-level API, mostly as fallback if the function
//! that you are looking for has not yet been lifted to the medium-level API. To get started, best
//! navigate to the [`Reaper`] struct, which contains all exposed functions.
//!
//! # Example
//!
//! ```no_run
//! # let reaper = reaper_low::Reaper::default();
//! use c_str_macro::c_str;
//! use std::ptr::null_mut;
//!
//! unsafe {
//!     reaper.ShowConsoleMsg(c_str!("Hello world from reaper-rs low-level API!").as_ptr());
//!     let track = reaper.GetTrack(null_mut(), 0);
//!     reaper.DeleteTrack(track);
//! }
//! ```
//!
//! # Design
//!
//! ## Goal
//!
//! The ultimate goal of the low-level API is to be on par with the REAPER C++ API, meaning
//! that everything which is possible with the REAPER C++ API is also possible with the *reaper-rs*
//! low-level API. Improvements regarding safety, convenience or style are not in its scope. It
//! should serve as a base for more idiomatic APIs built on top of it.
//!
//! ## Generated code
//!
//! Most parts of the low-level API are auto-generated from `reaper_plugin_functions.h` using a
//! combination of [bindgen](https://docs.rs/bindgen) and custom build script.
//!
//! ## C++ glue code
//!
//! There's some code which is not auto-generated, most notably the code to "restore" functionality
//! which "got lost in translation". The problem is that some parts of the REAPER C++ API not just
//! use C but also C++ features, in particular virtual base classes. Rust can't call virtual
//! functions or implement them.
//!
//! The solution is to take a detour via C++ glue code:
//!
//! - Rust calling a C++ virtual function provided by REAPER:
//!     - Implement a method on the raw struct in Rust which calls a function written in C which in
//!       turn calls the C++ virtual function (Rust function → C function → C++ virtual function)
//!     - Example: `midi.rs` & `midi.cpp`
//!
//! - REAPER calling a C++ virtual function provided by Rust:
//!     - Implement the virtual base class in C++, let each function delegate to a corresponding
//!       free Rust function which in turn calls a method of a trait object (REAPER → C++ virtual
//!       function → Rust function)
//!     - Example: `control_surface.cpp` & `control_surface.rs`
//!
//! [medium-level]: https://docs.rs/reaper-medium
//! [`Reaper`]: struct.Reaper.html
#[macro_use]
mod macros;

mod bindings;

pub mod raw;

mod control_surface;
pub use control_surface::*;

mod util;
pub use util::*;

mod plugin_context;
pub use plugin_context::*;

mod reaper;
pub use reaper::*;

mod reaper_impl;
pub use reaper_impl::*;

mod swell;
pub use swell::*;

mod static_context;
pub use static_context::*;

mod swell_impl;

mod midi;
pub use midi::*;

mod pcm_source;
pub use pcm_source::*;
