// Minimized Average Error from Jarvis, Judice and Ninke see
// https://en.wikipedia.org/wiki/Error_diffusion#minimized_average_error

use crate::{
    generic_image::GenericImage,
    image_buffer::ImageBuffer,
    pixel::{Pixel, PixelComponent},
};

use super::{quantization::nearest_color, Dither};

struct MinimizedAverageError {}

impl Dither for MinimizedAverageError {
    fn ditherize<'a, T: crate::generic_image::GenericImage>(
        image: &'a T,
        palette: &[T::Pixel],
    ) -> &'a T {
        return dither_minimized_average_error(image, palette);
    }
}

pub fn dither_minimized_average_error<I, P, S>(
    image: &I,
    palette: &[I::Pixel],
) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImage<Pixel = P>,
    P: Pixel<Subpixel = S>,
    S: PixelComponent,
{
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(width, height);

    for (x, y, v) in image.into_iter() {
        let commulative_v = v + out.get_pixel(x, y);
        let new_v = nearest_color(commulative_v, palette);
        out.put_pixel(x, y, new_v);

        let quantization_error = new_v - commulative_v;

        macro_rules! diffuse_error {
            ($ident:expr, $x:expr, $y:expr, $c:expr) => {
                if $ident.in_bounds($x, $y) {
                    $ident.put_pixel($x, $y, out.get_pixel($x, $y).map(|x| x * $c));
                }
            };
        }
        diffuse_error!(out, x + 1, y, 7 / 48);
        diffuse_error!(out, x + 2, y, 5 / 48);

        diffuse_error!(out, x - 2, y + 1, 3 / 48);
        diffuse_error!(out, x - 1, y + 1, 5 / 48);
        diffuse_error!(out, x, y + 1, 7 / 48);
        diffuse_error!(out, x + 1, y + 1, 5 / 48);
        diffuse_error!(out, x + 2, y + 1, 3 / 48);

        diffuse_error!(out, x - 2, y + 2, 1 / 48);
        diffuse_error!(out, x - 1, y + 2, 3 / 48);
        diffuse_error!(out, x, y + 2, 5 / 48);
        diffuse_error!(out, x + 1, y + 2, 3 / 48);
        diffuse_error!(out, x + 2, y + 2, 1 / 48);
    }

    return out;
}
