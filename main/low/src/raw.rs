//! Exposes important raw types, functions and constants from the C++ REAPER API.
#![allow(non_camel_case_types)]

use std::ffi::c_char;
use std::os::raw::{c_int, c_void};

/// Structs, types and constants defined by REAPER.
pub use super::bindings::root::{
    accelerator_register_t, audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output,
    midi_realtime_write_struct_t, preview_register_t, reaper_plugin_info_t, IReaperControlSurface,
    IReaperPitchShift, KbdCmd, KbdSectionInfo, MIDI_event_t, MIDI_eventlist, MediaItem,
    MediaItem_Take, MediaTrack, PCM_sink, PCM_source, PCM_source_peaktransfer_t,
    PCM_source_transfer_t, ProjectStateContext, REAPER_Resample_Interface, ReaProject, ReaSample,
    TrackEnvelope, WDL_HeapBuf, CSURF_EXT_RESET, CSURF_EXT_SETBPMANDPLAYRATE,
    CSURF_EXT_SETFOCUSEDFX, CSURF_EXT_SETFXCHANGE, CSURF_EXT_SETFXENABLED, CSURF_EXT_SETFXOPEN,
    CSURF_EXT_SETFXPARAM, CSURF_EXT_SETFXPARAM_RECFX, CSURF_EXT_SETINPUTMONITOR,
    CSURF_EXT_SETLASTTOUCHEDFX, CSURF_EXT_SETPAN_EX, CSURF_EXT_SETPROJECTMARKERCHANGE,
    CSURF_EXT_SETRECVPAN, CSURF_EXT_SETRECVVOLUME, CSURF_EXT_SETSENDPAN, CSURF_EXT_SETSENDVOLUME,
    CSURF_EXT_SUPPORTS_EXTENDED_TOUCH, CSURF_EXT_TRACKFX_PRESET_CHANGED,
    PCM_SOURCE_EXT_EXPORTTOFILE, PCM_SOURCE_EXT_GETMIDIDATAHASH, PCM_SOURCE_EXT_GETPOOLEDMIDIID,
    PCM_SOURCE_EXT_OPENEDITOR, PCM_SOURCE_EXT_SETPREVIEWTEMPO, REAPER_PITCHSHIFT_API_VER,
    REAPER_PLUGIN_VERSION, RESAMPLE_EXT_SETRSMODE, UNDO_STATE_ALL, UNDO_STATE_FREEZE,
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
    midi_quantize_mode_t, ACCEL, BM_GETCHECK, BM_GETIMAGE, BM_SETCHECK, BM_SETIMAGE, BOOL,
    BST_CHECKED, BST_INDETERMINATE, BST_UNCHECKED, CBN_CLOSEUP, CBN_DROPDOWN, CBN_EDITCHANGE,
    CBN_SELCHANGE, CB_ADDSTRING, CB_DELETESTRING, CB_FINDSTRING, CB_FINDSTRINGEXACT, CB_GETCOUNT,
    CB_GETCURSEL, CB_GETITEMDATA, CB_GETLBTEXT, CB_GETLBTEXTLEN, CB_INITSTORAGE, CB_INSERTSTRING,
    CB_RESETCONTENT, CB_SETCURSEL, CB_SETITEMDATA, COLOR_3DDKSHADOW, COLOR_3DFACE, COLOR_3DHILIGHT,
    COLOR_3DSHADOW, COLOR_BTNFACE, COLOR_BTNTEXT, COLOR_INFOBK, COLOR_INFOTEXT, COLOR_SCROLLBAR,
    COLOR_WINDOW, DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DT_BOTTOM, DT_CALCRECT, DT_CENTER,
    DT_END_ELLIPSIS, DT_LEFT, DT_NOCLIP, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_TOP, DT_VCENTER,
    DT_WORDBREAK, EN_CHANGE, EN_KILLFOCUS, EN_SETFOCUS, GMEM_DDESHARE, GMEM_DISCARDABLE,
    GMEM_FIXED, GMEM_LOWER, GMEM_MOVEABLE, GMEM_SHARE, GMEM_ZEROINIT, GUID, GWL_EXSTYLE,
    GWL_HWNDPARENT, GWL_ID, GWL_STYLE, GWL_WNDPROC, GW_CHILD, HANDLE, HBRUSH, HDC, HDC__,
    HGDIOBJ__, HINSTANCE, HMENU, HMENU__, HWND, HWND__, IDABORT, IDCANCEL, IDIGNORE, IDNO, IDOK,
    IDRETRY, IDYES, INT_PTR, LPARAM, LPSTR, LRESULT, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONSTOP,
    MB_OK, MB_OKCANCEL, MB_RETRYCANCEL, MB_YESNO, MB_YESNOCANCEL, MENUITEMINFO, MF_BITMAP,
    MF_BYCOMMAND, MF_BYPOSITION, MF_CHECKED, MF_DISABLED, MF_ENABLED, MF_GRAYED, MF_POPUP,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MIIM_BITMAP, MSG, PAINTSTRUCT,
    PCM_SINK_EXT_CREATESOURCE, PCM_SOURCE_EXT_ADDMIDIEVENTS, PCM_SOURCE_EXT_NOTIFYPREVIEWPLAYPOS,
    PCM_SOURCE_EXT_REMOVEFROMMIDIPOOL, POINT, RECT, SB_BOTH, SB_BOTTOM, SB_CTL, SB_ENDSCROLL,
    SB_HORZ, SB_LEFT, SB_LINEDOWN, SB_LINELEFT, SB_LINERIGHT, SB_LINEUP, SB_PAGEDOWN, SB_PAGELEFT,
    SB_PAGERIGHT, SB_PAGEUP, SB_RIGHT, SB_THUMBPOSITION, SB_THUMBTRACK, SB_TOP, SB_VERT,
    SCROLLINFO, SIF_ALL, SIF_DISABLENOSCROLL, SIF_PAGE, SIF_POS, SIF_RANGE, SIF_TRACKPOS,
    SM_CXHSCROLL, SM_CXSCREEN, SM_CXVSCROLL, SM_CYHSCROLL, SM_CYMENU, SM_CYSCREEN, SM_CYVSCROLL,
    SRCCOPY, SRCCOPY_USEALPHACHAN, TPM_BOTTOMALIGN, TPM_CENTERALIGN, TPM_HORIZONTAL, TPM_LEFTALIGN,
    TPM_LEFTBUTTON, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTALIGN, TPM_RIGHTBUTTON, TPM_TOPALIGN,
    TPM_VCENTERALIGN, TPM_VERTICAL, UINT, ULONG_PTR, VK_ADD, VK_BACK, VK_CAPITAL,
    VK_CLEAR, VK_CONTROL, VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
    VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F2, VK_F20,
    VK_F21, VK_F22, VK_F23, VK_F24, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HELP,
    VK_HOME, VK_INSERT, VK_LBUTTON, VK_LEFT, VK_LWIN, VK_MBUTTON, VK_MENU, VK_MULTIPLY, VK_NEXT,
    VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6,
    VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_PAUSE, VK_PRINT, VK_PRIOR, VK_RBUTTON, VK_RETURN,
    VK_RIGHT, VK_SCROLL, VK_SELECT, VK_SEPARATOR, VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT,
    VK_TAB, VK_UP, WDL_INT64, WM_ACTIVATE, WM_ACTIVATEAPP, WM_CAPTURECHANGED, WM_CHAR, WM_CLOSE,
    WM_COMMAND, WM_CONTEXTMENU, WM_COPYDATA, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLORDLG,
    WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORMSGBOX, WM_CTLCOLORSCROLLBAR,
    WM_CTLCOLORSTATIC, WM_DEADCHAR, WM_DESTROY, WM_DISPLAYCHANGE, WM_DRAWITEM, WM_DROPFILES,
    WM_ERASEBKGND, WM_GESTURE, WM_GETFONT, WM_GETMINMAXINFO, WM_GETOBJECT, WM_HSCROLL,
    WM_INITDIALOG, WM_INITMENUPOPUP, WM_KEYDOWN, WM_KEYFIRST, WM_KEYLAST, WM_KEYUP,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEACTIVATE, WM_MOUSEFIRST, WM_MOUSEHWHEEL, WM_MOUSELAST, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_MOVE, WM_NCCALCSIZE, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN,
    WM_NCLBUTTONUP, WM_NCMBUTTONDBLCLK, WM_NCMBUTTONDOWN, WM_NCMBUTTONUP, WM_NCMOUSEMOVE,
    WM_NCPAINT, WM_NCRBUTTONDBLCLK, WM_NCRBUTTONDOWN, WM_NCRBUTTONUP, WM_NOTIFY, WM_PAINT,
    WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETCURSOR, WM_SETFOCUS, WM_SETFONT,
    WM_SETREDRAW, WM_SETTEXT, WM_SHOWWINDOW, WM_SIZE, WM_STYLECHANGED, WM_SYSCHAR, WM_SYSCOMMAND,
    WM_SYSDEADCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER, WM_USER, WM_VSCROLL, WPARAM, WS_CAPTION,
    WS_SYSMENU, WS_THICKFRAME,
};

