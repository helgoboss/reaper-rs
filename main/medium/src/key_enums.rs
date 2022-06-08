use crate::{concat_reaper_strs, ReaperStr, ReaperStringArg};

use std::borrow::Cow;

/// Track attribute key which you can pass to [`get_set_media_track_info()`].
///
/// [`get_set_media_track_info()`]: struct.Reaper.html#method.get_set_media_track_info
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackAttributeKey<'a> {
    /// Parent track (read-only).
    ///
    /// `*mut MediaTrack`
    ParTrack,
    /// Parent project (read-only).
    ///
    /// `*mut ReaProject`
    Project,
    /// Track name (on master returns `null_mut()`).
    ///
    /// `*mut char`
    Name,
    /// Track icon.
    ///
    /// `*const char`
    ///
    /// Full file name or relative to resource path / data / track icons.
    Icon,
    /// Layout name.
    ///
    /// `*const char`
    McpLayout,
    /// Layout name.
    ///
    /// `*const char`
    TcpLayout,
    /// Extension-specific persistent data.
    ///
    /// `*mut char`
    ///
    /// Use [`ext()`] to create this variant.
    ///
    /// [`ext()`]: #method.ext
    Ext(Cow<'a, ReaperStr>),
    /// 6-byte GUID, can query or update.
    ///
    /// `*mut GUID`
    ///
    /// If using a `_string()` function, GUID is a string `{xyz-...}`.
    Guid,
    /// Muted.
    ///
    /// `*mut bool`
    Mute,
    /// Track phase inverted.
    ///
    /// `*mut bool`
    Phase,
    /// Track number
    ///
    /// `i32`
    ///
    /// 1-based, read-only, returns the i32 directly.
    ///
    /// - 0 → not found
    /// - -1 → master track
    TrackNumber,
    /// Soloed.
    ///
    /// `*mut i32`
    ///
    /// - 0 → not soloed
    /// - 1 → soloed
    /// - 2 → soloed in place
    /// - 5 → safe soloed
    /// - 6 → safe soloed in place
    Solo,
    /// FX enabled.
    ///
    /// `*mut i32`
    ///
    /// - 0 → bypassed
    /// - != 0 → FX active
    FxEn,
    /// Record armed.
    ///
    /// `*mut i32`
    ///
    /// - 0 → not record armed
    /// - 1 → record armed
    RecArm,
    /// Record input.
    ///
    /// `*mut i32`
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
    /// `*mut i32`
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
    /// `*mut i32`
    ///
    /// - 0 → off
    /// - 1 → normal
    /// - 2 → not when playing (tape style)
    RecMon,
    /// Monitor items while recording.
    ///
    /// `*mut i32`
    ///
    /// - 0 → off
    /// - 1 → on
    RecMonItems,
    /// Track automation mode.
    ///
    /// `*mut i32`
    ///
    /// - 0 → trim/off
    /// - 1 → read
    /// - 2 → touch
    /// - 3 → write
    /// - 4 → latch
    AutoMode,
    /// Number of track channels.
    ///
    /// `*mut i32`
    ///
    /// 2 - 64, even numbers only.
    Nchan,
    /// Track vu mode.
    ///  
    /// `*mut i32`
    /// - 0 → Stereo Peaks
    /// - 2 → Multichannel Peaks
    /// - 4 → Stereo RMS
    /// - 8 → Combined RMS
    /// - 12 → LUFS-M
    /// - 16 → LUFS-S (readout = Max)
    /// - 20 → LUFS-S (readout = Current)
    ///
    /// LUFS calculation on channels 1+2 only.
    VuMode,
    /// Track selected.
    ///
    /// `*mut i32`
    ///
    /// - 0 → unselected
    /// - 1 → selected
    Selected,
    /// Current TCP window height in pixels including envelopes (read-only).
    ///
    /// `*mut i32`
    WndH,
    /// Current TCP window height in pixels not including envelopes (read-only).
    ///
    /// `*mut i32`
    TcpH,
    /// Current TCP window Y-position in pixels relative to top of arrange view (read-only).
    ///
    /// `*mut i32`
    TcpY,
    /// Current MCP X-position in pixels relative to mixer container.
    ///
    /// `*mut i32`
    McpX,
    /// Current MCP Y-position in pixels relative to mixer container.
    ///
    /// `*mut i32`
    McpY,
    /// Current MCP width in pixels.
    ///
    /// `*mut i32`
    McpW,
    /// Current MCP height in pixels.
    ///
    /// `*mut i32`
    McpH,
    /// Folder depth change.
    ///
    /// `*mut i32`
    ///
    /// - 0 → normal
    /// - 1 → track is a folder parent
    /// - -1 → track is the last in the innermost folder
    /// - -2 → track is the last in the innermost and next-innermost folders
    /// - ...
    FolderDepth,
    /// Folder compacted state (only valid on folders).
    ///
    /// `*mut i32`
    ///
    /// - 0 → normal
    /// - 1 → small
    /// - 2 → tiny children
    FolderCompact,
    /// Track midi hardware output index.
    ///
    /// `*mut i32`
    ///
    /// Low 5 bits are which channels (1..=16, 0 → all), next 5 bits are output device index
    /// (0..=31). < 0 means disabled.
    MidiHwOut,
    /// Track performance flags.
    ///
    /// `*mut i32`
    ///
    /// &1 → no media buffering
    /// &2 → no anticipative FX
    PerfFlags,
    /// Custom color.
    ///
    /// `*mut i32`
    ///
    /// `<OS dependent color> | 0x100000` (i.e. `ColorToNative(r, g, b) | 0x100000`).
    /// If you don't do `| 0x100000`, then it will not be used, but will store the color anyway.
    CustomColor,
    /// Custom height override for TCP window.
    ///
    /// `*mut i32`
    ///
    /// 0 for none, otherwise size in pixels.
    HeightOverride,
    /// Track height lock.
    ///
    /// `*mut bool`
    ///
    /// Must set [`HeightOverride`] before locking.
    ///
    /// [`HeightOverride`]: #variant.HeightOverride
    HeightLock,
    /// Trim volume of track.
    ///
    /// `*mut f64`
    ///
    /// - 0 → -inf
    /// - 0.5 → -6dB
    /// - 1 → +0dB
    /// - 2 → +6dB
    /// - ...
    Vol,
    /// Trim pan of track
    ///
    /// `*mut f64`
    ///
    /// -1..=1.
    Pan,
    /// Width of track
    ///
    /// `*mut f64`
    ///
    /// -1..=1.
    Width,
    /// Dual pan position 1.
    ///
    /// `*mut f64`
    ///
    /// -1..=1, only if [`PanMode`] == 6.
    ///
    /// [`PanMode`]: #variant.PanMode
    DualPanL,
    /// Dual pan position 2.
    ///
    /// `*mut f64`
    ///
    /// -1..=1, only if [`PanMode`] == 6.
    ///
    /// [`PanMode`]: #variant.PanMode
    DualPanR,
    /// Pan mode.
    ///
    /// `*mut i32`
    ///
    /// - 0 → classic 3.x
    /// - 3 → new balance
    /// - 5 → stereo pan
    /// - 6 → dual pan
    PanMode,
    /// Pan law.
    ///
    /// `*mut f64`
    ///
    /// - < 0 → project default
    /// - 1 → +0 dB
    /// - ...
    PanLaw,
    /// TrackEnvelope (read only).
    ///
    /// `*mut TrackEnvelope`
    Env(EnvChunkName<'a>),
    /// Track control panel visible in mixer.
    ///
    /// `*mut bool`
    ///
    /// Do not use on master track.
    ShowInMixer,
    /// Track control panel visible in arrange view.
    ///
    /// `*mut bool`
    ///
    /// Do not use on master track.
    ShowInTcp,
    /// Track sends audio to parent.
    ///
    /// `*mut bool`
    MainSend,
    /// Channel offset of track send to parent.
    ///
    /// `*mut char`
    MainSendOffs,
    /// Track free item positioning enabled
    ///
    /// `*mut bool`
    ///
    /// Call [`update_timeline`] after changing.
    ///
    /// [`update_timeline`]: struct.ReaperSession.html#method.update_timeline
    FreeMode,
    /// Track timebase.
    ///
    /// `*mut char`
    ///
    /// - -1 → project default
    /// - 0 → time
    /// - 1 → beats (position, length, rate)
    /// - 2 → beats (position only)
    BeatAttachMode,
    /// Scale of FX and send area in MCP.
    ///
    /// `*mut f32`
    ///
    /// - 0 → minimum allowed
    /// - 1 → maximum allowed
    McpFxSendScale,
    /// Scale of send area as proportion of the FX and send total area.
    ///
    /// `*mut f32`
    ///
    /// - 0 → minimum allowed
    /// - 1 → maximum allowed
    McpSendRgnScale,
    /// Track playback offset state.
    ///
    /// `*mut i32`
    ///
    /// - &1 → bypassed
    /// - &2 → offset
    ///
    /// Value is measured in samples (otherwise measured in seconds).
    PlayOffsetFlag,
    /// Track playback offset.
    ///
    /// `*mut f64`
    ///
    /// Units depend on [`PlayOffsetFlag`].
    ///
    /// [`PlayOffsetFlag`]: #variant.PlayOffsetFlag
    PlayOffset,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom(Cow<'a, ReaperStr>),
}

impl<'a> TrackAttributeKey<'a> {
    /// Convenience function for creating an [`Ext`] key.
    ///
    /// [`Ext`]: #variant.Ext
    pub fn ext(key: impl Into<ReaperStringArg<'a>>) -> TrackAttributeKey<'a> {
        TrackAttributeKey::Ext(key.into().into_inner())
    }

    /// Convenience function for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackAttributeKey<'a> {
        TrackAttributeKey::Custom(key.into().into_inner())
    }

    pub(crate) fn into_raw(self) -> Cow<'a, ReaperStr> {
        use TrackAttributeKey::*;
        match self {
            FreeMode => reaper_str!("B_FREEMODE").into(),
            HeightLock => reaper_str!("B_HEIGHTLOCK").into(),
            MainSend => reaper_str!("B_MAINSEND").into(),
            Mute => reaper_str!("B_MUTE").into(),
            Phase => reaper_str!("B_PHASE").into(),
            ShowInMixer => reaper_str!("B_SHOWINMIXER").into(),
            ShowInTcp => reaper_str!("B_SHOWINTCP").into(),
            BeatAttachMode => reaper_str!("C_BEATATTACHMODE").into(),
            MainSendOffs => reaper_str!("C_MAINSEND_OFFS").into(),
            DualPanL => reaper_str!("D_DUALPANL").into(),
            DualPanR => reaper_str!("D_DUALPANR").into(),
            Pan => reaper_str!("D_PAN").into(),
            PanLaw => reaper_str!("D_PANLAW").into(),
            PlayOffset => reaper_str!("D_PLAY_OFFSET").into(),
            Vol => reaper_str!("D_VOL").into(),
            Width => reaper_str!("D_WIDTH").into(),
            McpFxSendScale => reaper_str!("F_MCP_FXSEND_SCALE").into(),
            McpSendRgnScale => reaper_str!("F_MCP_SENDRGN_SCALE").into(),
            Guid => reaper_str!("GUID").into(),
            AutoMode => reaper_str!("I_AUTOMODE").into(),
            CustomColor => reaper_str!("I_CUSTOMCOLOR").into(),
            FolderCompact => reaper_str!("I_FOLDERCOMPACT").into(),
            FolderDepth => reaper_str!("I_FOLDERDEPTH").into(),
            FxEn => reaper_str!("I_FXEN").into(),
            HeightOverride => reaper_str!("I_HEIGHTOVERRIDE").into(),
            McpH => reaper_str!("I_MCPH").into(),
            McpW => reaper_str!("I_MCPW").into(),
            McpX => reaper_str!("I_MCPX").into(),
            McpY => reaper_str!("I_MCPY").into(),
            MidiHwOut => reaper_str!("I_MIDIHWOUT").into(),
            Nchan => reaper_str!("I_NCHAN").into(),
            VuMode => reaper_str!("I_VUMODE").into(),
            PanMode => reaper_str!("I_PANMODE").into(),
            PerfFlags => reaper_str!("I_PERFFLAGS").into(),
            PlayOffsetFlag => reaper_str!("I_PLAY_OFFSET_FLAG").into(),
            RecArm => reaper_str!("I_RECARM").into(),
            RecInput => reaper_str!("I_RECINPUT").into(),
            RecMode => reaper_str!("I_RECMODE").into(),
            RecMon => reaper_str!("I_RECMON").into(),
            RecMonItems => reaper_str!("I_RECMONITEMS").into(),
            Selected => reaper_str!("I_SELECTED").into(),
            Solo => reaper_str!("I_SOLO").into(),
            TcpH => reaper_str!("I_TCPH").into(),
            TcpY => reaper_str!("I_TCPY").into(),
            WndH => reaper_str!("I_WNDH").into(),
            TrackNumber => reaper_str!("IP_TRACKNUMBER").into(),
            Env(env_chunk_name) => {
                concat_reaper_strs(reaper_str!("P_ENV:<"), env_chunk_name.into_raw().as_ref())
                    .into()
            }
            Ext(extension_specific_key) => {
                concat_reaper_strs(reaper_str!("P_EXT:"), extension_specific_key.as_ref()).into()
            }
            Icon => reaper_str!("P_ICON").into(),
            McpLayout => reaper_str!("P_MCP_LAYOUT").into(),
            Name => reaper_str!("P_NAME").into(),
            ParTrack => reaper_str!("P_PARTRACK").into(),
            Project => reaper_str!("P_PROJECT").into(),
            TcpLayout => reaper_str!("P_TCP_LAYOUT").into(),
            Custom(key) => key,
        }
    }
}

/// Take attribute key which you can pass to [`get_set_media_item_take_info()`].
///
/// [`get_set_media_item_take_info()`]: struct.Reaper.html#method.get_set_media_item_take_info
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TakeAttributeKey<'a> {
    /// Start offset in source media in seconds.
    StartOffs,
    /// Current source.
    ///
    /// Note that if setting this, you should first retrieve the old source, set the new, *then*
    /// delete the old.
    ///
    /// `*mut PCM_source`
    Source,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom(Cow<'a, ReaperStr>),
}

impl<'a> TakeAttributeKey<'a> {
    /// Convenience function for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TakeAttributeKey<'a> {
        TakeAttributeKey::Custom(key.into().into_inner())
    }

    pub(crate) fn into_raw(self) -> Cow<'a, ReaperStr> {
        use TakeAttributeKey::*;
        match self {
            Source => reaper_str!("P_SOURCE").into(),
            StartOffs => reaper_str!("D_STARTOFFS").into(),
            Custom(key) => key,
        }
    }
}

