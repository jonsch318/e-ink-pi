// This module contains only pure quantization methods. For dithering see the dithering.rs module
use crate::pixel::Pixel;

fn nearest_color_filter(pixels: Vec<Pixel>, palette: &Vec<Pixel>) -> Vec<Pixel> {
    let mut quantized = pixels.to_vec();
    for (i, pixel) in pixels.iter().enumerate() {
        quantized[i] = nearest_color(*pixel, palette)
    }
    return quantized;
}

pub fn nearest_color(color: Pixel, palette: &Vec<Pixel>) -> Pixel {
    assert!(palette.len() > 0);
    let mut min_dist: i32 = i32::MAX;
    let mut min_palette_color = palette[0];
    for p_color in palette {
        let dist = dist_euclidian(color, *p_color);
        if dist < min_dist {
            min_dist = dist;
            min_palette_color = *p_color;
        }
    }
    return min_palette_color;
}

pub fn dist_euclidian(c1: Pixel, c2: Pixel) -> i32 {
    let r_diff: i32 = i32::from(c1.0) - i32::from(c2.0);
    let g_diff: i32 = i32::from(c1.1) - i32::from(c2.1);
    let b_diff: i32 = i32::from(c1.2) - i32::from(c2.2);

    return (r_diff * r_diff) + (g_diff * g_diff) + (b_diff * b_diff);
}
