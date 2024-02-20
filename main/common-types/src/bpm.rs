use nutype::nutype;

/// Represents a tempo measured in beats per minute.
#[nutype(
    new_unchecked,
    validate(finite, greater_or_equal = 1.0, less_or_equal = 960.0),
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
    default = 1.0
)]
pub struct Bpm(f64);

impl Bpm {
    /// The minimum possible value.
    pub const MIN: Self = unsafe { Self::new_unchecked(1.0) };

    /// The maximum possible value.
    pub const MAX: Self = unsafe { Self::new_unchecked(960.0) };

    nutype_additions!(f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert!(Bpm::new(0.5).is_err());
        assert!(Bpm::new(1000.0).is_err());
        assert!(Bpm::new(60.0).unwrap() < Bpm::new(120.0).unwrap());
        assert_eq!(Bpm::default(), Bpm::new(1.0).unwrap());
        assert_eq!(
            serde_json::from_str::<Bpm>("5").unwrap(),
            Bpm::new(5.0).unwrap()
        );
        assert!(serde_json::from_str::<Bpm>("0.5").is_err());
        assert_eq!(Bpm::new(756.5).unwrap().to_string(), "756.5");
        assert_eq!(format!("{:?}", Bpm::new(756.5).unwrap()), "Bpm(756.5)");
        assert_eq!(Bpm::MIN.into_inner(), 1.0);
        const _ADDITION: f64 = Bpm::MIN.get() + Bpm::MAX.get();
        assert_eq!(Bpm::MIN.get(), 1.0);
        assert_eq!(Bpm::MAX.get(), 960.0);
        unsafe {
            assert_eq!(Bpm::new_unchecked(5.0).get(), 5.0);
        }
        let bpm: Bpm = 5.0.try_into().unwrap();
        assert_eq!(bpm, Bpm::new(5.0).unwrap());
        let bpm: Bpm = "5.0".parse().unwrap();
        assert_eq!(bpm, Bpm::new(5.0).unwrap());
    }

    #[test]
    #[should_panic]
    fn new_panic() {
        Bpm::new_panic(0.5);
    }
}
