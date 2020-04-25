#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FxChainType {
    OutputFxChain,
    InputFxChain,
}

// TODO-medium Maybe better implement this as normal pub(crate) method because it's an
// implementation detail
impl From<FxChainType> for bool {
    fn from(t: FxChainType) -> Self {
        t == FxChainType::InputFxChain
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MasterTrackBehavior {
    ExcludeMasterTrack,
    IncludeMasterTrack,
}

impl From<MasterTrackBehavior> for bool {
    fn from(v: MasterTrackBehavior) -> Self {
        v == MasterTrackBehavior::IncludeMasterTrack
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UndoHint {
    UndoIsRequired,
    UndoIsOptional,
}

impl From<UndoHint> for bool {
    fn from(v: UndoHint) -> Self {
        v == UndoHint::UndoIsOptional
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
