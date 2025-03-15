use crate::{image::Image, pixel::Pixel};

use super::quantization::nearest_color;
pub trait Dither {
    // add code here
    fn ditherize<T: Image>(image: &T, palette: &Vec<Color>) -> T;
}

struct RandomDither;
impl Dither for RandomDither {
    fn ditherize<T: Image>(image: &T, palette: &Vec<Color>) -> T {
        todo!()
    }
}

// ##### ERROR_DIFFUSION #####
struct FloydSteinbergDither;
impl Dither for FloydSteinbergDither {
    fn ditherize<T: Image>(image: &T, palette: &Vec<Color>) -> T {
        let mut dithered = image.clone();

        for (i, pixel) in dithered.data().iter().enumerate() {
            let nearest = nearest_color(*pixel, palette);
            let quantization_error = (
                pixel.0 - nearest.0,
                pixel.1 - nearest.1,
                pixel.2 - nearest.2,
            );

            dithered.data()[i + 1].0 = dithered.data()[i + 1].0 + quantization_error.0 * (7 / 16);
            dithered.data()[i + 1].1 = dithered.data()[i + 1].1 + quantization_error.1 * (7 / 16);
            dithered.data()[i + 1].2 = dithered.data()[i + 1].2 + quantization_error.2 * (7 / 16);

            dithered.data()[i + image.width() as usize].0 =
                dithered.data()[i + image.width() as usize].0 + quantization_error.0 * (3 / 16);
            dithered.data()[i + image.width() as usize].1 =
                dithered.data()[i + image.width() as usize].1 + quantization_error.1 * (3 / 16);
            dithered.data()[i + image.width() as usize].2 =
                dithered.data()[i + image.width() as usize].2 + quantization_error.2 * (3 / 16);
        }

        return dithered;
    }
}
