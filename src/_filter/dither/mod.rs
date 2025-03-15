pub mod atkinson;
pub mod burkes;
pub mod floyd_steinberg;
pub mod minimized_average_error;
pub mod quantization;
pub mod stucki;

use crate::generic_image::GenericImage;

pub trait Dither {
    fn ditherize<'a, T: GenericImage>(image: &'a T, palette: &[T::Pixel]) -> &'a T;
}
