use std::{fmt::Display, io::Read};

use crate::colors::rgb::RGB;

use super::{
    consts::{GIF_CONST_SIGNATURE, GIF_CONST_VERSION_87A, GIF_CONST_VERSION_89A},
    errors::*,
};

#[repr(u8)]
pub enum GIFBlockType {
    Graphic = 0x00,
    Trailer = 0x3B,
    Control = 0x80,
    SpecialPurpose = 0xFA,
}

impl From<u8> for GIFBlockType {
    fn from(value: u8) -> Self {
        match value {
            0x3B => GIFBlockType::Trailer,
            0x00..=0x7F => GIFBlockType::Graphic,
            0x80..=0xF9 => GIFBlockType::Control,
            0xFA..=0xFF => GIFBlockType::SpecialPurpose,
        }
    }
}

pub trait GIFBlock {
    const BLOCK_SIZE: usize;
    const TYPE: BlockType;
}

pub trait GIFExtension: GIFBlock {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ControlBlocks {
    GraphicsControlExtension = 0xF9,
    UnknownBlock = BlockType::Trailer as u8,
}

impl ControlBlocks {
    pub fn is_extension(self) -> bool {
        return false;
    }
}

impl From<u8> for ControlBlocks {
    fn from(value: u8) -> Self {
        match value {
            0xF9 => ControlBlocks::GraphicsControlExtension,
            _ => ControlBlocks::UnknownBlock,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GraphicRenderingBlocks {
    TableBasedImage = 0x2C,
    PlainTextExtension = 0x01,
    UnknownBlock = BlockType::Trailer as u8,
}

impl GraphicRenderingBlocks {
    pub fn is_extension(self) -> bool {
        return self == GraphicRenderingBlocks::TableBasedImage;
    }
}

impl From<u8> for GraphicRenderingBlocks {
    fn from(value: u8) -> Self {
        match value {
            0x2C => GraphicRenderingBlocks::TableBasedImage,
            0x01 => GraphicRenderingBlocks::PlainTextExtension,
            _ => GraphicRenderingBlocks::UnknownBlock,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum SpecialPurposeBlocks {
    UnknownBlock = BlockType::Trailer as u8,
    CommentExtension = 0xFE,
    ApplicationExtension = 0xFF,
}

impl SpecialPurposeBlocks {
    pub fn is_extension(self) -> bool {
        return false;
    }
}

impl From<u8> for SpecialPurposeBlocks {
    fn from(value: u8) -> Self {
        match value {
            0xFE => SpecialPurposeBlocks::CommentExtension,
            0xFF => SpecialPurposeBlocks::ApplicationExtension,
            _ => SpecialPurposeBlocks::UnknownBlock,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockRestrictions {
    None,
    GraphicRenderingBlock,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ExtensionType {
    GraphicControlExtension,
    ApplicationExtension,
    CommentExtension,
    PlainTextExtension,
}

impl ExtensionType {
    pub fn from_u8(x: u8) -> Option<Self> {
        match x {
            0xFF => Some(ExtensionType::ApplicationExtension),
            0xFE => Some(ExtensionType::CommentExtension),
            0xF9 => Some(ExtensionType::GraphicControlExtension),
            0x01 => Some(ExtensionType::PlainTextExtension),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum BlockType {
    Image = 0x2C,
    Extension = 0x21,
    Trailer = 0x3B,
}

//

impl BlockType {
    pub fn from_u8(x: u8) -> Option<Self> {
        match x {
            0x2C => Some(BlockType::Image),
            0x21 => Some(BlockType::Extension),
            0x3B => Some(BlockType::Trailer),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub version: GIFVersion,
}
impl Header {
    pub const BLOCK_SIZE: usize = 6;
}

impl TryFrom<&[u8; Self::BLOCK_SIZE as usize]> for Header {
    type Error = HeaderDecodeError;

    fn try_from(value: &[u8; Self::BLOCK_SIZE]) -> Result<Self, Self::Error> {
        let signature: &[u8; 3] = value[0..3]
            .try_into()
            .expect("signature slice has incorrect length");

        if signature == GIF_CONST_SIGNATURE {
            return Err(HeaderDecodeError::from(UnknownSignatureError {
                found: signature.clone(),
            }));
        }

        let version_bytes: &[u8; 3] = value[3..6]
            .try_into()
            .expect("signature slice has incorrect length");

        let version = GIFVersion::try_from(version_bytes)?;

        Ok(Header { version })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GIFVersion {
    Version87a,
    Version89a,
}

impl GIFVersion {
    fn as_bytes(&self) -> &'static [u8; 3] {
        match self {
            GIFVersion::Version87a => GIF_CONST_VERSION_87A,
            GIFVersion::Version89a => GIF_CONST_VERSION_89A,
        }
    }
}

impl TryFrom<&[u8; 3]> for GIFVersion {
    type Error = UnknownVersionError;

    fn try_from(value: &[u8; 3]) -> Result<Self, Self::Error> {
        match value {
            GIF_CONST_VERSION_87A => Ok(GIFVersion::Version87a),
            GIF_CONST_VERSION_89A => Ok(GIFVersion::Version89a),
            _ => Err(UnknownVersionError {
                found: value.clone(),
            }),
        }
    }
}

impl Display for GIFVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GIFVersion::Version87a => write!(f, "GIF 87a"),
            GIFVersion::Version89a => write!(f, "GIF 89a"),
        }
    }
}

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
    pub pixel_aspect_ration: u8,
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

    pub const BLOCK_SIZE: usize = size_of::<Self>();

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

impl From<&[u8; Self::BLOCK_SIZE]> for LogicalScreenDescriptor {
    fn from(value: &[u8; Self::BLOCK_SIZE as usize]) -> Self {
        return LogicalScreenDescriptor {
            logical_screen_width: u16::from_le_bytes([value[0], value[1]]),
            logical_screen_height: u16::from_le_bytes([value[2], value[3]]),
            flags: value[4],
            background_color_index: value[5],
            pixel_aspect_ration: value[6],
        };
    }
}

#[derive(Debug)]
pub struct ColorTableColor(pub u8, pub u8, pub u8);

const MAX_COLOR_TABLE_SIZE: usize = 3 * (256);

#[derive(Debug, Copy, Clone)]
pub struct ColorTable {
    pub sorted: bool,
    pub size: u16,
    pub data: [u8; MAX_COLOR_TABLE_SIZE],
}

impl ColorTable {
    pub const BLOCK_SIZE: usize = size_of::<ColorTable>();
    pub const MAX_COLOR_TABLE_SIZE: usize = MAX_COLOR_TABLE_SIZE;

    pub fn calculate_size(size_flag: u8) -> u16 {
        3u16 * (2 << (size_flag as u16))
    }

    pub(crate) fn try_from_reader<R: Read>(
        reader: R,
        size_flag: u8,
        sorted: bool,
    ) -> Result<Self, ColorTableError> {
        let size = Self::calculate_size(size_flag);
        let mut table = Self {
            size,
            sorted,
            data: [0u8; MAX_COLOR_TABLE_SIZE],
        };

        if size as usize > MAX_COLOR_TABLE_SIZE {
            return Err(ColorTableError::TooLarge);
        }

        let mut limited_reader = reader.take(size as u64);
        match limited_reader.read(&mut table.data) {
            Ok(0) => Ok(table),
            Ok(n) if n == size as usize => Ok(table),
            Ok(n) if n < size as usize => Err(ColorTableError::NotEnoughData(n)),
            Ok(_) => return Err(ColorTableError::TooLarge),
            Err(err) => return Err(ColorTableError::Io(err)),
        }
    }

    pub fn get(&self, color_index: u8) -> Option<RGB<u8>> {
        if self.size < (color_index as u16 + 1) * 3 {
            return None;
        }
        let i = color_index as usize;
        let mut data = [0u8; 3];
        data[0] = self.data[i];
        data[1] = self.data[i + 1];
        data[2] = self.data[i + 2];
        Some(RGB::<u8>::from(data))
    }
}

////////////////////////
//  Image Descriptor  //
////////////////////////

pub const LOCAL_COLOR_TABLE_FLAG_POS: u8 = 0b11111111;
pub const LOCAL_COLOR_TABLE_FLAG_BITS: u8 = 1;
pub const INTERLACE_FLAG_POS: u8 = 0b01111111;
pub const INTERLACE_FLAG_BITS: u8 = 1;
pub const SORT_FLAG_POS: u8 = 0b00111111;
pub const SORT_FLAG_BITS: u8 = 1;
pub const RESERVED_POS: u8 = 0b00001111;
pub const RESERVED_BITS: u8 = 2;
pub const LOCAL_COLOR_TABLE_SIZE_POS: u8 = 0b0;
pub const LOCAL_COLOR_TABLE_SIZE_BITS: u8 = 3;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ImageDescriptor {
    pub image_left_position: u16,
    pub image_top_position: u16,
    pub image_width: u16,
    pub image_height: u16,
    pub flags: u8,
}

impl ImageDescriptor {
    pub const LOCAL_COLOR_TABLE_FLAG: u8 = 0b1000_0000;
    pub const LOCAL_COLOR_TABLE_FLAG_OFFSET: u8 = 7;
    pub const INTERLACE_FLAG: u8 = 0b0100_0000;
    pub const INTERLACE_FLAG_OFFSET: u8 = 6;
    pub const SORT_FLAG: u8 = 0b0010_0000;
    pub const SORT_FLAG_OFFSET: u8 = 5;
    //RESERVED
    pub const LOCAL_COLOR_TABLE_SIZE: u8 = 0b0000_0111;
    pub const LOCAL_COLOR_TABLE_SIZE_OFFSET: u8 = 0;

    pub const BLOCK_SIZE: usize = size_of::<Self>();

    pub fn local_color_table_flag(&self) -> bool {
        return self.flags & Self::LOCAL_COLOR_TABLE_FLAG != 0;
    }
    pub fn interlace_flag(&self) -> bool {
        return self.flags & Self::INTERLACE_FLAG != 0;
    }
    pub fn sort_flag(&self) -> bool {
        return self.flags & Self::SORT_FLAG != 0;
    }
    pub fn local_color_table_size(&self) -> u8 {
        return (self.flags & Self::LOCAL_COLOR_TABLE_SIZE) >> Self::LOCAL_COLOR_TABLE_SIZE_OFFSET;
    }
}

impl From<&[u8; ImageDescriptor::BLOCK_SIZE]> for ImageDescriptor {
    fn from(value: &[u8; ImageDescriptor::BLOCK_SIZE as usize]) -> Self {
        ImageDescriptor {
            image_left_position: u16::from_le_bytes([value[0], value[1]]),
            image_top_position: u16::from_le_bytes([value[2], value[3]]),
            image_width: u16::from_le_bytes([value[4], value[5]]),
            image_height: u16::from_le_bytes([value[6], value[7]]),
            flags: value[8],
        }
    }
}

//////////////////
//  Extensions  //
//////////////////

#[derive(Debug, Clone, Copy)]
pub struct PlainTextExtension {
    text_grid_left_position: u16,
    text_grid_top_position: u16,
    text_grid_width: u16,
    text_grid_height: u16,
    char_cell_width: u8,
    char_cell_height: u8,
    text_foreground_color_index: u8,
    text_background_color_index: u8,
}

impl PlainTextExtension {
    pub const BLOCK_SIZE: u8 = 13u8;
}

impl TryFrom<&[u8; PlainTextExtension::BLOCK_SIZE as usize]> for PlainTextExtension {
    type Error = BlockDecodeError;

    fn try_from(
        value: &[u8; PlainTextExtension::BLOCK_SIZE as usize],
    ) -> Result<PlainTextExtension, Self::Error> {
        if value[0] != PlainTextExtension::BLOCK_SIZE - 1 {
            return Err(BlockDecodeError::InvalidSize {
                expected: Self::BLOCK_SIZE - 1,
                found: value[0],
            });
        }
        Ok(PlainTextExtension {
            text_grid_left_position: u16::from_le_bytes([value[1], value[2]]),
            text_grid_top_position: u16::from_le_bytes([value[3], value[4]]),
            text_grid_width: u16::from_le_bytes([value[5], value[6]]),
            text_grid_height: u16::from_le_bytes([value[7], value[8]]),
            char_cell_width: value[9],
            char_cell_height: value[10],
            text_foreground_color_index: value[11],
            text_background_color_index: value[12],
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ApplicationExtension {
    application_identifier: [u8; 8],
    application_authentication_code: [u8; 3],
}

impl ApplicationExtension {
    pub const BLOCK_SIZE: u8 = 12u8;
}

impl TryFrom<&[u8; ApplicationExtension::BLOCK_SIZE as usize]> for ApplicationExtension {
    type Error = BlockDecodeError;

    fn try_from(
        value: &[u8; ApplicationExtension::BLOCK_SIZE as usize],
    ) -> Result<Self, Self::Error> {
        if value[0] != ApplicationExtension::BLOCK_SIZE - 1 {
            return Err(BlockDecodeError::InvalidSize {
                expected: Self::BLOCK_SIZE - 1,
                found: value[0],
            });
        }
        Ok(ApplicationExtension {
            application_identifier: value[1..9].try_into().unwrap(),
            application_authentication_code: value[9..12].try_into().unwrap(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GraphicControlExtension {
    flags: u8,
    delay_time: u16,
    transparent_color_index: u8,
}
impl GraphicControlExtension {
    pub const BLOCK_SIZE: u8 = 5u8;
}
impl TryFrom<&[u8; GraphicControlExtension::BLOCK_SIZE as usize]> for GraphicControlExtension {
    type Error = BlockDecodeError;

    fn try_from(
        value: &[u8; GraphicControlExtension::BLOCK_SIZE as usize],
    ) -> Result<Self, Self::Error> {
        if value[0] != GraphicControlExtension::BLOCK_SIZE - 1 {
            return Err(BlockDecodeError::InvalidSize {
                expected: Self::BLOCK_SIZE - 1,
                found: value[0],
            });
        }
        Ok(GraphicControlExtension {
            flags: value[1],
            delay_time: u16::from_le_bytes([value[2], value[3]]),
            transparent_color_index: value[4],
        })
    }
}
