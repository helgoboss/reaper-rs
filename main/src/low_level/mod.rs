mod bindings;
pub use bindings::root::{
    audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, reaper_plugin_info_t,
    reaper_rs_midi::*, GetActiveWindow, IReaperControlSurface, KbdCmd, KbdSectionInfo,
    MIDI_event_t, MediaTrack, ReaProject, TrackEnvelope, ACCEL, CSURF_EXT_SETBPMANDPLAYRATE,
    CSURF_EXT_SETFOCUSEDFX, CSURF_EXT_SETFXCHANGE, CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN,
    CSURF_EXT_SETFXPARAM, CSURF_EXT_SETFXPARAM_RECFX, CSURF_EXT_SETINPUTMONITOR,
    CSURF_EXT_SETLASTTOUCHEDFX, CSURF_EXT_SETSENDPAN, CSURF_EXT_SETSENDVOLUME, GUID, HINSTANCE,
    HWND, REAPER_PLUGIN_VERSION,
};

mod control_surface;
pub use control_surface::*;

mod util;
pub use util::*;

mod reaper_plugin_context;
pub use reaper_plugin_context::*;

mod reaper;
pub use reaper::*;
