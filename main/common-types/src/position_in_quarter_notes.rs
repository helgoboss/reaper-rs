use crate::DurationInQuarterNotes;
use nutype::nutype;
use std::ops::{Add, Neg};

/// This represents a position expressed as an amount of quarter notes.
///
/// Can be negative.
#[nutype(
    new_unchecked,
    validate(finite),
    derive(
        Copy,
        Clone,
        Eq,
        PartialEq,
        Ord,
        PartialOrd,
        Debug,
        Default,
        Display,
        FromStr,
        Into,
        TryFrom,
        Serialize,
        Deserialize
    ),
    default = 0.0
)]
pub struct PositionInQuarterNotes(f64);

impl PositionInQuarterNotes {
    /// Position at 0.0 seconds. E.g. start of project, measure, etc. depending on the context.
    pub const ZERO: PositionInQuarterNotes = unsafe { PositionInQuarterNotes::new_unchecked(0.0) };

    nutype_additions!(f64);
}

impl From<DurationInQuarterNotes> for PositionInQuarterNotes {
    fn from(v: DurationInQuarterNotes) -> Self {
        PositionInQuarterNotes::new_panic(v.get())
    }
}

impl Add<DurationInQuarterNotes> for PositionInQuarterNotes {
    type Output = Self;

    fn add(self, rhs: DurationInQuarterNotes) -> Self {
        PositionInQuarterNotes::new_panic(self.get() + rhs.get())
    }
}

impl Neg for PositionInQuarterNotes {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new_panic(-self.get())
    }
}
