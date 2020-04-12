#![allow(non_camel_case_types)]
use crate::low_level::raw::{
    UNDO_STATE_FREEZE, UNDO_STATE_FX, UNDO_STATE_ITEMS, UNDO_STATE_MISCCFG, UNDO_STATE_TRACKCFG,
};
use crate::medium_level::ReaperStringArg;
use c_str_macro::c_str;
use enumflags2::BitFlags;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

pub type HookCommand = extern "C" fn(command_index: i32, _flag: i32) -> bool;
pub type ToggleAction = extern "C" fn(command_index: i32) -> i32;
pub type HookPostCommand = extern "C" fn(command_id: u32, _flag: i32);

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum FxShowFlag {
    HideChain = 0,
    ShowChain = 1,
    HideFloatingWindow = 2,
    ShowFloatingWindow = 3,
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackSendCategory {
    Receive = -1,
    Send = 0,
    HardwareOutput = 1,
}

impl From<SendOrReceive> for TrackSendCategory {
    fn from(v: SendOrReceive) -> Self {
        use SendOrReceive::*;
        match v {
            Receive => TrackSendCategory::Receive,
            Send => TrackSendCategory::Send,
        }
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum SendOrReceive {
    Receive = -1,
    Send = 0,
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum StuffMidiMessageTarget {
    VirtualMidiKeyboard = 0,
    MidiAsControlInputQueue = 1,
    VirtualMidiKeyboardOnCurrentChannel = 2,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReaperVersion {
    version_str: &'static CStr,
}

impl From<&'static CStr> for ReaperVersion {
    fn from(version_str: &'static CStr) -> Self {
        ReaperVersion { version_str }
    }
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum UndoFlag {
    Freeze = UNDO_STATE_FREEZE,
    Fx = UNDO_STATE_FX,
    Items = UNDO_STATE_ITEMS,
    MiscCfg = UNDO_STATE_MISCCFG,
    TrackCfg = UNDO_STATE_TRACKCFG,
}

// TODO Maybe call it TrackFxRef (in line with TrackRef and ProjectRef)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FxQueryIndex {
    InputFx(u32),
    OutputFx(u32),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
// Converts directly to the i32 value that is expected by low-level track-FX related functions
impl From<FxQueryIndex> for i32 {
    fn from(v: FxQueryIndex) -> Self {
        use FxQueryIndex::*;
        let positive = match v {
            InputFx(idx) => 0x1000000 + idx,
            OutputFx(idx) => idx,
        };
        positive as i32
    }
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
// Converts from a value returned by low-level track-FX related functions turned into u32.
impl From<u32> for FxQueryIndex {
    fn from(v: u32) -> Self {
        use FxQueryIndex::*;
        if v >= 0x1000000 {
            InputFx(v - 0x1000000)
        } else {
            OutputFx(v)
        }
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackFxAddByNameVariant {
    Add = -1,
    Query = 0,
    AddIfNotFound = 1,
}

pub enum KbdActionValue {
    AbsoluteLowRes(u8),   // TODO Maybe use U7 type
    AbsoluteHighRes(u16), // TODO Maybe use U14 type
    Relative1(u8),
    Relative2(u8),
    Relative3(u8),
}

pub enum RegInstr<'a> {
    Register(ExtensionType<'a>),
    Unregister(ExtensionType<'a>),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<RegInstr<'a>> for Cow<'a, CStr> {
    fn from(value: RegInstr<'a>) -> Self {
        use RegInstr::*;
        match value {
            Register(et) => et.into(),
            Unregister(et) => concat_c_strs(c_str!("-"), Cow::from(et).as_ref()).into(),
        }
    }
}

// TODO Make it possible for all Custom enum variants to pass any REAPER string. Must be documented.
pub enum ExtensionType<'a> {
    Api(Cow<'a, CStr>),
    ApiDef(Cow<'a, CStr>),
    HookCommand,
    HookPostCommand,
    HookCommand2,
    ToggleAction,
    ActionHelp,
    CommandId,
    CommandIdLookup,
    GAccel,
    CSurfInst,
    Custom(Cow<'a, CStr>),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<ExtensionType<'a>> for Cow<'a, CStr> {
    fn from(value: ExtensionType<'a>) -> Self {
        use ExtensionType::*;
        match value {
            GAccel => c_str!("gaccel").into(),
            CSurfInst => c_str!("csurf_inst").into(),
            Api(func_name) => concat_c_strs(c_str!("API_"), func_name.as_ref()).into(),
            ApiDef(func_def) => concat_c_strs(c_str!("APIdef_"), func_def.as_ref()).into(),
            HookCommand => c_str!("hookcommand").into(),
            HookPostCommand => c_str!("hookpostcommand").into(),
            HookCommand2 => c_str!("hookcommand2").into(),
            ToggleAction => c_str!("toggleaction").into(),
            ActionHelp => c_str!("action_help").into(),
            CommandId => c_str!("command_id").into(),
            CommandIdLookup => c_str!("command_id_lookup").into(),
            Custom(k) => k,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TrackRef {
    MasterTrack,
    TrackIndex(u32),
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum InputMonitoringMode {
    Off = 0,
    Normal = 1,
    /// Tape style
    NotWhenPlaying = 2,
}

pub enum ProjectRef {
    Current,
    CurrentlyRendering,
    TabIndex(u32),
}

/// Possible REAPER pointer types which can be passed to `Reaper::validate_ptr_2()`.
///
/// Except for the trailing asterisk, the variants are named exactly like the strings which will be
/// passed to the low-level REAPER function because the medium-level API is designed to still be
/// close to the raw REAPER API.
///
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
pub enum ReaperPointerType<'a> {
    MediaTrack,
    ReaProject,
    MediaItem,
    MediaItem_Take,
    TrackEnvelope,
    PCM_source,
    /// If a variant is missing in this enum, you can use this custom one as a last resort. Don't
    /// include the trailing asterisk (`*`)! It will be added to the call automatically.
    Custom(Cow<'a, CStr>),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<ReaperPointerType<'a>> for Cow<'a, CStr> {
    fn from(value: ReaperPointerType<'a>) -> Self {
        use ReaperPointerType::*;
        match value {
            MediaTrack => c_str!("MediaTrack*").into(),
            ReaProject => c_str!("ReaProject*").into(),
            MediaItem => c_str!("MediaItem*").into(),
            MediaItem_Take => c_str!("MediaItem_Take*").into(),
            TrackEnvelope => c_str!("TrackEnvelope*").into(),
            PCM_source => c_str!("PCM_source*").into(),
            Custom(name) => concat_c_strs(name.as_ref(), c_str!("*")).into(),
        }
    }
}

/// All the possible track info keys which you can pass to `Reaper::get_set_media_track_info()`.
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
/// The variants are named exactly like the strings which will be passed to the low-level REAPER
/// function because the medium-level API is designed to still be close to the raw REAPER API.  
pub enum TrackInfoKey<'a> {
    B_FREEMODE,
    B_HEIGHTLOCK,
    B_MAINSEND,
    B_MUTE,
    B_PHASE,
    B_SHOWINMIXER,
    B_SHOWINTCP,
    C_BEATATTACHMODE,
    C_MAINSEND_OFFS,
    D_DUALPANL,
    D_DUALPANR,
    D_PAN,
    D_PANLAW,
    D_PLAY_OFFSET,
    D_VOL,
    D_WIDTH,
    F_MCP_FXSEND_SCALE,
    F_MCP_SENDRGN_SCALE,
    GUID,
    I_AUTOMODE,
    I_CUSTOMCOLOR,
    I_FOLDERCOMPACT,
    I_FOLDERDEPTH,
    I_FXEN,
    I_HEIGHTOVERRIDE,
    I_MCPH,
    I_MCPW,
    I_MCPX,
    I_MCPY,
    I_MIDIHWOUT,
    I_NCHAN,
    I_PANMODE,
    I_PERFFLAGS,
    I_PLAY_OFFSET_FLAG,
    I_RECARM,
    I_RECINPUT,
    I_RECMODE,
    I_RECMON,
    I_RECMONITEMS,
    I_SELECTED,
    I_SOLO,
    I_TCPH,
    I_TCPY,
    I_WNDH,
    IP_TRACKNUMBER,
    P_ENV(EnvChunkName<'a>),
    P_EXT(Cow<'a, CStr>),
    P_ICON,
    P_MCP_LAYOUT,
    P_NAME,
    P_PARTRACK,
    P_PROJECT,
    P_TCP_LAYOUT,
    /// If a variant is missing in this enum, you can use this custom one as a last resort.
    Custom(Cow<'a, CStr>),
}

impl<'a> TrackInfoKey<'a> {
    pub fn p_ext(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::P_EXT(key.into().into_cow())
    }

    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::Custom(key.into().into_cow())
    }
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<TrackInfoKey<'a>> for Cow<'a, CStr> {
    fn from(value: TrackInfoKey<'a>) -> Self {
        use TrackInfoKey::*;
        match value {
            B_FREEMODE => c_str!("B_FREEMODE").into(),
            B_HEIGHTLOCK => c_str!("B_HEIGHTLOCK").into(),
            B_MAINSEND => c_str!("B_MAINSEND").into(),
            B_MUTE => c_str!("B_MUTE").into(),
            B_PHASE => c_str!("B_PHASE").into(),
            B_SHOWINMIXER => c_str!("B_SHOWINMIXER").into(),
            B_SHOWINTCP => c_str!("B_SHOWINTCP").into(),
            C_BEATATTACHMODE => c_str!("C_BEATATTACHMODE").into(),
            C_MAINSEND_OFFS => c_str!("C_MAINSEND_OFFS").into(),
            D_DUALPANL => c_str!("D_DUALPANL").into(),
            D_DUALPANR => c_str!("D_DUALPANR").into(),
            D_PAN => c_str!("D_PAN").into(),
            D_PANLAW => c_str!("D_PANLAW").into(),
            D_PLAY_OFFSET => c_str!("D_PLAY_OFFSET").into(),
            D_VOL => c_str!("D_VOL").into(),
            D_WIDTH => c_str!("D_WIDTH").into(),
            F_MCP_FXSEND_SCALE => c_str!("F_MCP_FXSEND_SCALE").into(),
            F_MCP_SENDRGN_SCALE => c_str!("F_MCP_SENDRGN_SCALE").into(),
            GUID => c_str!("GUID").into(),
            I_AUTOMODE => c_str!("I_AUTOMODE").into(),
            I_CUSTOMCOLOR => c_str!("I_CUSTOMCOLOR").into(),
            I_FOLDERCOMPACT => c_str!("I_FOLDERCOMPACT").into(),
            I_FOLDERDEPTH => c_str!("I_FOLDERDEPTH").into(),
            I_FXEN => c_str!("I_FXEN").into(),
            I_HEIGHTOVERRIDE => c_str!("I_HEIGHTOVERRIDE").into(),
            I_MCPH => c_str!("I_MCPH").into(),
            I_MCPW => c_str!("I_MCPW").into(),
            I_MCPX => c_str!("I_MCPX").into(),
            I_MCPY => c_str!("I_MCPY").into(),
            I_MIDIHWOUT => c_str!("I_MIDIHWOUT").into(),
            I_NCHAN => c_str!("I_NCHAN").into(),
            I_PANMODE => c_str!("I_PANMODE").into(),
            I_PERFFLAGS => c_str!("I_PERFFLAGS").into(),
            I_PLAY_OFFSET_FLAG => c_str!("I_PLAY_OFFSET_FLAG").into(),
            I_RECARM => c_str!("I_RECARM").into(),
            I_RECINPUT => c_str!("I_RECINPUT").into(),
            I_RECMODE => c_str!("I_RECMODE").into(),
            I_RECMON => c_str!("I_RECMON").into(),
            I_RECMONITEMS => c_str!("I_RECMONITEMS").into(),
            I_SELECTED => c_str!("I_SELECTED").into(),
            I_SOLO => c_str!("I_SOLO").into(),
            I_TCPH => c_str!("I_TCPH").into(),
            I_TCPY => c_str!("I_TCPY").into(),
            I_WNDH => c_str!("I_WNDH").into(),
            IP_TRACKNUMBER => c_str!("IP_TRACKNUMBER").into(),
            P_ENV(env_chunk_name) => {
                let cow: Cow<CStr> = env_chunk_name.into();
                concat_c_strs(c_str!("P_ENV:<"), cow.as_ref()).into()
            }
            P_EXT(extension_specific_key) => {
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key.as_ref()).into()
            }
            P_ICON => c_str!("P_ICON").into(),
            P_MCP_LAYOUT => c_str!("P_MCP_LAYOUT").into(),
            P_NAME => c_str!("P_NAME").into(),
            P_PARTRACK => c_str!("P_PARTRACK").into(),
            P_PROJECT => c_str!("P_PROJECT").into(),
            P_TCP_LAYOUT => c_str!("P_TCP_LAYOUT").into(),
            Custom(key) => key,
        }
    }
}

/// All the possible track send info keys which you can pass to `Reaper::get_set_track_send_info()`.
///
/// The variants are named exactly like the strings which will be passed to the low-level REAPER
/// function because the medium-level API is designed to still be close to the raw REAPER API.  
///
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
pub enum TrackSendInfoKey<'a> {
    B_MONO,
    B_MUTE,
    B_PHASE,
    D_PAN,
    D_PANLAW,
    D_VOL,
    I_AUTOMODE,
    I_DSTCHAN,
    I_MIDIFLAGS,
    I_SENDMODE,
    I_SRCCHAN,
    P_DESTTRACK,
    P_SRCTRACK,
    P_ENV(EnvChunkName<'a>),
    P_EXT(Cow<'a, CStr>),
    /// If a variant is missing in this enum, you can use this custom one as a last resort.
    Custom(Cow<'a, CStr>),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<TrackSendInfoKey<'a>> for Cow<'a, CStr> {
    fn from(value: TrackSendInfoKey<'a>) -> Self {
        use TrackSendInfoKey::*;
        match value {
            B_MONO => c_str!("B_MONO").into(),
            B_MUTE => c_str!("B_MUTE").into(),
            B_PHASE => c_str!("B_PHASE").into(),
            D_PAN => c_str!("D_PAN").into(),
            D_PANLAW => c_str!("D_PANLAW").into(),
            D_VOL => c_str!("D_VOL").into(),
            I_AUTOMODE => c_str!("I_AUTOMODE").into(),
            I_DSTCHAN => c_str!("I_DSTCHAN").into(),
            I_MIDIFLAGS => c_str!("I_MIDIFLAGS").into(),
            I_SENDMODE => c_str!("I_SENDMODE").into(),
            I_SRCCHAN => c_str!("I_SRCCHAN").into(),
            P_DESTTRACK => c_str!("P_DESTTRACK").into(),
            P_SRCTRACK => c_str!("P_SRCTRACK").into(),
            P_ENV(env_chunk_name) => {
                let cow: Cow<CStr> = env_chunk_name.into();
                concat_c_strs(c_str!("P_ENV:<"), cow.as_ref()).into()
            }
            P_EXT(extension_specific_key) => {
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key.as_ref()).into()
            }
            Custom(key) => key.into(),
        }
    }
}

/// Common envelope chunk names which you can pass to `TrackInfoKey::P_ENV()`.
///
/// The variants are named exactly like the strings which will be passed to the low-level REAPER
/// function because the medium-level API is designed to still be close to the raw REAPER API.  
///
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
pub enum EnvChunkName<'a> {
    VOLENV,
    PANENV,
    // TODO Figure out all common env chunk names
    // TODO Check if there are any *common* envelopes which don't have an own chunk. In this case
    //  we should provide a similar enum for get_track_envelope_by_name() envname as well.
    /// Use this for all non-common envelope names.
    Custom(Cow<'a, CStr>),
}

// TODO Maybe make this explicit and private because it exposes low-level logic.
impl<'a> From<EnvChunkName<'a>> for Cow<'a, CStr> {
    fn from(value: EnvChunkName<'a>) -> Self {
        use EnvChunkName::*;
        match value {
            VOLENV => c_str!("VOLENV").into(),
            PANENV => c_str!("PANENV").into(),
            Custom(name) => name,
        }
    }
}

fn concat_c_strs(first: &CStr, second: &CStr) -> CString {
    CString::new([first.to_bytes(), second.to_bytes()].concat()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_track_info_key() {
        use TrackInfoKey::*;
        assert_eq!(Cow::from(B_MUTE).as_ref(), c_str!("B_MUTE"));
        assert_eq!(
            Cow::from(P_ENV(EnvChunkName::VOLENV)).as_ref(),
            c_str!("P_ENV:<VOLENV")
        );
        assert_eq!(
            Cow::from(P_ENV(EnvChunkName::Custom(c_str!("MYENV").into()))).as_ref(),
            c_str!("P_ENV:<MYENV")
        );
        assert_eq!(
            Cow::from(TrackInfoKey::p_ext("SWS_FOO")).as_ref(),
            c_str!("P_EXT:SWS_FOO")
        );
        assert_eq!(
            Cow::from(TrackInfoKey::custom(c_str!("BLA"))).as_ref(),
            c_str!("BLA")
        );
    }
}
