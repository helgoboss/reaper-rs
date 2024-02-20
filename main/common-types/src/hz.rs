use nutype::nutype;
use std::ops::Div;

/// Represents a frequency measured in hertz (how often something happens per second).
#[nutype(
    new_unchecked,
    validate(finite, greater_or_equal = f64::EPSILON),
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
pub struct Hz(f64);

impl Hz {
    /// The minimum frequency.
    pub const MIN: Self = unsafe { Self::new_unchecked(f64::EPSILON) };

    nutype_additions!(f64);
}

impl Div<f64> for Hz {
    type Output = Option<Hz>;

    fn div(self, rhs: f64) -> Option<Hz> {
        Self::new(self.into_inner() / rhs).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert!(Hz::new(60.0).unwrap() < Hz::new(120.0).unwrap());
        assert_eq!(Hz::default(), Hz::new(f64::EPSILON).unwrap());
        assert_eq!(
            serde_json::from_str::<Hz>("5").unwrap(),
            Hz::new(5.0).unwrap()
        );
        assert!(serde_json::from_str::<Hz>("-0.5").is_err());
        assert_eq!(Hz::new(756.5).unwrap().to_string(), "756.5");
        assert_eq!(format!("{:?}", Hz::new(756.5).unwrap()), "Hz(756.5)");
        assert_eq!(Hz::MIN.into_inner(), f64::EPSILON);
        unsafe {
            assert_eq!(Hz::new_unchecked(5.0).get(), 5.0);
        }
        let hz: Hz = 5.0.try_into().unwrap();
        assert_eq!(hz, Hz::new(5.0).unwrap())
    }
}