/// Track send attribute key which you can pass to [`get_set_track_send_info()`].
///
/// [`get_set_track_send_info()`]: struct.Reaper.html#method.get_set_track_send_info
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendAttributeKey<'a> {
    /// Destination track (read-only).
    ///
    /// `*mut MediaTrack`
    ///
    /// Only applies for sends/receives.
    DestTrack,
    /// Source track (read-only).
    ///
    /// `*mut MediaTrack`
    ///
    /// Only applies for sends/receives.
    SrcTrack,
    /// Corresponding track send envelope.
    ///
    /// `*mut TrackEnvelope`
    Env(EnvChunkName<'a>),
    /// Extension-specific persistent data.
    ///
    /// `*mut char`
    ///
    /// Use [`ext()`] to create this variant.
    ///
    /// [`ext()`]: #method.ext
    Ext(Cow<'a, ReaperStr>),
    /// Muted.
    ///
    /// `*mut bool`
    Mute,
    /// Phase.
    ///
    /// `*mut bool`
    ///
    /// `true` to flip phase.
    Phase,
    /// Mono.
    ///
    /// `*mut bool`
    Mono,
    /// Volume.
    ///
    /// `*mut f64`
    ///
    /// 1.0 → +0 dB etc.
    Vol,
    /// Pan.
    ///
    /// `*mut f64`
    ///
    /// -1..=1
    Pan,
    /// Pan law.
    ///
    /// `*mut f64`
    ///
    /// - 1.0 → +0.0 dB
    /// - 0.5 → -6 dB
    /// - -1.0 → value defined in project
    PanLaw,
    /// Send mode.
    ///
    /// `*mut i32`
    ///
    /// - 0 → post-fader
    /// - 1 → pre-fx
    /// - 2 → post-fx (deprecated)
    /// - 3 → post-fx
    SendMode,
    /// Automation mode.
    ///
    /// `*mut i32`
    ///
    /// - -1 → use track automation mode
    /// - 0 → trim/off
    /// - 1 → read
    /// - 2 → touch
    /// - 3 → write
    /// - 4 → latch
    AutoMode,
    /// Source channel.
    ///
    /// `*mut i32`
    ///
    /// Index, &1024 → mono, -1 → none
    SrcChan,
    /// Destination channel.
    ///
    /// `*mut i32`
    /// Index, &1024 → mono, otherwise stereo pair, hwout: &512 → rearoute
    DstChan,
    /// MIDI flags.
    ///
    /// `*mut i32`
    ///
    /// - Low 5 bits → source channel (0 → all, 1..=16)
    /// - Next 5 bits → destination channel (0 → original, 1..=16)
    MidiFlags,
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom(Cow<'a, ReaperStr>),
}

impl<'a> TrackSendAttributeKey<'a> {
    /// Convenience function for creating an [`Ext`] key.
    ///
    /// [`Ext`]: #variant.Ext
    pub fn ext(key: impl Into<ReaperStringArg<'a>>) -> TrackSendAttributeKey<'a> {
        TrackSendAttributeKey::Ext(key.into().into_inner())
    }

    /// Convenience function for creating a [`Custom`] key.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> TrackSendAttributeKey<'a> {
        TrackSendAttributeKey::Custom(key.into().into_inner())
    }

    pub(crate) fn into_raw(self) -> Cow<'a, ReaperStr> {
        use TrackSendAttributeKey::*;
        match self {
            Mono => reaper_str!("B_MONO").into(),
            Mute => reaper_str!("B_MUTE").into(),
            Phase => reaper_str!("B_PHASE").into(),
            Pan => reaper_str!("D_PAN").into(),
            PanLaw => reaper_str!("D_PANLAW").into(),
            Vol => reaper_str!("D_VOL").into(),
            AutoMode => reaper_str!("I_AUTOMODE").into(),
            DstChan => reaper_str!("I_DSTCHAN").into(),
            MidiFlags => reaper_str!("I_MIDIFLAGS").into(),
            SendMode => reaper_str!("I_SENDMODE").into(),
            SrcChan => reaper_str!("I_SRCCHAN").into(),
            DestTrack => reaper_str!("P_DESTTRACK").into(),
            SrcTrack => reaper_str!("P_SRCTRACK").into(),
            Env(env_chunk_name) => {
                concat_reaper_strs(reaper_str!("P_ENV:<"), env_chunk_name.into_raw().as_ref())
                    .into()
            }
            Ext(key) => concat_reaper_strs(reaper_str!("P_EXT:"), key.as_ref()).into(),
            Custom(key) => key,
        }
    }
}

/// Envelope chunk name which you can pass e.g. to [`TrackAttributeKey::Env()`].
///
/// [`TrackAttributeKey::Env()`]: enum.TrackAttributeKey.html#variant.Env
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
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom(Cow<'a, ReaperStr>),
}

impl<'a> EnvChunkName<'a> {
    /// Convenience function for creating a [`Custom`] name.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(name: impl Into<ReaperStringArg<'a>>) -> EnvChunkName<'a> {
        EnvChunkName::Custom(name.into().into_inner())
    }

    pub(crate) fn into_raw(self) -> Cow<'a, ReaperStr> {
        use EnvChunkName::*;
        match self {
            VolEnv => reaper_str!("VOLENV").into(),
            PanEnv => reaper_str!("PANENV").into(),
            VolEnv2 => reaper_str!("VOLENV2").into(),
            PanEnv2 => reaper_str!("PANENV2").into(),
            WidthEnv => reaper_str!("WIDTHENV").into(),
            WidthEnv2 => reaper_str!("WIDTHENV2").into(),
            VolEnv3 => reaper_str!("VOLENV3").into(),
            MuteEnv => reaper_str!("MUTEENV").into(),
            Custom(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_track_attribute_key() {
        use TrackAttributeKey::*;
        assert_eq!(Mute.into_raw().as_ref(), reaper_str!("B_MUTE"));
        assert_eq!(
            Env(EnvChunkName::VolEnv).into_raw().as_ref(),
            reaper_str!("P_ENV:<VOLENV")
        );
        assert_eq!(
            Env(EnvChunkName::Custom(reaper_str!("MYENV").into()))
                .into_raw()
                .as_ref(),
            reaper_str!("P_ENV:<MYENV")
        );
        assert_eq!(
            TrackAttributeKey::ext("SWS_FOO").into_raw().as_ref(),
            reaper_str!("P_EXT:SWS_FOO")
        );
        assert_eq!(
            TrackAttributeKey::custom(reaper_str!("BLA"))
                .into_raw()
                .as_ref(),
            reaper_str!("BLA")
        );
    }
}
