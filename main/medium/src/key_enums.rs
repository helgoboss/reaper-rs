use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use std::borrow::Cow;
use std::ffi::CStr;

/// All the possible track info keys which you can pass to `Reaper::get_set_media_track_info()`.
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
#[derive(Clone, Debug)]
pub enum TrackInfoKey<'a> {
    FreeMode,
    HeightLock,
    MainSend,
    Mute,
    Phase,
    ShowInMixer,
    ShowInTcp,
    BeatAttachMode,
    MainSendOffs,
    DualPanL,
    DualPanR,
    Pan,
    PanLaw,
    PlayOffset,
    Vol,
    Width,
    McpFxSendScale,
    McpSendRgnScale,
    Guid,
    AutoMode,
    CustomColor,
    FolderCompact,
    FolderDepth,
    FxEn,
    HeightOverride,
    McpH,
    McpW,
    McpX,
    McpY,
    MidiHwOut,
    NChan,
    PanMode,
    PerfFlags,
    PlayOffsetFlag,
    RecArm,
    RecInput,
    RecMode,
    RecMon,
    RecMonItems,
    Selected,
    Solo,
    TcpH,
    TcpY,
    WndH,
    TrackNumber,
    Env(EnvChunkName<'a>),
    Ext(Cow<'a, CStr>),
    Icon,
    McpLayout,
    Name,
    ParTrack,
    Project,
    TcpLayout,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    Custom(Cow<'a, CStr>),
}

impl<'a> TrackInfoKey<'a> {
    pub fn ext(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::Ext(key.into().into_cow())
    }

    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::Custom(key.into().into_cow())
    }
}

impl<'a> From<TrackInfoKey<'a>> for Cow<'a, CStr> {
    fn from(value: TrackInfoKey<'a>) -> Self {
        use TrackInfoKey::*;
        match value {
            FreeMode => c_str!("B_FREEMODE").into(),
            HeightLock => c_str!("B_HEIGHTLOCK").into(),
            MainSend => c_str!("B_MAINSEND").into(),
            Mute => c_str!("B_MUTE").into(),
            Phase => c_str!("B_PHASE").into(),
            ShowInMixer => c_str!("B_SHOWINMIXER").into(),
            ShowInTcp => c_str!("B_SHOWINTCP").into(),
            BeatAttachMode => c_str!("C_BEATATTACHMODE").into(),
            MainSendOffs => c_str!("C_MAINSEND_OFFS").into(),
            DualPanL => c_str!("D_DUALPANL").into(),
            DualPanR => c_str!("D_DUALPANR").into(),
            Pan => c_str!("D_PAN").into(),
            PanLaw => c_str!("D_PANLAW").into(),
            PlayOffset => c_str!("D_PLAY_OFFSET").into(),
            Vol => c_str!("D_VOL").into(),
            Width => c_str!("D_WIDTH").into(),
            McpFxSendScale => c_str!("F_MCP_FXSEND_SCALE").into(),
            McpSendRgnScale => c_str!("F_MCP_SENDRGN_SCALE").into(),
            Guid => c_str!("GUID").into(),
            AutoMode => c_str!("I_AUTOMODE").into(),
            CustomColor => c_str!("I_CUSTOMCOLOR").into(),
            FolderCompact => c_str!("I_FOLDERCOMPACT").into(),
            FolderDepth => c_str!("I_FOLDERDEPTH").into(),
            FxEn => c_str!("I_FXEN").into(),
            HeightOverride => c_str!("I_HEIGHTOVERRIDE").into(),
            McpH => c_str!("I_MCPH").into(),
            McpW => c_str!("I_MCPW").into(),
            McpX => c_str!("I_MCPX").into(),
            McpY => c_str!("I_MCPY").into(),
            MidiHwOut => c_str!("I_MIDIHWOUT").into(),
            NChan => c_str!("I_NCHAN").into(),
            PanMode => c_str!("I_PANMODE").into(),
            PerfFlags => c_str!("I_PERFFLAGS").into(),
            PlayOffsetFlag => c_str!("I_PLAY_OFFSET_FLAG").into(),
            RecArm => c_str!("I_RECARM").into(),
            RecInput => c_str!("I_RECINPUT").into(),
            RecMode => c_str!("I_RECMODE").into(),
            RecMon => c_str!("I_RECMON").into(),
            RecMonItems => c_str!("I_RECMONITEMS").into(),
            Selected => c_str!("I_SELECTED").into(),
            Solo => c_str!("I_SOLO").into(),
            TcpH => c_str!("I_TCPH").into(),
            TcpY => c_str!("I_TCPY").into(),
            WndH => c_str!("I_WNDH").into(),
            TrackNumber => c_str!("IP_TRACKNUMBER").into(),
            Env(env_chunk_name) => {
                let cow: Cow<CStr> = env_chunk_name.into();
                concat_c_strs(c_str!("P_ENV:<"), cow.as_ref()).into()
            }
            Ext(extension_specific_key) => {
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key.as_ref()).into()
            }
            Icon => c_str!("P_ICON").into(),
            McpLayout => c_str!("P_MCP_LAYOUT").into(),
            Name => c_str!("P_NAME").into(),
            ParTrack => c_str!("P_PARTRACK").into(),
            Project => c_str!("P_PROJECT").into(),
            TcpLayout => c_str!("P_TCP_LAYOUT").into(),
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
#[derive(Clone, Debug)]
pub enum TrackSendInfoKey<'a> {
    Mono,
    Mute,
    Phase,
    Pan,
    PanLaw,
    Vol,
    AutoMode,
    DstChan,
    MidiFlags,
    SendMode,
    SrcChan,
    DestTrack,
    SrcTrack,
    Env(EnvChunkName<'a>),
    Ext(Cow<'a, CStr>),
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    Custom(Cow<'a, CStr>),
}

impl<'a> TrackSendInfoKey<'a> {
    pub fn p_ext(key: impl Into<ReaperStringArg<'a>>) -> TrackSendInfoKey<'a> {
        TrackSendInfoKey::Ext(key.into().into_cow())
    }

    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackSendInfoKey<'a> {
        TrackSendInfoKey::Custom(key.into().into_cow())
    }
}

impl<'a> From<TrackSendInfoKey<'a>> for Cow<'a, CStr> {
    fn from(value: TrackSendInfoKey<'a>) -> Self {
        use TrackSendInfoKey::*;
        match value {
            Mono => c_str!("B_MONO").into(),
            Mute => c_str!("B_MUTE").into(),
            Phase => c_str!("B_PHASE").into(),
            Pan => c_str!("D_PAN").into(),
            PanLaw => c_str!("D_PANLAW").into(),
            Vol => c_str!("D_VOL").into(),
            AutoMode => c_str!("I_AUTOMODE").into(),
            DstChan => c_str!("I_DSTCHAN").into(),
            MidiFlags => c_str!("I_MIDIFLAGS").into(),
            SendMode => c_str!("I_SENDMODE").into(),
            SrcChan => c_str!("I_SRCCHAN").into(),
            DestTrack => c_str!("P_DESTTRACK").into(),
            SrcTrack => c_str!("P_SRCTRACK").into(),
            Env(env_chunk_name) => {
                let cow: Cow<CStr> = env_chunk_name.into();
                concat_c_strs(c_str!("P_ENV:<"), cow.as_ref()).into()
            }
            Ext(key) => concat_c_strs(c_str!("P_EXT:"), key.as_ref()).into(),
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
#[derive(Clone, Debug)]
pub enum EnvChunkName<'a> {
    /// Volume (Pre-FX)
    VolEnv,
    /// Pan (Pre-FX)
    PanEnv,
    /// Volume
    VolEnv2,
    /// Pan
    PanEnv2,
    /// Width (Pre-FX)
    WidthEnv,
    /// Width
    WidthEnv2,
    /// Trim Volume
    VolEnv3,
    /// Mute
    MuteEnv,
    /// Use this for all non-common envelope names.
    Custom(Cow<'a, CStr>),
}

impl<'a> EnvChunkName<'a> {
    pub fn custom(name: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Custom(name.into().into_cow())
    }
}

impl<'a> From<EnvChunkName<'a>> for Cow<'a, CStr> {
    fn from(value: EnvChunkName<'a>) -> Self {
        use EnvChunkName::*;
        match value {
            VolEnv => c_str!("VOLENV").into(),
            PanEnv => c_str!("PANENV").into(),
            VolEnv2 => c_str!("VOLENV2").into(),
            PanEnv2 => c_str!("PANENV2").into(),
            WidthEnv => c_str!("WIDTHENV").into(),
            WidthEnv2 => c_str!("WIDTHENV2").into(),
            VolEnv3 => c_str!("VOLENV3").into(),
            MuteEnv => c_str!("MUTEENV").into(),
            Custom(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_track_info_key() {
        use TrackInfoKey::*;
        assert_eq!(Cow::from(Mute).as_ref(), c_str!("B_MUTE"));
        assert_eq!(
            Cow::from(Env(EnvChunkName::VolEnv)).as_ref(),
            c_str!("P_ENV:<VOLENV")
        );
        assert_eq!(
            Cow::from(Env(EnvChunkName::Custom(c_str!("MYENV").into()))).as_ref(),
            c_str!("P_ENV:<MYENV")
        );
        assert_eq!(
            Cow::from(TrackInfoKey::ext("SWS_FOO")).as_ref(),
            c_str!("P_EXT:SWS_FOO")
        );
        assert_eq!(
            Cow::from(TrackInfoKey::custom(c_str!("BLA"))).as_ref(),
            c_str!("BLA")
        );
    }
}
