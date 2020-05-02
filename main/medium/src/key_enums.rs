use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use reaper_rs_low::raw;
use reaper_rs_low::raw::GUID;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

/// Track info key which you can pass to [`get_set_media_track_info()`].
///
/// [`get_set_media_track_info()`]: struct.ReaperFunctions.html#method.get_set_media_track_info
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackInfoKey<'a> {
    /// Parent track (read-only).
    ParTrack,
    /// Parent project (read-only).
    Project,
    /// Track name (on master returns `null`).
    Name,
    /// Track icon.
    ///
    /// Full file name or relative to resource path / data / track icons.
    Icon,
    /// Layout name.
    McpLayout,
    /// Layout name.
    TcpLayout,
    /// Extension-specific persistent data.
    Ext(Cow<'a, CStr>),
    /// 6-byte GUID, can query or update.
    ///
    /// If using a `_string()` function, GUID is a string `{xyz-...}`.
    Guid,
    /// Muted.
    Mute,
    /// Track phase inverted.
    Phase,
    /// Track number
    ///
    /// 1-based, read-only, returns the i32 directly.
    ///
    /// - 0 → not found
    /// - -1 → master track
    TrackNumber,
    /// Soloed.
    ///
    /// - 0 → not soloed
    /// - 1 → soloed
    /// - 2 → soloed in place
    /// - 5 → safe soloed
    /// - 6 → safe soloed in place
    Solo,
    /// FX enabled.
    ///
    /// - 0 → bypassed
    /// - != 0 → FX active
    FxEn,
    /// Record armed.
    ///
    /// - 0 → not record armed
    /// - 1 → record armed
    RecArm,
    /// Record input.
    ///
    /// - <0 → no input
    /// - 0..=n → mono hardware input
    /// - 512 + n → rearoute input
    /// - &1024 → stereo input pair
    /// - &4096 → MIDI input, if set then low 5 bits represent channel (0 → all, 1 - 16 → only
    ///   channel), next 6 bits represent physical input (63 → all, 62 → VKB)
    RecInput,
    /// Record mode.
    ///
    /// - 0 → input
    /// - 1 → stereo out
    /// - 2 → none
    /// - 3 → stereo out with latency compensation
    /// - 4 → midi output
    /// - 5 → mono out
    /// - 6 → mono out with latency compensation
    /// - 7 → MIDI overdub
    /// - 8 → MIDI replace
    RecMode,
    /// Record monitoring.
    ///
    /// - 0 → off
    /// - 1 → normal
    /// - 2 → not when playing (tape style)
    RecMon,
    /// Monitor items while recording.
    ///
    /// - 0 → off
    /// - 1 → on
    RecMonItems,
    /// Track automation mode.
    ///
    /// - 0 → trim/off
    /// - 1 → read
    /// - 2 → touch
    /// - 3 → write
    /// - 4 → latch
    AutoMode,
    /// Number of track channels.
    ///
    /// 2 - 64, even numbers only.
    Nchan,
    /// Track selected.
    ///
    /// - 0 → unselected
    /// - 1 → selected
    Selected,
    /// Current TCP window height in pixels including envelopes (read-only).
    WndH,
    /// Current TCP window height in pixels not including envelopes (read-only).
    TcpH,
    /// Current TCP window Y-position in pixels relative to top of arrange view (read-only).
    TcpY,
    /// Current MCP X-position in pixels relative to mixer container.
    McpX,
    /// Current MCP Y-position in pixels relative to mixer container.
    McpY,
    /// Current MCP width in pixels.
    McpW,
    /// Current MCP height in pixels.
    McpH,
    /// Folder depth change.
    ///
    /// - 0 → normal
    /// - 1 → track is a folder parent
    /// - -1 → track is the last in the innermost folder
    /// - -2 → track is the last in the innermost and next-innermost folders
    /// - ...
    FolderDepth,
    /// Folder compacted state (only valid on folders).
    ///
    /// - 0 → normal
    /// - 1 → small
    /// - 2 → tiny children
    FolderCompact,
    /// Track midi hardware output index.
    ///
    /// Low 5 bits are which channels (1..=16, 0 → all), next 5 bits are output device index
    /// (0..=31). < 0 means disabled.
    MidiHwOut,
    /// Track performance flags.
    ///
    /// &1 → no media buffering
    /// &2 → no anticipative FX
    PerfFlags,
    /// Custom color.
    ///
    /// `<OS dependent color> | 0x100000` (i.e. `ColorToNative(r, g, b) | 0x100000`).
    /// If you don't do `| 0x100000`, then it will not be used, but will store the color anyway.
    CustomColor,
    /// Custom height override for TCP window.
    ///
    /// 0 for none, otherwise size in pixels.
    HeightOverride,
    /// Track height lock.
    ///
    /// Must set [`HeightOverride`] before locking.
    ///
    /// [`HeightOverride`]: #variant.HeightOverride
    HeightLock,
    /// Trim volume of track.
    ///
    /// - 0 → -inf
    /// - 0.5 → -6dB
    /// - 1 → +0dB
    /// - 2 → +6dB
    /// - ...
    Vol,
    /// Trim pan of track
    ///
    /// -1..=1.
    Pan,
    /// Width of track
    ///
    /// -1..=1.
    Width,
    /// Dual pan position 1.
    ///
    /// -1..=1, only if [`PanMode`] == 6.
    ///
    /// [`PanMode`]: #variant.PanMode
    DualPanL,
    /// Dual pan position 2.
    ///
    /// -1..=1, only if [`PanMode`] == 6.
    ///
    /// [`PanMode`]: #variant.PanMode
    DualPanR,
    /// Pan mode.
    ///
    /// - 0 → classic 3.x
    /// - 3 → new balance
    /// - 5 → stereo pan
    /// - 6 → dual pan
    PanMode,
    /// Pan law.
    ///
    /// - < 0 → project default
    /// - 1 → +0 dB
    /// - ...
    PanLaw,
    /// TrackEnvelope (read only).
    Env(EnvChunkName<'a>),
    /// Track control panel visible in mixer.
    ///
    /// Do not use on master track.
    ShowInMixer,
    /// Track control panel visible in arrange view.
    ///
    /// Do not use on master track.
    ShowInTcp,
    /// Track sends audio to parent.
    MainSend,
    /// Channel offset of track send to parent.
    MainSendOffs,
    /// Track free item positioning enabled
    ///
    /// Call [`update_timeline`] after changing.
    ///
    /// [`update_timeline`]: struct.Reaper.html#method.update_timeline
    FreeMode,
    /// Track timebase.
    ///
    /// - -1 → project default
    /// - 0 → time
    /// - 1 → beats (position, length, rate)
    /// - 2 → beats (position only)
    BeatAttachMode,
    /// Scale of FX and send area in MCP.
    ///
    /// - 0 → minimum allowed
    /// - 1 → maximum allowed
    McpFxSendScale,
    /// Scale of send area as proportion of the FX and send total area.
    ///
    /// - 0 → minimum allowed
    /// - 1 → maximum allowed
    McpSendRgnScale,
    /// Track playback offset state.
    ///
    /// - &1 → bypassed
    /// - &2 → offset
    ///
    /// Value is measured in samples (otherwise measured in seconds).
    PlayOffsetFlag,
    /// Track playback offset.
    ///
    /// Units depend on [`PlayOffsetFlag`].
    ///
    /// [`PlayOffsetFlag`]: #variant.PlayOffsetFlag
    PlayOffset,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    Custom(Cow<'a, CStr>),
}

impl<'a> TrackInfoKey<'a> {
    /// Convenience method for creating an [`Ext`] key.
    ///
    /// [`Ext`]: #variant.Ext
    pub fn ext(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::Ext(key.into().into_inner())
    }

    /// Convenience method for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackInfoKey<'a> {
        TrackInfoKey::Custom(key.into().into_inner())
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
            Nchan => c_str!("I_NCHAN").into(),
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

/// Track info key which you can pass to [`get_set_track_send_info()`].
///
/// [`get_set_track_send_info()`]: struct.ReaperFunctions.html#method.get_set_track_send_info
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendInfoKey<'a> {
    /// Returns the destination track (read-only).
    ///
    /// Only applies for sends/receives.
    DestTrack,
    /// Returns the source track (read-only).
    ///
    /// Only applies for sends/receives.
    SrcTrack,
    /// Returns the corresponding track send envelope.
    Env(EnvChunkName<'a>),
    /// Extension-specific persistent data.
    Ext(Cow<'a, CStr>),
    Mute,
    /// `true` to flip phase.
    Phase,
    Mono,
    /// 1.0 → +0 dB etc.
    Vol,
    /// -1..=1
    Pan,
    ///
    /// - 1.0 → +0.0 dB
    /// - 0.5 → -6 dB
    /// - -1.0 → value defined in project
    PanLaw,
    ///
    /// - 0 → post-fader
    /// - 1 → pre-fx
    /// - 2 → post-fx (deprecated)
    /// - 3 → post-fx
    SendMode,
    /// Automation mode.
    ///
    /// - -1 → use track automation mode
    /// - 0 → trim/off
    /// - 1 → read
    /// - 2 → touch
    /// - 3 → write
    /// - 4 → latch
    AutoMode,
    /// Index, &1024 → mono, -1 → none
    SrcChan,
    /// Index, &1024 → mono, otherwise stereo pair, hwout: &512 → rearoute
    DstChan,
    /// 
    /// - Low 5 bits → source channel (0 → all, 1..=16)
    /// - Next 5 bits → destination channel (0 → original, 1..=16)
    MidiFlags,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    Custom(Cow<'a, CStr>),
}

impl<'a> TrackSendInfoKey<'a> {
    /// Convenience method for creating an [`Ext`] key.
    ///
    /// [`Ext`]: #variant.Ext
    pub fn p_ext(key: impl Into<ReaperStringArg<'a>>) -> TrackSendInfoKey<'a> {
        TrackSendInfoKey::Ext(key.into().into_inner())
    }

    /// Convenience method for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackSendInfoKey<'a> {
        TrackSendInfoKey::Custom(key.into().into_inner())
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

/// Envelope chunk name which you can pass e.g. to [`TrackInfoKey::Env()`].
///
/// [`TrackInfoKey::Env()`]: enum.TrackInfoKey.html#variant.Env
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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
    /// Convenience method for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(name: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Custom(name.into().into_inner())
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