// Some constants which are calculated from other constants are not picked up by bindgen.
pub const TBM_GETPOS: u32 = WM_USER;
pub const TBM_SETTIC: u32 = WM_USER + 4;
pub const TBM_SETPOS: u32 = WM_USER + 5;
pub const TBM_SETRANGE: u32 = WM_USER + 6;
pub const TBM_SETSEL: u32 = WM_USER + 10;

// Some constants/types are different in Unix/SWELL. Search for "these differ" in SWELL source code
// for explanation.
#[cfg(target_family = "unix")]
pub use crate::bindings::root::{
    // Mutex
    pthread_mutex_t,
    // MIIM
    MIIM_DATA,
    MIIM_ID,
    MIIM_STATE,
    MIIM_SUBMENU,
    MIIM_TYPE,
    // SWP
    SWP_FRAMECHANGED,
    SWP_NOACTIVATE,
    SWP_NOCOPYBITS,
    SWP_NOMOVE,
    SWP_NOSIZE,
    SWP_NOZORDER,
    SWP_SHOWWINDOW,
    // SW
    SW_HIDE,
    SW_NORMAL,
    SW_RESTORE,
    SW_SHOW,
    SW_SHOWDEFAULT,
    SW_SHOWMAXIMIZED,
    SW_SHOWMINIMIZED,
    SW_SHOWNA,
    SW_SHOWNOACTIVATE,
    SW_SHOWNORMAL,
    // GDI
    TRANSPARENT,
    OPAQUE,
    NULL_BRUSH,
    NULL_PEN,
};

