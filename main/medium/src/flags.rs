#![allow(renamed_and_removed_lints)]
use enumflags2::BitFlags;
use reaper_low::raw;

/// When creating an undo point, this defines what parts of the project might have been affected by
/// the undoable operation.
#[derive(Copy, Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, BitFlags)]
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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, BitFlags)]
#[repr(u32)]
pub enum TrackArea {
    /// Track control panel.
    Tcp = 1,
    /// Mixer control panel.
    Mcp = 2,
}

/// Defines how REAPER will buffer when playing previews.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, BitFlags)]
#[repr(u32)]
pub enum BufferingBehavior {
    /// Buffers the source.
    BufferSource = 1,
    /// Treats length changes in source as vari-speed and adjusts internal state accordingly if
    /// buffering.
    VariSpeed = 2,
}
