use crate::duration_in_seconds::DurationInSeconds;
use nutype::nutype;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// Represents a position expressed as amount of seconds.
///
/// Sometimes this is a negative number, e.g. when it's a position on the timeline and a metronome
/// count-in is used or at the very beginning of the project (maybe because of rounding). Negative
/// project start values don't seem to cause negative position values though.
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
pub struct PositionInSeconds(f64);

impl PositionInSeconds {
    /// Position at 0.0 seconds. E.g. start of project, measure, etc. depending on the context.
    pub const ZERO: PositionInSeconds = unsafe { PositionInSeconds::new_unchecked(0.0) };

    nutype_additions!(f64);

    /// See [`f64::rem_euclid`].
    pub fn rem_euclid(self, rhs: DurationInSeconds) -> DurationInSeconds {
        DurationInSeconds::new_panic(self.get().rem_euclid(rhs.get()))
    }

    /// Computes the absolute value, returning a duration.
    pub fn abs(self) -> DurationInSeconds {
        DurationInSeconds::new_panic(self.get().abs())
    }
}

impl From<DurationInSeconds> for PositionInSeconds {
    fn from(v: DurationInSeconds) -> Self {
        PositionInSeconds::new_panic(v.get())
    }
}

impl Sub for PositionInSeconds {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        PositionInSeconds::new_panic(self.get() - rhs.get())
    }
}

impl Sub<f64> for PositionInSeconds {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self {
        PositionInSeconds::new_panic(self.get() - rhs)
    }
}

impl Add for PositionInSeconds {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        PositionInSeconds::new_panic(self.get() + rhs.get())
    }
}

impl Add<f64> for PositionInSeconds {
    type Output = Self;

    fn add(self, rhs: f64) -> Self {
        PositionInSeconds::new_panic(self.get() + rhs)
    }
}

impl Mul<f64> for PositionInSeconds {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new_panic(self.get() * rhs)
    }
}

impl Add<DurationInSeconds> for PositionInSeconds {
    type Output = Self;

    fn add(self, rhs: DurationInSeconds) -> Self {
        PositionInSeconds::new_panic(self.get() + rhs.get())
    }
}

// impl AddAssign<DurationInSeconds> for PositionInSeconds {
//     fn add_assign(&mut self, rhs: DurationInSeconds) {
//         self.0 += rhs.0;
//     }
// }

impl Sub<DurationInSeconds> for PositionInSeconds {
    type Output = Self;

    fn sub(self, rhs: DurationInSeconds) -> Self {
        Self::new_panic(self.get() - rhs.get())
    }
}

impl PartialEq<DurationInSeconds> for PositionInSeconds {
    fn eq(&self, other: &DurationInSeconds) -> bool {
        self.get() == other.get()
    }
}

impl Neg for PositionInSeconds {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new_panic(-self.get())
    }
}

impl PartialOrd<DurationInSeconds> for PositionInSeconds {
    fn partial_cmp(&self, other: &DurationInSeconds) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl Div<DurationInSeconds> for PositionInSeconds {
    type Output = Self;

    fn div(self, rhs: DurationInSeconds) -> Self::Output {
        Self::new_panic(self.get() / rhs.get())
    }
}

impl Rem<DurationInSeconds> for PositionInSeconds {
    type Output = Option<Self>;

    fn rem(self, rhs: DurationInSeconds) -> Option<Self> {
        let res = self.get() % rhs.get();
        PositionInSeconds::new(res).ok()
    }
}
