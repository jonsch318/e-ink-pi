use std::ops::{Add, Index, IndexMut, Sub};

use crate::pixel::{EuclidianDistance, Pixel, PixelComponent};

use super::ColorType;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Luma<T>(pub [T; 1]);
impl<T: PixelComponent> Pixel for Luma<T> {
    type Subpixel = T;

    const CHANNEL_COUNT: u8 = 1;

    const COLOR_TYPE: ColorType = ColorType::L8;

    #[inline(always)]
    fn channels(&self) -> &[Self::Subpixel] {
        &self.0
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        &mut self.0
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        assert_eq!(slice.len(), usize::from(Self::CHANNEL_COUNT));
        unsafe { &*(slice.as_ptr() as *const Luma<T>) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &Self {
        assert_eq!(slice.len(), usize::from(Self::CHANNEL_COUNT));
        unsafe { &*(slice.as_mut_ptr() as *const Luma<T>) }
    }

    fn map<F>(&self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let this: Self = *self;
        this.map_with_alpha(f, |x| x);
        this
    }

    fn map_with_alpha<F, G>(&self, mut f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let _ = g;
        let mut this = (*self).clone();
        this.0[0] = f(this.0[0]);
        this
    }
}

impl<T: PixelComponent> Sub for Luma<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Luma([self.0 - rhs.0]);
    }
}
impl<T: PixelComponent> Add for Luma<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Luma([self.0 + rhs.0]);
    }
}

impl<T: PixelComponent> EuclidianDistance for Luma<T> {
    fn dist_euclidian(&self, other: &Self) -> i32 {
        let diff: i32 = i32::from(self.0) - i32::from(other.0);

        return diff * diff;
    }
}
impl<T: PixelComponent> Index<usize> for Luma<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, i: usize) -> &T {
        &self.0[i]
    }
}
impl<T: PixelComponent> IndexMut<usize> for Luma<T> {
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.0[i]
    }
}

impl<T: PixelComponent> From<[T; 3]> for Luma<T> {
    fn from(c: [T; 3]) -> Self {
        Self(c)
    }
}
