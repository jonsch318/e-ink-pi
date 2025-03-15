use std::{
    io::{self, Read},
    mem,
};

use crate::formats::gif::gif::Version;

use super::Block;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LogicalScreenDescriptor {
    ///width in pixels of the logical screen
    pub logical_screen_width: u16,
    ///height in pixels of the logical screen
    pub logical_screen_height: u16,
    /// Global Color Table Flag       1 Bit
    /// Color Resolution              3 Bits
    /// Sort Flag                     1 Bits
    /// Size of Global Color Table    3 Bits
    flags: u8,
    /// Index into the Global Color Table for the Background Color
    pub background_color_index: u8,
    /// Factor used to compute an approximation of the aspect ratio of the pixel in the original
    /// image.
    pub pixel_aspect_ratio: u8,
}

impl LogicalScreenDescriptor {
    pub const GLOBAL_COLOR_TABLE_FLAG_BIT: u8 = 0b10000000;
    pub const GLOBAL_COLOR_TABLE_OFFSET: u8 = 7;
    pub const COLOR_RESOLUTION_BITS: u8 = 0b1110000;
    pub const COLOR_RESOLUTION_OFFSET: u8 = 4;
    pub const SORT_FLAG_BIT: u8 = 0b1000;
    pub const SORT_FLAG_OFFSET: u8 = 3;
    pub const GLOBAL_COLOR_TABLE_SIZE_BITS: u8 = 0b0;
    pub const GLOBAL_COLOR_TABLE_SIZE_OFFSET: u8 = 0;

    pub fn global_color_table_flag(&self) -> bool {
        return self.flags & Self::GLOBAL_COLOR_TABLE_FLAG_BIT != 0;
    }
    pub fn color_resolution(&self) -> u8 {
        return (self.flags & Self::COLOR_RESOLUTION_BITS) >> Self::COLOR_RESOLUTION_OFFSET;
    }
    pub fn sort_flag(&self) -> bool {
        return self.flags & Self::SORT_FLAG_BIT != 0;
    }
    pub fn global_color_table_size(&self) -> u8 {
        return (self.flags & Self::GLOBAL_COLOR_TABLE_SIZE_BITS)
            >> Self::GLOBAL_COLOR_TABLE_SIZE_OFFSET;
    }
}

impl Block for LogicalScreenDescriptor {
    const BLOCK_SIZE: usize = 7;

    const VERSION: Version = Version::Version87a;
}

impl LogicalScreenDescriptor {
    pub fn parse<R: Read>(
        reader: &mut R,
    ) -> Result<LogicalScreenDescriptor, LogicalScreenDescriptorParseError> {
        let mut buf = [0u8; LogicalScreenDescriptor::BLOCK_SIZE];
        reader.read_exact(&mut buf)?;

        let descriptor = LogicalScreenDescriptor::from(&buf);
        Ok(descriptor)
    }
}

// Conversions
impl From<&[u8; Self::BLOCK_SIZE]> for LogicalScreenDescriptor {
    fn from(value: &[u8; Self::BLOCK_SIZE as usize]) -> Self {
        //TODO: fast_from with unsafe transmute.
        return LogicalScreenDescriptor {
            logical_screen_width: u16::from_le_bytes([value[0], value[1]]),
            logical_screen_height: u16::from_le_bytes([value[2], value[3]]),
            flags: value[4],
            background_color_index: value[5],
            pixel_aspect_ratio: value[6],
        };
    }
}

pub enum LogicalScreenDescriptorParseError {
    Io(io::Error),
}

impl From<io::Error> for LogicalScreenDescriptorParseError {
    fn from(value: io::Error) -> Self {
        LogicalScreenDescriptorParseError::Io(value)
    }
}
