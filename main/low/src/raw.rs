//! Exposes important raw types, functions and constants from the C++ REAPER API.

use std::os::raw::c_int;

/// Structs, types and constants defined by REAPER.
pub use super::bindings::root::{
    audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, reaper_plugin_info_t,
    IReaperControlSurface, KbdCmd, KbdSectionInfo, MIDI_event_t, MIDI_eventlist, MediaItem,
    MediaItem_Take, MediaTrack, PCM_source, ReaProject, ReaSample, TrackEnvelope,
    CSURF_EXT_SETBPMANDPLAYRATE, CSURF_EXT_SETFOCUSEDFX, CSURF_EXT_SETFXCHANGE,
    CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN, CSURF_EXT_SETFXPARAM, CSURF_EXT_SETFXPARAM_RECFX,
    CSURF_EXT_SETINPUTMONITOR, CSURF_EXT_SETLASTTOUCHEDFX, CSURF_EXT_SETSENDPAN,
    CSURF_EXT_SETSENDVOLUME, REAPER_PLUGIN_VERSION, UNDO_STATE_ALL, UNDO_STATE_FREEZE,
    UNDO_STATE_FX, UNDO_STATE_ITEMS, UNDO_STATE_MISCCFG, UNDO_STATE_TRACKCFG,
};

/// Structs, types and constants defined by `swell.h` (on Linux and Mac OS X) and
/// `windows.h` (on Windows).
///
/// When exposing a Windows API struct/types or a REAPER struct which contains Windows API
/// structs/types, it would be good to recheck if its binary representation is the same
/// in the Windows-generated `bindings.rs` (based on `windows.h`) as in the
/// Linux-generated `bindings.rs` (based on `swell.h`). If not, that can introduce
/// cross-platform issues.
///
/// It seems SWELL itself does a pretty good job already to keep the binary representations
/// the same. E.g. `DWORD` ends up as `c_ulong` on Windows (= `u32` on Windows) and
/// `c_uint` on Linux (= `u32` on Linux).
pub use super::bindings::root::{
    ACCEL, DLL_PROCESS_ATTACH, GUID, HINSTANCE, HWND, HWND__, LPARAM, LRESULT, SW_SHOW, UINT,
    VK_CONTROL, VK_MENU, VK_SHIFT, WM_ACTIVATE, WM_ACTIVATEAPP, WM_CAPTURECHANGED, WM_CHAR,
    WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_COPYDATA, WM_CREATE, WM_DEADCHAR, WM_DESTROY,
    WM_DISPLAYCHANGE, WM_DRAWITEM, WM_DROPFILES, WM_ERASEBKGND, WM_GESTURE, WM_GETFONT,
    WM_GETMINMAXINFO, WM_GETOBJECT, WM_HSCROLL, WM_INITDIALOG, WM_INITMENUPOPUP, WM_KEYDOWN,
    WM_KEYFIRST, WM_KEYLAST, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEFIRST,
    WM_MOUSEHWHEEL, WM_MOUSELAST, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_MOVE, WM_NCCALCSIZE,
    WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP,
    WM_NCMBUTTONDBLCLK, WM_NCMBUTTONDOWN, WM_NCMBUTTONUP, WM_NCMOUSEMOVE, WM_NCPAINT,
    WM_NCRBUTTONDBLCLK, WM_NCRBUTTONDOWN, WM_NCRBUTTONUP, WM_NOTIFY, WM_PAINT, WM_RBUTTONDBLCLK,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETCURSOR, WM_SETFONT, WM_SETREDRAW, WM_SETTEXT,
    WM_SHOWWINDOW, WM_SIZE, WM_STYLECHANGED, WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSDEADCHAR,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER, WM_USER, WM_VSCROLL, WPARAM,
};

/// Function pointer type for hook commands.
pub type HookCommandFn = extern "C" fn(command_id: c_int, flag: c_int) -> bool;

/// Function pointer type for toggle actions.
pub type ToggleActionFn = extern "C" fn(command_id: c_int) -> c_int;

/// Function pointer type for hook post commands.
pub type HookPostCommandFn = extern "C" fn(command_id: c_int, flag: c_int);
