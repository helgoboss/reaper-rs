use nutype::nutype;

/// This represents a position expressed as an amount of pulses per quarter note
/// (= PPQ or MIDI ticks).
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
pub struct PositionInPulsesPerQuarterNote(f64);

impl PositionInPulsesPerQuarterNote {
    /// Position at 0.0 seconds. E.g. start of project, measure, etc. depending on the context.
    pub const ZERO: PositionInPulsesPerQuarterNote =
        unsafe { PositionInPulsesPerQuarterNote::new_unchecked(0.0) };

    nutype_additions!(f64);
}
