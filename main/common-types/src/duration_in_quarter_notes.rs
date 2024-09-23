use nutype::nutype;

/// This represents a duration expressed as positive amount of quarter notes.
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
pub struct DurationInQuarterNotes(f64);

impl DurationInQuarterNotes {
    /// The minimum duration (zero, empty).
    pub const ZERO: DurationInQuarterNotes = unsafe { DurationInQuarterNotes::new_unchecked(0.0) };

    /// The maximum possible duration (highest possible floating-point number).
    pub const MAX: DurationInQuarterNotes =
        unsafe { DurationInQuarterNotes::new_unchecked(f64::MAX) };

    nutype_additions!(f64);
}
