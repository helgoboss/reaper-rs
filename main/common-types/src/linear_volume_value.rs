use crate::constants::TWENTY_OVER_LN10;
use crate::Db;
use nutype::nutype;

/// This represents a volume measured in REAPER's native volume unit.
///
/// What I call "REAPER's native volume unit" is in REAPER/WDL code often called `val` or `gain`. It's essentially
/// a dB value represented as linear factor, and thus it's suitable in scenarios such as altering the amplitude of
/// a sample, simply by multiplying with this value.
///
/// # Formulas
///
/// Some formulas for conversion from val to dB and vice versa. Using the constants
/// is maybe slightly more efficient.
///
/// ```ignore
/// TWENTY_OVER_LN10 = 20 / log(10)
/// LN10_OVER_TWENTY = log(10) / 20
///
/// db = log10(val) * 20
///    = log(val) / LN10_OVER_TWENTY
///    = log(val) * TWENTY_OVER_LN10
///
/// val = pow(10, db / 20.0)
///     = exp(db * LN10_OVER_TWENTY)
/// ```
///
/// # Examples
///
/// - A value of 0.0 or very close corresponds to -inf dB
/// - A value of 0.000000063095734448019 corresponds to -144 dB (the first dB value not showed as -inf anymore
///   in the REAPER GUI)
/// - A value of 0.5 corresponds to -6.02 dB (roughly halved volume)
/// - A value of 1.0 corresponds to 0.0 dB (unaltered volume)
/// - A value of 2.0 corresponds to 6.02 dB (roughly doubled volume)
/// - A value of 3.981071705535 corresponds to 12 dB (REAPER's "soft maximum" volume)
/// - Higher values are possible but harder to enter via GUI
///
/// # Usages
///
/// - Track volume
/// - Send volume
/// - Item volume
/// - Take volume
/// - Track peaks
#[nutype(
    new_unchecked,
    validate(greater_or_equal = 0.0),
    derive(
        Copy,
        Clone,
        PartialEq,
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
    default = 1.0
)]
pub struct LinearVolumeValue(f64);

impl LinearVolumeValue {
    /// The minimum possible value (0.0).
    ///
    /// In practice, REAPER considers this as largely equal to the [`MINUS_150_DB`] value.
    ///
    /// There's no reasonable maximum value because REAPER allows to exceed the "soft maximum" of 12 dB!
    ///
    /// [`MINUS_150_DB`]: #associatedconstant.MINUS_150_DB
    pub const MIN: LinearVolumeValue = unsafe { LinearVolumeValue::new_unchecked(0.0) };

    /// The not-a-number volume ([`f64::NAN`] = 1.#R dB).
    ///
    /// It's reasonable to assume that this isn't actually a valid value. However, REAPER doesn't
    /// prevent extensions from setting it, so you might run into it.
    ///
    /// [`f64::NAN`]: https://doc.rust-lang.org/std/f64/constant.NAN.html
    pub const NAN: LinearVolumeValue = unsafe { LinearVolumeValue::new_unchecked(f64::NAN) };

    /// The "soft minimum" volume (3.1622776601684e-008 = -150.0 dB).
    ///
    /// When setting a value, use [`MIN`] (0.0) instead because this is just an approximation.
    ///
    /// [`MIN`]: #associatedconstant.MIN
    pub const MINUS_150_DB: LinearVolumeValue =
        unsafe { LinearVolumeValue::new_unchecked(0.000000031622776601684) };

    /// The new (?) "soft minimum" volume (0.000000063095734448019 = -144.0 dB).
    ///
    /// When setting a value, use [`MIN`] (0.0) instead because this is just an approximation.
    ///
    /// [`MIN`]: #associatedconstant.MIN
    pub const MINUS_144_DB: LinearVolumeValue =
        unsafe { LinearVolumeValue::new_unchecked(0.000000063095734448019) };

    /// The "unaltered" volume (1.0 = 0.0 dB).
    pub const ZERO_DB: LinearVolumeValue = unsafe { LinearVolumeValue::new_unchecked(1.0) };

    /// The "soft maximum" volume (3.981071705535 = 12.0 dB).
    pub const TWELVE_DB: LinearVolumeValue =
        unsafe { LinearVolumeValue::new_unchecked(3.981071705535) };

    nutype_additions!(f64);

    /// Efficient conversion to a dB value with -150 dB minimum, pretty much as done in WDL's `db2val.h`.
    ///
    /// Doesn't call the REAPER API.
    pub fn to_db(&self) -> Db {
        self.to_db_ex(Db::MINUS_150_DB)
    }

    /// Efficient conversion to a dB value with configurable minimum, exactly as done in WDL's `db2val.h`.
    ///
    /// Doesn't call the REAPER API.
    pub fn to_db_ex(&self, min_db: Db) -> Db {
        let min_val = min_db.to_linear_volume_value();
        if *self <= min_val {
            return min_db;
        }
        Db::new_panic(self.into_inner().ln() * TWENTY_OVER_LN10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert!(LinearVolumeValue::new(-0.5).is_err());
        assert!(LinearVolumeValue::new(60.0).unwrap() < LinearVolumeValue::new(120.0).unwrap());
        assert_eq!(
            LinearVolumeValue::default(),
            LinearVolumeValue::new(1.0).unwrap()
        );
        assert_eq!(
            serde_json::from_str::<LinearVolumeValue>("5").unwrap(),
            LinearVolumeValue::new(5.0).unwrap()
        );
        assert!(serde_json::from_str::<LinearVolumeValue>("-0.5").is_err());
        assert_eq!(LinearVolumeValue::new(756.5).unwrap().to_string(), "756.5");
        assert_eq!(
            format!("{:?}", LinearVolumeValue::new(756.5).unwrap()),
            "LinearVolumeValue(756.5)"
        );
        unsafe {
            assert_eq!(LinearVolumeValue::new_unchecked(5.0).get(), 5.0);
        }
        let val: LinearVolumeValue = 5.0.try_into().unwrap();
        assert_eq!(val, LinearVolumeValue::new(5.0).unwrap())
    }
}
