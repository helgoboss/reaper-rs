use crate::constants::LN10_OVER_TWENTY;
use crate::LinearVolumeValue;
use nutype::nutype;

/// Represents a volume measured in decibel.
#[nutype(
    new_unchecked,
    validate(greater_or_equal = -1000.0),
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
    default = 0.0
)]
pub struct Db(f64);

impl Db {
    /// The minimum possible value (= [`MINUS_INF`]).
    ///
    /// There's no maximum value because REAPER allows to exceed the "soft maximum" of 12 dB!
    ///
    /// [`MINUS_INF`]: #associatedconstant.MINUS_INF
    pub const MIN: Db = Db::MINUS_INF;

    /// The not-a-number volume ([`f64::NAN`] = 1.#R dB).
    ///
    /// It's reasonable to assume that this isn't actually a valid value. However, REAPER doesn't
    /// prevent extensions from setting it, so you might run into it.
    ///
    /// [`f64::NAN`]: https://doc.rust-lang.org/std/f64/constant.NAN.html
    pub const NAN: Db = unsafe { Db::new_unchecked(f64::NAN) };

    /// The negative infinity volume (-1000.0 = -inf dB).
    pub const MINUS_INF: Db = unsafe { Db::new_unchecked(-1000.0) };

    /// The old (?) "soft minimum" volume (-150.0 dB).
    pub const MINUS_150_DB: Db = unsafe { Db::new_unchecked(-150.0) };

    /// The new (?) "soft minimum" volume (-144.0 dB).
    pub const MINUS_144_DB: Db = unsafe { Db::new_unchecked(-144.0) };

    /// The "unaltered" volume (0.0 dB).
    pub const ZERO_DB: Db = unsafe { Db::new_unchecked(0.0) };

    /// The "soft maximum" volume (12.0 dB).
    pub const TWELVE_DB: Db = unsafe { Db::new_unchecked(12.0) };

    nutype_additions!(f64);

    /// Efficient conversion to a gain value, exactly as done in WDL's `db2val.h`.
    ///
    /// Doesn't call the REAPER API.
    pub fn to_linear_volume_value(&self) -> LinearVolumeValue {
        LinearVolumeValue::new((self.into_inner() * LN10_OVER_TWENTY).exp())
            .expect("couldn't convert Db to LinearVolumeValue")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert!(Db::new(60.0).unwrap() < Db::new(120.0).unwrap());
        assert_eq!(Db::default(), Db::new(1.0).unwrap());
        assert_eq!(
            serde_json::from_str::<Db>("5").unwrap(),
            Db::new(5.0).unwrap()
        );
        assert!(serde_json::from_str::<Db>("-2000").is_err());
        assert_eq!(Db::new(756.5).unwrap().to_string(), "756.5");
        assert_eq!(format!("{:?}", Db::new(756.5).unwrap()), "Db(756.5)");
        unsafe {
            assert_eq!(Db::new_unchecked(5.0).get(), 5.0);
        }
        let db: Db = 5.0.try_into().unwrap();
        assert_eq!(db, Db::new(5.0).unwrap())
    }
}
