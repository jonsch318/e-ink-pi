use std::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Add, Div, Index, IndexMut, Mul, Rem, Sub},
};

use crate::pixel::{EuclidianDistance, Pixel, PixelComponent};

// ##### DEFINITIONS #####
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct RGB<T: PixelComponent>(pub [T; 3]);
impl<T: PixelComponent> Pixel for RGB<T> {
    type Subpixel = T;

    const CHANNEL_COUNT: u8 = 3;

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
        unsafe { &*(slice.as_ptr() as *const RGB<T>) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut RGB<T> {
        assert_eq!(slice.len(), usize::from(Self::CHANNEL_COUNT));

        unsafe { &mut *(slice.as_mut_ptr() as *mut RGB<T>) }
    }

    fn map_with_alpha<F, G>(&self, mut f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let _ = g;
        let mut this = *self;

        for v in this.0[..3].iter_mut() {
            *v = f(*v)
        }
        this
    }

    fn map<F>(&self, _f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        todo!()
    }

    const DEFAULT_MAX_VALUE: Self = Self([
        T::DEFAULT_MAX_VALUE,
        T::DEFAULT_MAX_VALUE,
        T::DEFAULT_MAX_VALUE,
    ]);

    const DEFAULT_MIN_VALUE: Self = Self([
        T::DEFAULT_MIN_VALUE,
        T::DEFAULT_MIN_VALUE,
        T::DEFAULT_MIN_VALUE,
    ]);
}

impl<T: Default + PixelComponent> Default for RGB<T> {
    fn default() -> Self {
        Self([T::default(), T::default(), T::default()])
    }
}

impl<T: PixelComponent> Add for RGB<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return RGB([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ]);
    }
}

impl<T: PixelComponent> Sub for RGB<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return RGB([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ]);
    }
}

impl<T: PixelComponent> Mul for RGB<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        return RGB([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ]);
    }
}

impl<T: PixelComponent> Div for RGB<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        return RGB([
            self.0[0] / rhs.0[0],
            self.0[1] / rhs.0[1],
            self.0[2] / rhs.0[2],
        ]);
    }
}

impl<T: PixelComponent> Rem for RGB<T> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        return RGB([
            self.0[0] % rhs.0[0],
            self.0[1] % rhs.0[1],
            self.0[2] % rhs.0[2],
        ]);
    }
}

impl<T: PixelComponent> EuclidianDistance for RGB<T> {
    fn dist_euclidian(&self, _other: &Self) -> i32 {
        todo!();
        // let r_diff: i32 = i32::from(self.0) - i32::from(other.0);
        // let g_diff: i32 = i32::from(self.1) - i32::from(other.1);
        // let b_diff: i32 = i32::from(self.2) - i32::from(other.2);
        //
        // return (r_diff * r_diff) + (g_diff * g_diff) + (b_diff * b_diff);
    }
}

impl<T: PixelComponent> Index<usize> for RGB<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, i: usize) -> &T {
        &self.0[i]
    }
}
impl<T: PixelComponent> IndexMut<usize> for RGB<T> {
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.0[i]
    }
}

impl<T: PixelComponent> From<[T; 3]> for RGB<T> {
    fn from(c: [T; 3]) -> Self {
        Self(c)
    }
}

impl<T: PixelComponent> From<&[T; 3]> for RGB<T> {
    fn from(c: &[T; 3]) -> Self {
        Self([c[0], c[1], c[2]])
    }
}

impl<T: PixelComponent + LowerHex> Display for RGB<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl<T: PixelComponent + LowerHex> LowerHex for RGB<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
    }
}
