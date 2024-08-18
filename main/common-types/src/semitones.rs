use nutype::nutype;

/// Represents a pitch delta measured in semitones.
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
    default = f64::EPSILON
)]
pub struct Semitones(f64);

impl Semitones {
    /// No difference.
    pub const ZERO: Self = unsafe { Self::new_unchecked(0.0) };

    nutype_additions!(f64);
}
