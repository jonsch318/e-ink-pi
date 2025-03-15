use std::ops::{Index, IndexMut};

use crate::pixel::Pixel;

#[derive(Debug)]
pub struct PixelIter<'a, I: ?Sized + 'a> {
    image: &'a I,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl<'a, I: GenericImage> Iterator for PixelIter<'a, I> {
    type Item = (u32, u32, &'a I::Pixel);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            //wrap around
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.height {
            return None;
        }

        let p = (self.x, self.y, self.image.get_pixel(self.x, self.y));
        self.x += 1;
        return Some(p);
    }
}

pub trait GenericImage: Index<(usize, usize)> {
    type Pixel: Pixel;

    fn dimensions(&self) -> (u32, u32);
    fn width(&self) -> u32 {
        let (w, _) = self.dimensions();
        return w;
    }
    fn height(&self) -> u32 {
        let (_, h) = self.dimensions();
        return h;
    }

    fn in_bounds(&self, x: u32, y: u32) -> bool {
        let (w, h) = self.dimensions();
        return x < w && y < h;
    }

    fn get_pixel(&self, x: u32, y: u32) -> &Self::Pixel;
    fn get_pixel_checked(&self, x: u32, y: u32) -> Option<&Self::Pixel>;

    fn iter(&self) -> PixelIter<Self> {
        let (width, height) = self.dimensions();
        PixelIter {
            image: self,
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

pub trait GenericImageMut: GenericImage + IndexMut<(usize, usize)> {
    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Self::Pixel;
    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel);

    // possibly unsafe get,put pixel without bounds checking

    /// Fills a rectagle with the data.
    ///
    /// Ignores out of bounds pixels if the rectangle is larger or outside of the image. If
    /// wrapping is wanted use `put_rect_wrapping` instead.
    ///
    /// Errors when `data.len() < width * height` even when the data is outside the image. This
    /// expects at least the rectangle size defined by width and height
    ///
    /// * `offset_x`:
    /// * `offset_y`:
    /// * `width`:
    /// * `height`:
    /// * `data`:
    fn put_rect(
        &mut self,
        offset_x: u32,
        offset_y: u32,
        width: u32,
        height: u32,
        data: &[Self::Pixel],
    ) -> Result<(), ()>;
}
