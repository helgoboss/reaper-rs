//! This module contains the low-level API of *reaper-rs*.
//!
//! It is not recommended to use this API directly because it just exposes the raw REAPER C++
//! functions, types and constants one to one in Rust. If you want idiomatic Rust, type safety and
//! convenience, please use the [medium-level] or [high-level] API instead.
//!
//! At times it can still be useful to access the low-level API, mostly as fallback if the function
//! that you are looking for has not yet been lifted to the medium-level API. To get started, best
//! navigate to the [`Reaper`] struct, which contains all exposed functions.
//!
//! # Design
//!
//! ## Goal
//!
//! The ultimate goal of the low-level API is to be on par with the REAPER C++ SDK, meaning
//! that everything which is possible with the REAPER C++ SDK is also possible with the *reaper-rs*
//! low-level API. Improvements regarding safety, convenience or style are not in its scope. It
//! should serve as a solid base for more idiomatic APIs built on top of it.
//!
//! ## Generated code
//!
//! Most parts of the low-level API are auto-generated from `reaper_plugin_functions.h` using a
//! combination of [bindgen](https://docs.rs/bindgen) and custom build script.
//!
//! ## C++ glue code
//!
//! There's some code which is not auto-generated, most notably the code to "restore" functionality
//! which "got lost in translation". The problem is that some parts of the REAPER SDK not just use
//! C but also C++ features, in particular virtual base classes. Rust can't call virtual functions
//! or implement them.
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
//! [medium-level]: /reaper_rs_medium/index.html
//! [high-level]: /reaper_rs_high/index.html
//! [`Reaper`]: struct.Reaper.html
mod bindings;

pub mod raw {
    //! Exposes important raw types, functions and constants from the C++ REAPER SDK.
    pub use super::bindings::root::{
        audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, reaper_plugin_info_t,
        IReaperControlSurface, KbdCmd, KbdSectionInfo, MIDI_event_t, MIDI_eventlist, MediaItem,
        MediaItem_Take, MediaTrack, PCM_source, ReaProject, ReaSample, TrackEnvelope, ACCEL,
        CSURF_EXT_SETBPMANDPLAYRATE, CSURF_EXT_SETFOCUSEDFX, CSURF_EXT_SETFXCHANGE,
        CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN, CSURF_EXT_SETFXPARAM,
        CSURF_EXT_SETFXPARAM_RECFX, CSURF_EXT_SETINPUTMONITOR, CSURF_EXT_SETLASTTOUCHEDFX,
        CSURF_EXT_SETSENDPAN, CSURF_EXT_SETSENDVOLUME, GUID, HINSTANCE, HWND, HWND__,
        REAPER_PLUGIN_VERSION, UNDO_STATE_ALL, UNDO_STATE_FREEZE, UNDO_STATE_FX, UNDO_STATE_ITEMS,
        UNDO_STATE_MISCCFG, UNDO_STATE_TRACKCFG, VK_CONTROL, VK_MENU, VK_SHIFT,
    };
}

mod control_surface;
pub use control_surface::*;

mod util;
pub use util::*;

mod reaper_plugin_context;
pub use reaper_plugin_context::*;

mod reaper;
pub use reaper::*;

mod reaper_impl;
pub use reaper_impl::*;

mod midi;
pub use midi::*;
