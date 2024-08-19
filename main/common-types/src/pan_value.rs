use nutype::nutype;

/// This represents a pan measured in REAPER's native pan unit.
#[nutype(
    new_unchecked,
    validate(finite, greater_or_equal = -1.0, less_or_equal = 1.0),
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
pub struct PanValue(f64);

impl PanValue {
    /// The minimum possible value (= [`LEFT`]).
    ///
    /// [`LEFT`]: #associatedconstant.LEFT
    pub const MIN: PanValue = PanValue::LEFT;

    /// The "extreme" left value (-1.0).
    pub const LEFT: PanValue = unsafe { PanValue::new_unchecked(-1.0) };

    /// The center value (0.0).
    pub const CENTER: PanValue = unsafe { PanValue::new_unchecked(0.0) };

    /// The "extreme" right value (1.0).
    pub const RIGHT: PanValue = unsafe { PanValue::new_unchecked(1.0) };

    /// The maximum possible value (= [`RIGHT`]).
    ///
    /// [`RIGHT`]: #associatedconstant.RIGHT
    pub const MAX: PanValue = PanValue::RIGHT;

    nutype_additions!(f64);

    /// Constructs a new value of this type, clamping to the minimum or maximum if the given raw value is invalid.
    ///
    /// # Panics
    ///
    /// Panics if the given raw value is `NaN`.
    pub fn new_clamped(raw_value: f64) -> Self {
        Self::new_panic(raw_value.clamp(-1.0, 1.0))
    }
}
