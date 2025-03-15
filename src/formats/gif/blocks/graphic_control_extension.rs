use std::io::{self, Read};

use crate::formats::gif::gif::{DisposalMethod, Version};

use super::{Block, BlockLabel, LabeledBlock};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GraphicControlExtension {
    flags: u8,
    delay_time: u16,
    transparent_color_index: u8,
}

impl Block for GraphicControlExtension {
    const BLOCK_SIZE: usize = 4;

    const VERSION: Version = Version::Version89a;
}

impl LabeledBlock for GraphicControlExtension {
    const LABEL: BlockLabel = BlockLabel::GraphicControlExtension;
}

const DEFAULT_DELAY_TIME: u16 = 0u16;

const DEFAULT_DISPOSAL_METHOD: u8 = 1u8;
const DEFAULT_USER_INPUT_FLAG: u8 = 0u8;
const DEFAULT_TRANSPARENT_COLOR_FLAG: u8 = 0u8;
const DEFAULT_TRANSPARENT_COLOR_INDEX: u8 = 0u8;

impl Default for GraphicControlExtension {
    fn default() -> Self {
        GraphicControlExtension {
            delay_time: DEFAULT_DELAY_TIME,
            flags: (DEFAULT_DISPOSAL_METHOD << 2)
                | (DEFAULT_USER_INPUT_FLAG << 1)
                | (DEFAULT_TRANSPARENT_COLOR_FLAG),
            transparent_color_index: DEFAULT_TRANSPARENT_COLOR_INDEX,
        }
    }
}

impl GraphicControlExtension {
    pub fn parse<R: Read>(
        reader: &mut R,
    ) -> Result<GraphicControlExtension, GraphicControlExtensionParseError> {
        const READ_SIZE: usize = GraphicControlExtension::BLOCK_SIZE + 1 + 1; // +size flag +terminator
        let mut buf = [0u8; READ_SIZE];
        reader.read_exact(&mut buf)?;

        if buf[0] as usize != GraphicControlExtension::BLOCK_SIZE {
            return Err(GraphicControlExtensionParseError::InvalidBlockSize(buf[0]));
        }

        if buf[5] != 0u8 {
            return Err(GraphicControlExtensionParseError::InvalidBlockTerminator);
        }

        Ok(GraphicControlExtension {
            flags: buf[1],
            delay_time: u16::from_le_bytes([buf[2], buf[3]]),
            transparent_color_index: buf[4],
        })
    }

    //Flags
    //  RESERVED
    const DISPOSAL_METHOD: u8 = 0b11100;
    const DISPOSAL_METHOD_OFFSET: u8 = 2;
    const USER_INPUT_FLAG: u8 = 0b10;
    const USER_INPUT_FLAG_OFFSET: u8 = 1;
    const TRANSPARENT_COLOR_FLAG: u8 = 0b01;
    const TRANSPARENT_COLOR_FLAG_OFFSET: u8 = 0;

    pub fn transparent_color_index(&self) -> Option<u8> {
        if !self.has_transparent_color() {
            return None;
        }
        Some(self.transparent_color_index)
    }

    pub fn delay_time(&self) -> u16 {
        self.delay_time
    }

    pub fn disposal_method(&self) -> DisposalMethod {
        DisposalMethod::from((self.flags & Self::DISPOSAL_METHOD) >> Self::DISPOSAL_METHOD_OFFSET)
    }
    pub fn user_input_flag(&self) -> bool {
        (self.flags & Self::USER_INPUT_FLAG) != 0
    }
    pub fn has_transparent_color(&self) -> bool {
        (self.flags & Self::TRANSPARENT_COLOR_FLAG) != 0
    }
}

pub enum GraphicControlExtensionParseError {
    Io(io::Error),
    InvalidBlockSize(u8),
    InvalidBlockTerminator,
}

impl From<io::Error> for GraphicControlExtensionParseError {
    fn from(arguments: io::Error) -> Self {
        GraphicControlExtensionParseError::Io(arguments)
    }
}
