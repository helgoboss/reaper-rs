#![allow(non_camel_case_types)]
use c_str_macro::c_str;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

pub type HookCommand = extern "C" fn(command_index: i32, _flag: i32) -> bool;
pub type ToggleAction = extern "C" fn(command_index: i32) -> i32;
pub type HookPostCommand = extern "C" fn(command_id: u32, _flag: i32);

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

pub enum RegInstr {
    Register(ExtensionType),
    Unregister(ExtensionType),
}

impl From<RegInstr> for Cow<'static, CStr> {
    fn from(value: RegInstr) -> Self {
        use RegInstr::*;
        match value {
            Register(et) => et.into(),
            Unregister(et) => concat_c_strs(c_str!("-"), Cow::from(et).as_ref()).into(),
        }
    }
}

pub enum ExtensionType {
    Api(&'static CStr),
    ApiDef(&'static CStr),
    HookCommand,
    HookPostCommand,
    HookCommand2,
    ToggleAction,
    ActionHelp,
    CommandId,
    CommandIdLookup,
    GAccel,
    CSurfInst,
    Custom(&'static CStr),
}

impl From<ExtensionType> for Cow<'static, CStr> {
    fn from(value: ExtensionType) -> Self {
        use ExtensionType::*;
        match value {
            GAccel => c_str!("gaccel").into(),
            CSurfInst => c_str!("csurf_inst").into(),
            Api(func_name) => concat_c_strs(c_str!("API_"), func_name).into(),
            ApiDef(func_def) => concat_c_strs(c_str!("APIdef_"), func_def).into(),
            HookCommand => c_str!("hookcommand").into(),
            HookPostCommand => c_str!("hookpostcommand").into(),
            HookCommand2 => c_str!("hookcommand2").into(),
            ToggleAction => c_str!("toggleaction").into(),
            ActionHelp => c_str!("action_help").into(),
            CommandId => c_str!("command_id").into(),
            CommandIdLookup => c_str!("command_id_lookup").into(),
            Custom(k) => k.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TrackNumberResult {
    MasterTrack,
    NotFound,
    // TODO Maybe use non-zero (because it's one-rooted)
    TrackNumber(u32),
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
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
pub enum ReaperPointerType {
    MediaTrack,
    ReaProject,
    MediaItem,
    MediaItem_Take,
    TrackEnvelope,
    PCM_source,
    /// If a variant is missing in this enum, you can use this custom one as a last resort. Don't
    /// include the trailing asterisk (`*`)! It will be added to the call automatically.
    Custom(&'static CStr),
}

impl From<ReaperPointerType> for Cow<'static, CStr> {
    fn from(value: ReaperPointerType) -> Self {
        use ReaperPointerType::*;
        match value {
            MediaTrack => c_str!("MediaTrack*").into(),
            ReaProject => c_str!("ReaProject*").into(),
            MediaItem => c_str!("MediaItem*").into(),
            MediaItem_Take => c_str!("MediaItem_Take*").into(),
            TrackEnvelope => c_str!("TrackEnvelope*").into(),
            PCM_source => c_str!("PCM_source*").into(),
            Custom(name) => concat_c_strs(name, c_str!("*")).into(),
        }
    }
}

/// All the possible track info keys which you can pass to `Reaper::get_set_media_track_info()`.
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
/// The variants are named exactly like the strings which will be passed to the low-level REAPER
/// function because the medium-level API is designed to still be close to the raw REAPER API.  
pub enum TrackInfoKey {
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
    P_ENV(EnvChunkName),
    P_EXT(&'static CStr),
    P_ICON,
    P_MCP_LAYOUT,
    P_NAME,
    P_PARTRACK,
    P_PROJECT,
    P_TCP_LAYOUT,
    /// If a variant is missing in this enum, you can use this custom one as a last resort.
    Custom(&'static CStr),
}

impl From<TrackInfoKey> for Cow<'static, CStr> {
    fn from(value: TrackInfoKey) -> Self {
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
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key).into()
            }
            P_ICON => c_str!("P_ICON").into(),
            P_MCP_LAYOUT => c_str!("P_MCP_LAYOUT").into(),
            P_NAME => c_str!("P_NAME").into(),
            P_PARTRACK => c_str!("P_PARTRACK").into(),
            P_PROJECT => c_str!("P_PROJECT").into(),
            P_TCP_LAYOUT => c_str!("P_TCP_LAYOUT").into(),
            Custom(key) => key.into(),
        }
    }
}

/// All the possible track send info keys which you can pass to `Reaper::get_set_track_send_info()`.
///
/// The variants are named exactly like the strings which will be passed to the low-level REAPER
/// function because the medium-level API is designed to still be close to the raw REAPER API.  
///
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
pub enum TrackSendInfoKey {
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
    P_ENV(EnvChunkName),
    P_EXT(&'static CStr),
    /// If a variant is missing in this enum, you can use this custom one as a last resort.
    Custom(&'static CStr),
}

impl From<TrackSendInfoKey> for Cow<'static, CStr> {
    fn from(value: TrackSendInfoKey) -> Self {
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
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key).into()
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
pub enum EnvChunkName {
    VOLENV,
    PANENV,
    /// Use this for all non-common envelope names.
    Custom(&'static CStr),
}

impl From<EnvChunkName> for Cow<'static, CStr> {
    fn from(value: EnvChunkName) -> Self {
        use EnvChunkName::*;
        match value {
            VOLENV => c_str!("VOLENV").into(),
            PANENV => c_str!("PANENV").into(),
            Custom(name) => name.into(),
        }
    }
}

fn concat_c_strs(first: &CStr, second: &CStr) -> CString {
    CString::new([first.to_bytes(), second.to_bytes()].concat()).unwrap()
}

#[cfg(test)]
mod test {
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
            Cow::from(P_ENV(EnvChunkName::Custom(c_str!("MYENV")))).as_ref(),
            c_str!("P_ENV:<MYENV")
        );
        assert_eq!(
            Cow::from(P_EXT(c_str!("SWS_FOO"))).as_ref(),
            c_str!("P_EXT:SWS_FOO")
        );
        assert_eq!(Cow::from(Custom(c_str!("BLA"))).as_ref(), c_str!("BLA"));
    }
}
