// This module contains only pure quantization methods. For dithering see the dithering.rs module
use crate::pixel::Pixel;

use super::Dither;

struct NearstColorQuantization {}

impl Dither for NearstColorQuantization {
    fn ditherize<'a, T: crate::generic_image::GenericImage>(
        image: &'a T,
        palette: &[T::Pixel],
    ) -> &'a T {
        let mut copy = image.clone();
        for (x, y, v) in copy.into_iter() {
            copy.put_pixel(x, y, nearest_color(v, palette))
        }
        copy
    }
}

pub fn nearest_color<T: Pixel>(color: T, palette: &[T]) -> T {
    assert!(palette.len() > 0);
    let mut min_dist: i32 = i32::MAX;
    let mut min_palette_color = palette[0];
    for p_color in palette {
        let dist = color.dist_euclidian(p_color);
        if dist < min_dist {
            min_dist = dist;
            min_palette_color = *p_color;
        }
    }
    return min_palette_color;
}
