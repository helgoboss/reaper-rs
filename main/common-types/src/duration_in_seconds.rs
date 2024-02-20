use crate::PositionInSeconds;
use nutype::nutype;
use std::ops::{Add, Mul};

/// This represents a duration expressed as positive amount of seconds.
#[nutype(
    new_unchecked,
    validate(finite, greater_or_equal = 0.0),
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
pub struct DurationInSeconds(f64);

impl DurationInSeconds {
    /// The minimum duration (zero, empty).
    pub const ZERO: DurationInSeconds = unsafe { DurationInSeconds::new_unchecked(0.0) };

    /// The minimum duration (zero, empty).
    pub const MIN: DurationInSeconds = Self::ZERO;

    /// The maximum possible duration (highest possible floating-point number).
    pub const MAX: DurationInSeconds = unsafe { DurationInSeconds::new_unchecked(f64::MAX) };

    nutype_additions!(f64);

    /// Saturating duration subtraction.
    ///
    /// Computes `self - rhs`, saturating at zero.
    pub fn saturating_sub(&self, rhs: DurationInSeconds) -> DurationInSeconds {
        DurationInSeconds::new_panic(0.0f64.max(self.get() - rhs.get()))
    }
}

impl Add for DurationInSeconds {
    type Output = DurationInSeconds;

    fn add(self, rhs: DurationInSeconds) -> Self::Output {
        Self::new_panic(self.get() + rhs.get())
    }
}

impl Mul<f64> for DurationInSeconds {
    type Output = Option<Self>;

    fn mul(self, rhs: f64) -> Option<Self> {
        if rhs < 0.0 {
            return None;
        }
        Some(Self::new_panic(self.get() * rhs))
    }
}

impl TryFrom<PositionInSeconds> for DurationInSeconds {
    type Error = DurationInSecondsError;

    fn try_from(value: PositionInSeconds) -> Result<Self, Self::Error> {
        value.into_inner().try_into()
    }
}
