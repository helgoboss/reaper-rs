/// 24-bit non-linear sRGB color.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    /// Creates this color by providing the non-linear sRGB components contained in an array.
    pub const fn from_array(rgb: [u8; 3]) -> Self {
        Self::rgb(rgb[0], rgb[1], rgb[2])
    }

    /// Creates this color by providing the non-linear sRGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[cfg(feature = "palette")]
mod palette_impl {
    use crate::RgbColor;
    use palette::rgb::Rgb;
    use palette::{Hsl, IntoColor, LinSrgb, Srgb};

    impl RgbColor {
        /// Converts an arbitrary palette RGB color to our color type.
        pub fn from_palette<S, T>(color: Rgb<S, T>) -> Self
        where
            Srgb<u8>: From<Rgb<S, T>>,
        {
            // We want non-linear sRGB
            let srgb: Srgb<u8> = color.into();
            Self::rgb(srgb.red, srgb.green, srgb.blue)
        }

        /// Converts this color to its closest palette color type (24-bit sRGB without alpha).
        pub fn to_palette(&self) -> Srgb<u8> {
            Srgb::new(self.r, self.g, self.b)
        }

        /// Convenience function to start working with the color in the RGB color space.
        ///
        /// Can be converted back into our color type using `.into()`.
        pub fn to_linear_srgb(&self) -> LinSrgb {
            self.to_palette().into_linear()
        }

        /// Convenience function to start working with the color in the HSL color space.
        pub fn to_hsl(&self) -> Hsl {
            let srgb: Srgb = self.to_palette().into_format();
            srgb.into_color()
        }
    }

    impl<S, T> From<Rgb<S, T>> for RgbColor
    where
        Srgb<u8>: From<Rgb<S, T>>,
    {
        fn from(value: Rgb<S, T>) -> Self {
            RgbColor::from_palette(value)
        }
    }

    impl From<Hsl> for RgbColor {
        fn from(value: Hsl) -> Self {
            let srgb: Srgb = value.into_color();
            let srgb: Srgb<u8> = srgb.into_format();
            RgbColor::from_palette(srgb)
        }
    }
}
