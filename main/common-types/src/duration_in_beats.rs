use nutype::nutype;

/// This represents a duration expressed as positive amount of beats.
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
pub struct DurationInBeats(f64);

impl DurationInBeats {
    /// The minimum duration (zero, empty).
    pub const ZERO: DurationInBeats = unsafe { DurationInBeats::new_unchecked(0.0) };

    /// The maximum possible duration (highest possible floating-point number).
    pub const MAX: DurationInBeats = unsafe { DurationInBeats::new_unchecked(f64::MAX) };

    nutype_additions!(f64);
}
