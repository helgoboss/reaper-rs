#![allow(renamed_and_removed_lints)]
use enumflags2::BitFlags;
use reaper_low::raw;

/// When creating an undo point, this defines what parts of the project might have been affected by
/// the undoable operation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UndoScope {
    /// Everything could have been affected.
    ///
    /// This is the safest variant but can lead to very large undo states.
    All,
    /// A combination of the given project parts could have been affected.
    ///
    /// If you miss some parts, *undo* can behave in weird ways.
    Scoped(BitFlags<ProjectPart>),
}

impl UndoScope {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use UndoScope::*;
        match self {
            All => raw::UNDO_STATE_ALL as i32,
            Scoped(flags) => flags.bits() as i32,
        }
    }
}

/// Part of a project that could have been affected by an undoable operation.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum ProjectPart {
    /// Freeze state.
    Freeze = raw::UNDO_STATE_FREEZE,
    /// Track master FX.
    Fx = raw::UNDO_STATE_FX,
    /// Track items.
    Items = raw::UNDO_STATE_ITEMS,
    /// Loop selection, markers, regions and extensions.
    MiscCfg = raw::UNDO_STATE_MISCCFG,
    /// Track/master vol/pan/routing and aLL envelopes (master included).
    TrackCfg = raw::UNDO_STATE_TRACKCFG,
}

/// Area in the REAPER window where a track might be displayed.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum TrackArea {
    /// Track control panel.
    Tcp = 1,
    /// Mixer control panel.
    Mcp = 2,
}

/// Defines how REAPER will buffer when playing previews.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum BufferingBehavior {
    /// Buffers the source.
    BufferSource = 1,
    /// Treats length changes in source as vari-speed and adjusts internal state accordingly if
    /// buffering.
    VariSpeed = 2,
}

/// Defines the behavior of an accelerator.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum AcceleratorBehavior {
    /// The ALT key must be held down.
    Alt = 0x10,
    /// The CTRL key must be held down.
    Control = 0x08,
    /// The SHIFT key must be held down.
    Shift = 0x04,
    /// The key member specifies a virtual-key code.
    ///
    /// If this flag is not specified, key is assumed to specify a character code.
    VirtKey = 0x01,
}

/// Activates certain behaviors when inserting a media file.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum InsertMediaFlag {
    StretchLoopToFitTimeSelection = 4,
    TryToMatchTempo1X = 8,
    TryToMatchTempo05X = 16,
    TryToMatchTempo2X = 32,
    DontPreservePitchWhenMatchingTempo = 64,
    NoLoopSectionIfStartPctEndPctSet = 128,
    /// Force loop regardless of global preference for looping imported items.
    ForceLoopRegardlessOfGlobalPreference = 256,
    /// Move to source preferred position (BWF start offset).
    MoveSourceToPreferredPosition = 4096,
    Reverse = 8192,
}

/// Defines which track grouping behaviors to prevent when using the `set_track_ui_*` functions.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum SetTrackUiFlags {
    PreventTrackGrouping = 1,
    PreventSelectionGanging = 2,
}

/// Defines nudge mode in `apply_nudge`
///
/// if not SetToValue — will nudge by value.
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum ApplyNudgeFlag {
    SetToValue = 1,
    Snap = 2,
}

/// Defines how project is saved in `Reaper::save_project_ex()`
#[enumflags2::bitflags]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum SaveProjectFlags {
    /// Save as RTrackTemplate.
    AsTrackTemplate = 1,
    /// Include media in track template.
    WithMedia = 2,
    /// Include envelopes in track template.
    WithEnvelopes = 4,
}
