//! This module contains the low-level API of *reaper-rs*, meaning that it exposes the raw C++
//! REAPER functions one to one, nothing more and nothing less. If you want "Rust feeling" or
//! additional convenience, use the [medium-level](../medium_level/index.html) or
//! [high-level](../high_level/index.html) API.
//!
//! Most parts of the low-level API are auto-generated from `reaper_plugin_functions.h`.
//! For a list of all exposed functions, have a look at the [Reaper](struct.Reaper.html) type.
mod bindings;

pub mod raw {
    //! Exposes a few important raw types, functions and constants from the C++ REAPER SDK.
    pub use super::bindings::root::{
        audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, reaper_plugin_info_t,
        GetActiveWindow, IReaperControlSurface, KbdCmd, KbdSectionInfo, MIDI_event_t,
        MIDI_eventlist, MediaItem, MediaItem_Take, MediaTrack, PCM_source, ReaProject,
        TrackEnvelope, ACCEL, CSURF_EXT_SETBPMANDPLAYRATE, CSURF_EXT_SETFOCUSEDFX,
        CSURF_EXT_SETFXCHANGE, CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN, CSURF_EXT_SETFXPARAM,
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

mod midi;
pub use midi::*;
