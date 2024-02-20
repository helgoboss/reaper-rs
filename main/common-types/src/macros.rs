/// Construct a color like this: `color!("EFEFEF")`
#[cfg(feature = "color-macros")]
#[macro_export]
macro_rules! color {
    ($arr:literal) => {
        reaper_common_types::RgbColor::from_array(reaper_common_types::hex!($arr))
    };
}

/// Construct a set of color constants.
#[cfg(feature = "color-macros")]
#[macro_export]
macro_rules! colors {
    (
        $(
            $name:ident = $arr:literal;
        )+
    ) => {
        $(
            pub const $name: reaper_common_types::RgbColor = reaper_common_types::color!($arr);
        )+
    };
}

macro_rules! nutype_additions {
    ($ty:ty) => {
        /// Constructs a new value of this type, panicking if the given raw value is invalid.
        ///
        /// Use this if you are reasonably sure that the given value is valid. If your assumption turns out to be not
        /// true, you will get a panic with a good error message, which is at least better than undefined behavior.
        ///
        /// # Panics
        ///
        /// Panics if the given raw value is invalid.
        pub fn new_panic(raw_value: $ty) -> Self {
            Self::new(raw_value).unwrap_or_else(|e| {
                panic!(
                    "couldn't create {} from {}: {e}",
                    std::any::type_name::<Self>(),
                    raw_value
                )
            })
        }
    };
}