#[cfg(target_family = "windows")]
mod windows_constants {
    // MIIM
    pub const MIIM_STATE: u32 = 0x00000001;
    pub const MIIM_ID: u32 = 0x00000002;
    pub const MIIM_DATA: u32 = 0x00000020;
    pub const MIIM_SUBMENU: u32 = 0x00000004;
    pub const MIIM_TYPE: u32 = 0x00000010;
    // SWP
    pub const SWP_FRAMECHANGED: u32 = 0x0020;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_NOCOPYBITS: u32 = 0x0100;
    pub const SWP_NOMOVE: u32 = 0x0002;
    pub const SWP_NOSIZE: u32 = 0x0001;
    pub const SWP_NOZORDER: u32 = 0x0004;
    pub const SWP_SHOWWINDOW: u32 = 0x0040;
    // SW
    pub const SW_HIDE: i32 = 0;
    pub const SW_NORMAL: i32 = 1;
    pub const SW_RESTORE: i32 = 9;
    pub const SW_SHOW: i32 = 5;
    pub const SW_SHOWDEFAULT: i32 = 10;
    pub const SW_SHOWMAXIMIZED: i32 = 3;
    pub const SW_SHOWMINIMIZED: i32 = 2;
    pub const SW_SHOWNA: i32 = 8;
    pub const SW_SHOWNOACTIVATE: i32 = 4;
    pub const SW_SHOWNORMAL: i32 = 1;
    // GDI
    pub const TRANSPARENT: u32 = 1;
    pub const OPAQUE: u32 = 2;
    pub const NULL_BRUSH: u32 = 5;
    pub const NULL_PEN: u32 = 8;
}

#[cfg(target_family = "windows")]
pub use windows_constants::*;

/// Function pointer type for hook commands.
pub type HookCommand = extern "C" fn(command_id: c_int, flag: c_int) -> bool;

/// Function pointer type for hook commands with MIDI/OSC support.
pub type HookCommand2 = extern "C" fn(
    sec: *mut KbdSectionInfo,
    command_id: c_int,
    val: c_int,
    valhw: c_int,
    relmode: c_int,
    hwnd: HWND,
) -> bool;

/// Function pointer type for a menu hook function that is called when a customizable REAPER menu is initialized or
/// shown.
pub type HookCustomMenu = extern "C" fn(menuidstr: *const c_char, menu: HMENU, flag: c_int);

/// Function pointer type for toggle actions.
pub type ToggleAction = extern "C" fn(command_id: c_int) -> c_int;

/// Function pointer type for getting notified about invocation of hook command.
pub type HookPostCommand = extern "C" fn(command_id: c_int, flag: c_int);

/// Function pointer type for timers.
pub type TimerFunction = extern "C" fn();

/// Function pointer type for getting notified about invocation of hook command 2.
pub type HookPostCommand2 = extern "C" fn(
    section: *mut KbdSectionInfo,
    action_command_id: c_int,
    val: c_int,
    valhw: c_int,
    relmode: c_int,
    hwnd: HWND,
    proj: *mut ReaProject,
);

/// Function pointer type for exposing custom API functions to ReaScript.
pub type ApiVararg = unsafe extern "C" fn(*mut *mut c_void, c_int) -> *mut c_void;
