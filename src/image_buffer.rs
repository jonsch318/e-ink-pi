use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut, Range},
};

use num_traits::Zero;

use crate::{
    generic_image::{GenericImage, GenericImageMut},
    pixel::Pixel,
};

#[derive(Debug, Clone, Copy, Hash)]
pub struct ImageBuffer<P, Container>
where
    P: Pixel,
{
    width: u32,
    height: u32,
    _phantom: PhantomData<P>,
    data: Container,
}

impl<P, Container> Index<(usize, usize)> for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    type Output = P;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.get_pixel(x as u32, y as u32)
    }
}

impl<P, Container> IndexMut<(usize, usize)> for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]> + DerefMut,
{
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.get_pixel_mut(x as u32, y as u32)
    }
}

impl<P, Container> ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    #[inline(always)]
    /// Gets a range for some pixel i.e. pixel index .. pixel index + channel_count
    /// This expect fully correct container structure
    ///
    /// * `x`: the pixel index horizontally
    /// * `y`: the pixel index vertically
    fn get_pixel_range(&self, x: u32, y: u32) -> Range<usize> {
        let num_channels = P::CHANNEL_COUNT as usize;
        let index = (y as usize * self.width as usize + x as usize) * num_channels;
        index..index + num_channels
    }

    ////////////
    //  From  //
    ////////////
    pub fn from_container(width: u32, height: u32, buffer: Container) -> Option<Self> {
        if buffer.len() >= width as usize * height as usize * P::CHANNEL_COUNT as usize {
            return None;
        }
        Some(ImageBuffer {
            data: buffer,
            width,
            height,
            _phantom: PhantomData,
        })
    }

    ////////////
    //  Into  //
    ////////////
    pub fn into_container(self) -> Container {
        self.data
    }
    pub fn as_container(&self) -> &Container {
        &self.data
    }
}

impl<P, Container> GenericImage for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    type Pixel = P;

    fn dimensions(&self) -> (u32, u32) {
        return (self.width, self.height);
    }

    fn get_pixel(&self, x: u32, y: u32) -> &Self::Pixel {
        P::from_slice(&self.data[self.get_pixel_range(x, y)])
    }
    fn get_pixel_checked(&self, x: u32, y: u32) -> Option<&Self::Pixel> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some(P::from_slice(&self.data[self.get_pixel_range(x, y)]))
    }
}

impl<P, Container> GenericImageMut for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]> + DerefMut,
{
    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Self::Pixel {
        let range = self.get_pixel_range(x, y);
        P::from_slice_mut(&mut self.data[range])
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        *self.get_pixel_mut(x, y) = pixel;
    }

    fn put_rect(
        &mut self,
        offset_x: u32,
        offset_y: u32,
        width: u32,
        height: u32,
        data: &[Self::Pixel],
    ) -> Result<(), ()> {
        if data.len() < (width as usize * height as usize) {
            return Err(());
        }

        let (image_w, image_h) = self.dimensions();

        let mut y = offset_y;
        while y < image_h && y < offset_y + height {
            let mut x = offset_x;
            while x < image_w && x < offset_x + width {
                let data_index = (y - offset_y) * width + (x - offset_x);
                self.put_pixel(x, y, data[data_index as usize]);

                x += 1;
            }
            y += 1;
        }
        Ok(())
    }
}

////////////////////////////////
//  Concrete Implementations  //
////////////////////////////////

impl<P: Pixel> ImageBuffer<P, Vec<P::Subpixel>> {
    #[must_use]
    pub fn new(width: u32, height: u32) -> ImageBuffer<P, Vec<P::Subpixel>> {
        let size = width as usize * height as usize * P::CHANNEL_COUNT as usize;
        ImageBuffer {
            data: vec![Zero::zero(); size],
            width,
            height,
            _phantom: PhantomData,
        }
    }

    ////////////
    //  From  //
    ////////////
    pub fn from_vec(
        width: u32,
        height: u32,
        buffer: Vec<P::Subpixel>,
    ) -> Option<ImageBuffer<P, Vec<P::Subpixel>>> {
        ImageBuffer::from_container(width, height, buffer)
    }

    ////////////
    //  Into  //
    ////////////
    pub fn into_vec(self) -> Vec<P::Subpixel> {
        self.into_container()
    }
}

#[cfg(test)]
mod test {
    // use crate::{colors::rgb::RGB, generic_image::GenericImageMut};
    //
    // use super::ImageBuffer;

    #[test]
    fn zero_width_zero_height() {
        // let mut image: ImageBuffer<RGB<u8>, Vec<u8>> = ImageBuffer::new(0, 0);
    }
}
