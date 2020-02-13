//! Types copied from generated bindings
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::{ReaProject, MediaTrack};
use crate::low_level::bindings::root::{HWND, IReaperControlSurface};
use crate::low_level::{KbdSectionInfo, TrackEnvelope, GUID};
use crate::low_level::bindings::root;

pub type GetFunc = unsafe extern "C" fn(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void;

pub type EnumProjects = fn(
    idx: ::std::os::raw::c_int,
    projfnOutOptional: *mut ::std::os::raw::c_char,
    projfnOutOptional_sz: ::std::os::raw::c_int,
) -> *mut ReaProject;

pub type GetTrack = fn(
    proj: *mut ReaProject,
    trackidx: ::std::os::raw::c_int,
) -> *mut MediaTrack;

pub type ValidatePtr2 = fn(
    proj: *mut ReaProject,
    pointer: *mut ::std::os::raw::c_void,
    ctypename: *const ::std::os::raw::c_char,
) -> bool;

pub type GetSetMediaTrackInfo = fn(
    tr: *mut MediaTrack,
    parmname: *const ::std::os::raw::c_char,
    setNewValue: *mut ::std::os::raw::c_void,
) -> *mut ::std::os::raw::c_void;

pub type ShowConsoleMsg = fn(msg: *const ::std::os::raw::c_char);

pub type plugin_register = fn(
    name: *const ::std::os::raw::c_char,
    infostruct: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int;

pub type GetMainHwnd = fn() -> HWND;

pub type KBD_OnMainActionEx = fn(
    cmd: ::std::os::raw::c_int,
    val: ::std::os::raw::c_int,
    valhw: ::std::os::raw::c_int,
    relmode: ::std::os::raw::c_int,
    hwnd: HWND,
    proj: *mut ReaProject,
) -> ::std::os::raw::c_int;

pub type SectionFromUniqueID = fn(uniqueID: ::std::os::raw::c_int) -> *mut KbdSectionInfo;

pub type NamedCommandLookup = fn(
    command_name: *const ::std::os::raw::c_char,
) -> ::std::os::raw::c_int;

pub type ClearConsole = fn();

pub type CountTracks = fn(proj: *mut ReaProject) -> ::std::os::raw::c_int;

pub type InsertTrackAtIndex = fn(idx: ::std::os::raw::c_int, wantDefaults: bool);

pub type TrackList_UpdateAllExternalSurfaces = fn();

pub type GetMediaTrackInfo_Value = fn(
    tr: *mut MediaTrack,
    parmname: *const ::std::os::raw::c_char,
) -> f64;

pub type GetAppVersion = fn() -> *const ::std::os::raw::c_char;

pub type GetTrackEnvelopeByName = fn(
    track: *mut MediaTrack,
    envname: *const ::std::os::raw::c_char,
) -> *mut TrackEnvelope;

pub type GetTrackAutomationMode = fn(tr: *mut MediaTrack) -> ::std::os::raw::c_int;

pub type GetGlobalAutomationOverride = fn() -> ::std::os::raw::c_int;

pub type TrackFX_GetRecCount = fn(track: *mut MediaTrack) -> ::std::os::raw::c_int;

pub type TrackFX_GetCount = fn(track: *mut MediaTrack) -> ::std::os::raw::c_int;

pub type TrackFX_GetFXGUID = fn(
    track: *mut MediaTrack,
    fx: ::std::os::raw::c_int,
) -> *mut GUID;

pub type TrackFX_GetParamNormalized = fn(
    track: *mut MediaTrack,
    fx: ::std::os::raw::c_int,
    param: ::std::os::raw::c_int,
) -> f64;

pub type GetMasterTrack = fn(proj: *mut ReaProject) -> *mut MediaTrack;

pub type guidToString = fn(g: *const GUID, destNeed64: *mut ::std::os::raw::c_char);

pub type stringToGuid = fn(str: *const ::std::os::raw::c_char, g: *mut GUID);

pub type CSurf_OnInputMonitorChangeEx = fn(
    trackid: *mut MediaTrack,
    monitor: ::std::os::raw::c_int,
    allowgang: bool,
) -> ::std::os::raw::c_int;

pub type SetMediaTrackInfo_Value = fn(
    tr: *mut MediaTrack,
    parmname: *const ::std::os::raw::c_char,
    newvalue: f64,
) -> bool;

pub type DB2SLIDER = fn(x: f64) -> f64;

pub type SLIDER2DB = fn(y: f64) -> f64;

pub type GetTrackUIVolPan = fn(
    track: *mut MediaTrack,
    volumeOut: *mut f64,
    panOut: *mut f64,
) -> bool;

pub type CSurf_OnVolumeChangeEx = fn(
    trackid: *mut MediaTrack,
    volume: f64,
    relative: bool,
    allowGang: bool,
) -> f64;

pub type CSurf_SetSurfaceVolume = fn(
    trackid: *mut MediaTrack,
    volume: f64,
    ignoresurf: *mut IReaperControlSurface,
);

pub type CSurf_OnPanChangeEx = fn(
    trackid: *mut MediaTrack,
    pan: f64,
    relative: bool,
    allowGang: bool,
) -> f64;

pub type CSurf_SetSurfacePan = fn(
    trackid: *mut MediaTrack,
    pan: f64,
    ignoresurf: *mut IReaperControlSurface,
);

pub type CountSelectedTracks2 = fn(
    proj: *mut ReaProject,
    wantmaster: bool,
) -> ::std::os::raw::c_int;

pub type SetTrackSelected = fn(track: *mut MediaTrack, selected: bool);

pub type GetSelectedTrack2 = fn(
    proj: *mut ReaProject,
    seltrackidx: ::std::os::raw::c_int,
    wantmaster: bool,
) -> *mut MediaTrack;

pub type SetOnlyTrackSelected = fn(track: *mut MediaTrack);

pub type GetTrackStateChunk = fn(
    track: *mut MediaTrack,
    strNeedBig: *mut ::std::os::raw::c_char,
    strNeedBig_sz: ::std::os::raw::c_int,
    isundoOptional: bool,
) -> bool;

pub type CSurf_OnRecArmChangeEx = fn(
    trackid: *mut MediaTrack,
    recarm: ::std::os::raw::c_int,
    allowgang: bool,
) -> bool;

pub type SetTrackStateChunk = fn(
    track: *mut MediaTrack,
    str: *const ::std::os::raw::c_char,
    isundoOptional: bool,
) -> bool;

pub type DeleteTrack = fn(tr: *mut MediaTrack);

pub type GetTrackNumSends = fn(
    tr: *mut MediaTrack,
    category: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int;

pub type GetSetTrackSendInfo = fn(
    tr: *mut MediaTrack,
    category: ::std::os::raw::c_int,
    sendidx: ::std::os::raw::c_int,
    parmname: *const ::std::os::raw::c_char,
    setNewValue: *mut ::std::os::raw::c_void,
) -> *mut ::std::os::raw::c_void;

pub type CreateTrackSend = fn(
    tr: *mut MediaTrack,
    desttrInOptional: *mut MediaTrack,
) -> ::std::os::raw::c_int;

pub type CSurf_OnSendVolumeChange = fn(
    trackid: *mut MediaTrack,
    send_index: ::std::os::raw::c_int,
    volume: f64,
    relative: bool,
) -> f64;

pub type CSurf_OnSendPanChange = fn(
    trackid: *mut MediaTrack,
    send_index: ::std::os::raw::c_int,
    pan: f64,
    relative: bool,
) -> f64;

pub type GetTrackSendUIVolPan = fn(
    track: *mut MediaTrack,
    send_index: ::std::os::raw::c_int,
    volumeOut: *mut f64,
    panOut: *mut f64,
) -> bool;

pub type kbd_getTextFromCmd = fn(
    cmd: root::DWORD,
    section: *mut root::KbdSectionInfo,
) -> *const ::std::os::raw::c_char;

pub type GetToggleCommandState2 = fn(
    section: *mut root::KbdSectionInfo,
    command_id: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int;

pub type ReverseNamedCommandLookup = fn(
    command_id: ::std::os::raw::c_int,
) -> *const ::std::os::raw::c_char;

pub type Main_OnCommandEx = fn(
    command: ::std::os::raw::c_int,
    flag: ::std::os::raw::c_int,
    proj: *mut root::ReaProject,
);

pub type CSurf_SetSurfaceMute = fn(
    trackid: *mut root::MediaTrack,
    mute: bool,
    ignoresurf: *mut root::IReaperControlSurface,
);

pub type CSurf_SetSurfaceSolo = fn(
    trackid: *mut root::MediaTrack,
    solo: bool,
    ignoresurf: *mut root::IReaperControlSurface,
);

pub type genGuid = fn(g: *mut root::GUID);

pub type GetMaxMidiInputs = fn() -> ::std::os::raw::c_int;

pub type GetMIDIInputName = fn(
    dev: ::std::os::raw::c_int,
    nameout: *mut ::std::os::raw::c_char,
    nameout_sz: ::std::os::raw::c_int,
) -> bool;

pub type GetMaxMidiOutputs = fn() -> ::std::os::raw::c_int;

pub type GetMIDIOutputName = fn(
    dev: ::std::os::raw::c_int,
    nameout: *mut ::std::os::raw::c_char,
    nameout_sz: ::std::os::raw::c_int,
) -> bool;