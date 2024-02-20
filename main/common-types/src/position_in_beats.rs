use nutype::nutype;

/// This represents a position expressed as an amount of beats.
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
pub struct PositionInBeats(f64);

impl PositionInBeats {
    /// Position at 0.0 seconds. E.g. start of project, measure, etc. depending on the context.
    pub const ZERO: PositionInBeats = unsafe { PositionInBeats::new_unchecked(0.0) };

    nutype_additions!(f64);
}
