use std::{
    io::{self, Read},
    string::FromUtf8Error,
};

use crate::formats::gif::gif::Version;

use super::{Block, BlockLabel, LabeledBlock, decoding::read_subblock};

#[derive(Debug, Clone)]
pub struct PlainTextExtension {
    text_grid_left_position: u16,
    text_grid_top_position: u16,
    text_grid_width: u16,
    text_grid_height: u16,
    char_cell_width: u8,
    char_cell_height: u8,
    text_foreground_color_index: u8,
    text_background_color_index: u8,
    plain_text_data: String,
}

// specified by the gif-spec p. 21
impl Block for PlainTextExtension {
    const BLOCK_SIZE: usize = 12;

    const VERSION: Version = Version::Version89a;
}
impl LabeledBlock for PlainTextExtension {
    const LABEL: super::BlockLabel = BlockLabel::PlainTextExtension;
}

impl PlainTextExtension {
    /// parses a PlainTextExtension from after the plain text extension label.
    /// This method will allow all correct utf-8 strings as data. Use `parse_strict`
    /// to allow only valid 7-bit printable ascii chars like specified by the gif spec p. 21
    ///
    /// This will also not validate any character position on the logical screen, thus they could
    /// be partially or fully out of bounds
    pub fn parse<R: Read>(
        reader: &mut R,
    ) -> Result<PlainTextExtension, PlainTextExtensionParseError> {
        let mut parameters = [0u8; PlainTextExtension::BLOCK_SIZE + 1]; // + block_size
        reader.read_exact(&mut parameters)?;

        if parameters[0] as usize != Self::BLOCK_SIZE {
            return Err(PlainTextExtensionParseError::InvalidBlockSize {
                expected: Self::BLOCK_SIZE,
                found: parameters[0] as usize,
            });
        }

        // we could check position and width etc. of the characters, but since GIF is
        // largely best effort we will do it during the actual rendering of the block.
        // and cut of any parts that are out of bounds

        let plain_text_data_bytes = read_subblock(reader)?;

        // !!WARNING: We consciously allow all utf characters here.
        let plain_text_data = String::from_utf8(plain_text_data_bytes)?;

        Ok(PlainTextExtension {
            text_grid_left_position: u16::from_le_bytes([parameters[1], parameters[2]]),
            text_grid_top_position: u16::from_le_bytes([parameters[3], parameters[4]]),
            text_grid_width: u16::from_le_bytes([parameters[5], parameters[6]]),
            text_grid_height: u16::from_le_bytes([parameters[7], parameters[8]]),
            char_cell_width: parameters[9],
            char_cell_height: parameters[10],
            text_foreground_color_index: parameters[11],
            text_background_color_index: parameters[12],
            plain_text_data,
        })
    }

    pub fn parse_strict<R: Read>(
        reader: &mut R,
    ) -> Result<PlainTextExtension, PlainTextExtensionParseError> {
        let res = PlainTextExtension::parse(reader)?;
        if !res.plain_text_data.is_ascii() {
            return Err(PlainTextExtensionParseError::InvalidASCII);
        }
        Ok(res)
    }
}

pub enum PlainTextExtensionParseError {
    Io(io::Error),
    InvalidBlockSize { expected: usize, found: usize },
    InvalidASCII,
}

impl From<io::Error> for PlainTextExtensionParseError {
    fn from(arguments: io::Error) -> Self {
        return PlainTextExtensionParseError::Io(arguments);
    }
}

impl From<FromUtf8Error> for PlainTextExtensionParseError {
    fn from(_arguments: FromUtf8Error) -> Self {
        PlainTextExtensionParseError::InvalidASCII
    }
}
