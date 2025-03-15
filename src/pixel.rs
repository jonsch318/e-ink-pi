use num_traits::{Bounded, NumOps, PrimInt};

use crate::colors::ColorType;

pub trait EuclidianDistance {
    fn dist_euclidian(&self, other: &Self) -> i32;
}

/// A general Pixel. Can be any format
pub trait Pixel: Sized + Copy + Clone + EuclidianDistance + NumOps {
    type Subpixel: PixelComponent;

    const CHANNEL_COUNT: u8;
    //const COLOR_TYPE: ColorType;

    fn channels(&self) -> &[Self::Subpixel];
    fn channels_mut(&mut self) -> &mut [Self::Subpixel];

    fn from_slice(slice: &[Self::Subpixel]) -> &Self;
    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self;

    /// applies a function f to every color channel of the pixel and g to the alpha channel
    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel;

    /// applies a function to every color channel of the pixel;
    fn map<F>(&self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.map_with_alpha(f, |x| x)
    }

    fn iter(&self) -> PixelIter<'_, Self::Subpixel> {
        PixelIter {
            index: 0,
            data: self.channels(),
        }
    }

    const DEFAULT_MAX_VALUE: Self;
    const DEFAULT_MIN_VALUE: Self;
}

pub struct PixelIter<'a, T: PixelComponent> {
    index: usize,
    data: &'a [T],
}
impl<'a, T: PixelComponent> Iterator for PixelIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= self.data.len() {
            let res = self.data[self.index];
            self.index += 1;
            return Some(res);
        } else {
            None
        }
    }
}

/// The type of each channel in a pixel. For example, this can be `u8`, `u16`, `f32`.
pub trait PixelComponent: Clone + Copy + PartialOrd<Self> + Bounded + PrimInt {
    /// The maximum value for this type of primitive within the context of color.
    /// For floats, the maximum is `1.0`, whereas the integer types inherit their usual maximum values.
    const DEFAULT_MAX_VALUE: Self;

    /// The minimum value for this type of primitive within the context of color.
    /// For floats, the minimum is `0.0`, whereas the integer types inherit their usual minimum values.
    const DEFAULT_MIN_VALUE: Self;
}

macro_rules! declare_primitive {
    ($base:ty: ($from:expr)..$to:expr) => {
        impl PixelComponent for $base {
            const DEFAULT_MAX_VALUE: Self = $to;
            const DEFAULT_MIN_VALUE: Self = $from;
        }
    };
}

declare_primitive!(usize: (0)..Self::MAX);
declare_primitive!(u8: (0)..Self::MAX);
declare_primitive!(u16: (0)..Self::MAX);
declare_primitive!(u32: (0)..Self::MAX);
declare_primitive!(u64: (0)..Self::MAX);

declare_primitive!(isize: (Self::MIN)..Self::MAX);
declare_primitive!(i8: (Self::MIN)..Self::MAX);
declare_primitive!(i16: (Self::MIN)..Self::MAX);
declare_primitive!(i32: (Self::MIN)..Self::MAX);
declare_primitive!(i64: (Self::MIN)..Self::MAX);
//declare_primitive!(f32: (0.0)..1.0);
//declare_primitive!(f64: (0.0)..1.0);
