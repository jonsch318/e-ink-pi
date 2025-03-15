//! sdasd

use std::fmt::Display;

use crate::{colors::rgb::RGB, image_buffer::ImageBuffer};

use super::{
    consts::{GIF_CONST_VERSION_87A, GIF_CONST_VERSION_89A},
    errors::GIFParseError,
};

enum GIFFrameDisposal {
    Unspecified,
    NoDisposal,
    RestoreBackground,
    RestorePrevious,
}

/// GIF Image either Single Image Buffer or GIF Image Animation
/// TODO: Implement MultiImageBuffer and use it here
pub enum GIFImage {
    None,
    Single(SingleGIF),
    Animation(MultiGIF),
}

pub trait GIFDecode {
    fn decode(self) -> Result<GIFImage, GIFParseError>;
}

type SingleGIF = ImageBuffer<RGB<u8>, Vec<u8>>;

pub struct MultiGIF {
    images: Vec<ImageBuffer<RGB<u8>, Vec<u8>>>,
    delay_time: u16,
    frame_disposal: GIFFrameDisposal,
    user_input: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    Version87a,
    Version89a,
}

impl Version {
    fn as_bytes(&self) -> &'static [u8; 3] {
        match self {
            Version::Version87a => GIF_CONST_VERSION_87A,
            Version::Version89a => GIF_CONST_VERSION_89A,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Version87a => write!(f, "GIF 87a"),
            Version::Version89a => write!(f, "GIF 89a"),
        }
    }
}

#[repr(u8)]
pub enum DisposalMethod {
    NoDisposal = 0,
    DoNotDispose = 1,
    RestoreBackground = 2,
    RestorePrevious = 3,
}

impl From<u8> for DisposalMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => DisposalMethod::NoDisposal,
            1 => DisposalMethod::DoNotDispose,
            2 => DisposalMethod::RestoreBackground,
            3 => DisposalMethod::RestoreBackground,
            _ => DisposalMethod::NoDisposal,
        }
    }
}
