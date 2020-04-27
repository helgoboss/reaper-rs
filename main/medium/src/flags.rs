use enumflags2::BitFlags;
use reaper_rs_low::raw;

pub enum UndoScope {
    All,
    Scoped(BitFlags<UndoFlag>),
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum UndoFlag {
    Freeze = raw::UNDO_STATE_FREEZE,
    Fx = raw::UNDO_STATE_FX,
    Items = raw::UNDO_STATE_ITEMS,
    MiscCfg = raw::UNDO_STATE_MISCCFG,
    TrackCfg = raw::UNDO_STATE_TRACKCFG,
}

impl From<UndoScope> for i32 {
    fn from(s: UndoScope) -> Self {
        use UndoScope::*;
        match s {
            All => raw::UNDO_STATE_ALL as i32,
            Scoped(flags) => flags.bits() as i32,
        }
    }
}
