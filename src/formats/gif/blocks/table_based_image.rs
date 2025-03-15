//! yes I know it is technically multiple blocks, however they must follow each other and only the
//! ImageDescriptor has a Label, as such it is entierly plausible to let them act as one.

use std::io::{self, Read};

use crate::formats::gif::{
    blocks::decoding::{read_n_byte, read_subblock},
    gif::Version,
    lzw::{LZW, LZWDecodeError, LZWDecoder},
};

use super::{Block, BlockLabel, ColorTable, ColorTableParseError, LabeledBlock};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ImagePosition {
    pub left: u16,
    pub top: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct ImageDescriptor {
    image_left_position: u16,
    image_top_position: u16,
    image_width: u16,
    image_height: u16,
    flags: u8,
}

impl ImageDescriptor {
    const LOCAL_COLOR_TABLE_FLAG: u8 = 0b1000_0000;
    const LOCAL_COLOR_TABLE_FLAG_OFFSET: u8 = 7;
    const INTERLACE_FLAG: u8 = 0b0100_0000;
    const INTERLACE_FLAG_OFFSET: u8 = 6;
    const SORT_FLAG: u8 = 0b0010_0000;
    const SORT_FLAG_OFFSET: u8 = 5;
    //RESERVED
    const LOCAL_COLOR_TABLE_SIZE: u8 = 0b0000_0111;
    const LOCAL_COLOR_TABLE_SIZE_OFFSET: u8 = 0;

    pub fn local_color_table_flag(&self) -> bool {
        return self.flags & Self::LOCAL_COLOR_TABLE_FLAG != 0;
    }
    pub fn interlace_flag(&self) -> bool {
        return self.flags & Self::INTERLACE_FLAG != 0;
    }
    pub fn sort_flag(&self) -> bool {
        return self.flags & Self::SORT_FLAG != 0;
    }
    pub fn local_color_table_size_flag(&self) -> u8 {
        return (self.flags & Self::LOCAL_COLOR_TABLE_SIZE) >> Self::LOCAL_COLOR_TABLE_SIZE_OFFSET;
    }

    pub fn image_position(&self) -> ImagePosition {
        return ImagePosition {
            left: self.image_left_position,
            top: self.image_top_position,
        };
    }
    pub fn image_dim(&self) -> (u16, u16) {
        return (self.image_width, self.image_height);
    }
}

impl From<&[u8; TableBasedImage::BLOCK_SIZE]> for ImageDescriptor {
    fn from(value: &[u8; TableBasedImage::BLOCK_SIZE as usize]) -> Self {
        ImageDescriptor {
            image_left_position: u16::from_le_bytes([value[0], value[1]]),
            image_top_position: u16::from_le_bytes([value[2], value[3]]),
            image_width: u16::from_le_bytes([value[4], value[5]]),
            image_height: u16::from_le_bytes([value[6], value[7]]),
            flags: value[8],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableBasedImage {
    descriptor: ImageDescriptor,
    color_table: Option<ColorTable>,
    image_data: Vec<u8>,
}

impl Block for TableBasedImage {
    const BLOCK_SIZE: usize = 9;

    const VERSION: Version = Version::Version87a;
}

impl LabeledBlock for TableBasedImage {
    const LABEL: BlockLabel = BlockLabel::ImageDescriptor;
}

impl TableBasedImage {
    pub fn local_color_table(&self) -> Option<ColorTable> {
        self.color_table
    }
    pub fn local_color_table_ref(&self) -> Option<&ColorTable> {
        self.color_table.as_ref()
    }

    pub fn descriptor(&self) -> ImageDescriptor {
        self.descriptor
    }

    pub fn data(&self) -> &Vec<u8> {
        return &self.image_data;
    }

    pub fn parse<R: Read>(reader: &mut R) -> Result<TableBasedImage, TableBasedImageParseError> {
        //there is no size_flag
        const READ_SIZE: usize = TableBasedImage::BLOCK_SIZE;
        let buf: [u8; READ_SIZE] = read_n_byte(reader)?;

        let descriptor = ImageDescriptor::from(&buf);

        let mut color_table_opt: Option<ColorTable> = None;
        if descriptor.local_color_table_flag() == true {
            let size_flag = descriptor.local_color_table_size_flag();
            color_table_opt = Some(ColorTable::try_from_reader(
                reader,
                size_flag,
                descriptor.sort_flag(),
            )?);
        }

        //read lzw image
        let buf: [u8; 1] = read_n_byte(reader)?;
        let minimum_code_size = buf[0];

        let data = read_subblock(reader)?;
        let image_data = LZWDecoder::decode(&data[..], minimum_code_size)?;

        Ok(TableBasedImage {
            descriptor,
            color_table: color_table_opt,
            image_data,
        })
    }
}

pub enum TableBasedImageParseError {
    Io(io::Error),
    InvalidColorTable,
    InvalidLZWCode,
}

impl From<io::Error> for TableBasedImageParseError {
    fn from(value: io::Error) -> Self {
        TableBasedImageParseError::Io(value)
    }
}

impl From<ColorTableParseError> for TableBasedImageParseError {
    fn from(_value: ColorTableParseError) -> Self {
        TableBasedImageParseError::InvalidColorTable
    }
}

impl From<LZWDecodeError> for TableBasedImageParseError {
    fn from(value: LZWDecodeError) -> Self {
        TableBasedImageParseError::InvalidLZWCode
    }
}
