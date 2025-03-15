//pub mod luma;
pub mod rgb;

//enum of supported color types
#[derive(Copy, PartialEq, Eq, Debug, Clone, Hash)]
#[non_exhaustive]
pub enum ColorType {
    /// Pixel is 8-bit luminance
    L8,
    /// Pixel is 8-bit luminance with an alpha channel
    // La8,
    /// Pixel contains 8-bit R, G and B channels
    Rgb8,
    // /// Pixel is 8-bit RGB with an alpha channel
    // Rgba8,
    // /// Pixel is 16-bit luminance
    // L16,
    // /// Pixel is 16-bit luminance with an alpha channel
    // La16,
    // /// Pixel is 16-bit RGB
    // Rgb16,
    // /// Pixel is 16-bit RGBA
    // Rgba16,
    //
    // /// Pixel is 32-bit float RGB
    // Rgb32F,
    // /// Pixel is 32-bit float RGBA
    // Rgba32F,
}

impl ColorType {
    #[must_use]
    pub fn bytes_per_pixel(self) -> u8 {
        match self {
            ColorType::L8 => 1,
            ColorType::Rgb8 => 3,
        }
    }

    #[must_use]
    pub fn has_alpha(self) -> bool {
        match self {
            ColorType::L8 | ColorType::Rgb8 => false,
        }
    }

    #[must_use]
    pub fn channel_count(self) -> u8 {
        match self {
            ColorType::L8 => 1,
            ColorType::Rgb8 => 3,
        }
    }
}
