use crate::MidiDeviceId;
use helgoboss_midi::{U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackFxChainType {
    NormalFxChain,
    /// On the master track this corresponds to the monitoring FX chain
    InputFxChain,
}

// TODO-medium Maybe better implement this as normal pub(crate) method because it's an
// implementation detail
impl From<TrackFxChainType> for bool {
    fn from(t: TrackFxChainType) -> Self {
        t == TrackFxChainType::InputFxChain
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MasterTrackBehavior {
    WithoutMasterTrack,
    WithMasterTrack,
}

impl From<MasterTrackBehavior> for bool {
    fn from(v: MasterTrackBehavior) -> Self {
        v == MasterTrackBehavior::WithMasterTrack
    }
}

// TODO-medium Wait for jf to explain the meaning of this
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UndoHint {
    Normal,
    IsUndo,
}

impl From<UndoHint> for bool {
    fn from(v: UndoHint) -> Self {
        v == UndoHint::IsUndo
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueChange<T: Copy> {
    Absolute(T),
    Relative(T),
}

impl<T: Copy> ValueChange<T> {
    pub(crate) fn value(&self) -> T {
        use ValueChange::*;
        match self {
            Absolute(v) => *v,
            Relative(v) => *v,
        }
    }

    pub(crate) fn is_relative(&self) -> bool {
        matches!(self, ValueChange::Relative(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UndoBehavior {
    WithoutUndoPoint,
    WithUndoPoint,
}

impl From<UndoBehavior> for bool {
    fn from(h: UndoBehavior) -> Self {
        h == UndoBehavior::WithUndoPoint
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferBehavior {
    Copy,
    Move,
}

impl From<TransferBehavior> for bool {
    fn from(t: TransferBehavior) -> Self {
        t == TransferBehavior::Move
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackDefaultsBehavior {
    WithoutDefaultEnvAndFx,
    WithDefaultEnvAndFx,
}

impl From<TrackDefaultsBehavior> for bool {
    fn from(v: TrackDefaultsBehavior) -> Self {
        v == TrackDefaultsBehavior::WithDefaultEnvAndFx
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GangBehavior {
    GangDenied,
    GangAllowed,
}

impl From<GangBehavior> for bool {
    fn from(v: GangBehavior) -> Self {
        v == GangBehavior::GangAllowed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum RecordArmState {
    Unarmed = 0,
    Armed = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum FxShowFlag {
    HideChain = 0,
    ShowChain = 1,
    HideFloatingWindow = 2,
    ShowFloatingWindow = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackSendDirection {
    Receive = -1,
    Send = 0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackSendCategory {
    Receive = -1,
    Send = 0,
    HardwareOutput = 1,
}

impl From<TrackSendDirection> for TrackSendCategory {
    fn from(v: TrackSendDirection) -> Self {
        use TrackSendDirection::*;
        match v {
            Receive => TrackSendCategory::Receive,
            Send => TrackSendCategory::Send,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StuffMidiMessageTarget {
    VirtualMidiKeyboardQueue,
    MidiAsControlInputQueue,
    VirtualMidiKeyboardQueueOnCurrentChannel,
    MidiOutputDevice(MidiDeviceId),
}

impl From<StuffMidiMessageTarget> for i32 {
    fn from(t: StuffMidiMessageTarget) -> Self {
        use StuffMidiMessageTarget::*;
        match t {
            VirtualMidiKeyboardQueue => 0,
            MidiAsControlInputQueue => 1,
            VirtualMidiKeyboardQueueOnCurrentChannel => 2,
            MidiOutputDevice(id) => 16 + id.0 as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackFxRef {
    NormalFxChain(u32),
    InputFxChain(u32),
}

// Converts directly to the i32 value that is expected by low-level track-FX related functions
impl From<TrackFxRef> for i32 {
    fn from(v: TrackFxRef) -> Self {
        use TrackFxRef::*;
        let positive = match v {
            InputFxChain(idx) => 0x1000000 + idx,
            NormalFxChain(idx) => idx,
        };
        positive as i32
    }
}

// Converts from a value returned by low-level track-FX related functions turned into u32.
impl From<u32> for TrackFxRef {
    fn from(v: u32) -> Self {
        use TrackFxRef::*;
        if v >= 0x1000000 {
            InputFxChain(v - 0x1000000)
        } else {
            NormalFxChain(v)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackFxAddByNameBehavior {
    Add = -1,
    Query = 0,
    AddIfNotFound = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionValueChange {
    AbsoluteLowRes(U7),
    AbsoluteHighRes(U14),
    Relative1(U7),
    Relative2(U7),
    Relative3(U7),
}
