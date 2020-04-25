use enumflags2::BitFlags;
use reaper_rs_low::raw;

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum UndoFlag {
    Freeze = raw::UNDO_STATE_FREEZE,
    Fx = raw::UNDO_STATE_FX,
    Items = raw::UNDO_STATE_ITEMS,
    MiscCfg = raw::UNDO_STATE_MISCCFG,
    TrackCfg = raw::UNDO_STATE_TRACKCFG,
}
