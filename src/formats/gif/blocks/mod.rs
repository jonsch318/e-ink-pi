mod application_extension;
mod categories;
mod color_table;
mod comment_extension;
mod decoding;
mod graphic_control_extension;
mod header;
mod logical_screen_descriptor;
mod plain_text_extension;
mod table_based_image;

use super::gif::Version;
//re-export blocks
pub use {
    application_extension::*, categories::*, color_table::*, comment_extension::*,
    graphic_control_extension::*, header::*, logical_screen_descriptor::*, plain_text_extension::*,
    table_based_image::*,
};

pub trait Block {
    const BLOCK_SIZE: usize;
    const VERSION: Version;
}

pub trait LabeledBlock: Block {
    const LABEL: BlockLabel;
    fn block_label_type() -> BlockLabelType {
        return BlockLabelType::from(Self::LABEL);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum BlockSeparator {
    Image = BlockLabel::ImageDescriptor as u8,
    Trailer = BlockLabel::Trailer as u8,
    Extension = 0x21,
}

impl BlockSeparator {
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0x2c => Some(BlockSeparator::Image),
            0x3B => Some(BlockSeparator::Trailer),
            0x21 => Some(BlockSeparator::Extension),
            _ => None,
        }
    }

    pub fn can_be_type(&self, check: BlockLabelType) -> bool {
        match self {
            BlockSeparator::Image if check == BlockLabelType::Graphic => true,
            BlockSeparator::Trailer if check == BlockLabelType::Trailer => true,
            BlockSeparator::Extension if check != BlockLabelType::Trailer => true, // an extension
            // can everything except a trailer
            _ => false,
        }
    }
}

impl From<BlockLabel> for BlockSeparator {
    fn from(value: BlockLabel) -> Self {
        match value {
            BlockLabel::ImageDescriptor => BlockSeparator::Image,
            BlockLabel::Trailer => BlockSeparator::Trailer,
            _ => BlockSeparator::Extension,
        }
    }
}

impl TryFrom<u8> for BlockSeparator {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        BlockSeparator::try_from_u8(value).ok_or(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum BlockLabelType {
    Graphic = 0x00,
    Trailer = 0x3B,
    Control = 0x80,
    SpecialPurpose = 0xFA,
}

impl From<u8> for BlockLabelType {
    fn from(value: u8) -> Self {
        match value {
            0x3B => BlockLabelType::Trailer,
            0x00..=0x7F => BlockLabelType::Graphic,
            0x80..=0xF9 => BlockLabelType::Control,
            0xFA..=0xFF => BlockLabelType::SpecialPurpose,
        }
    }
}
impl From<BlockLabel> for BlockLabelType {
    fn from(value: BlockLabel) -> Self {
        match value {
            BlockLabel::ImageDescriptor => BlockLabelType::Graphic,
            BlockLabel::PlainTextExtension => BlockLabelType::Graphic,
            BlockLabel::GraphicControlExtension => BlockLabelType::Control,
            BlockLabel::Trailer => BlockLabelType::Trailer,
            BlockLabel::CommentExtension => BlockLabelType::SpecialPurpose,
            BlockLabel::ApplicationExtension => BlockLabelType::SpecialPurpose,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BlockLabel {
    // Graphic
    ImageDescriptor = 0x2C,
    PlainTextExtension = 0x01,
    // Control
    GraphicControlExtension = 0xF9,

    // SpecialPurpose
    Trailer = 0x3B,
    CommentExtension = 0xFE,
    ApplicationExtension = 0xFF,
}

impl BlockLabel {
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0x2C => Some(BlockLabel::ImageDescriptor),
            0x01 => Some(BlockLabel::PlainTextExtension),
            0xF9 => Some(BlockLabel::GraphicControlExtension),
            0x3B => Some(BlockLabel::Trailer),
            0xFE => Some(BlockLabel::CommentExtension),
            0xFF => Some(BlockLabel::ApplicationExtension),
            _ => None,
        }
    }
    pub fn is_extension(&self) -> bool {
        match self {
            BlockLabel::PlainTextExtension
            | BlockLabel::GraphicControlExtension
            | BlockLabel::CommentExtension
            | BlockLabel::ApplicationExtension => true,
            _ => false,
        }
    }
    pub fn is_of_type(&self, label_type: BlockLabelType) -> bool {
        return BlockLabelType::from(*self) == label_type;
    }
    pub fn is_of_seperator_type(&self, descriminant: BlockSeparator) -> bool {
        return BlockSeparator::from(*self) == descriminant;
    }
}

impl TryFrom<u8> for BlockLabel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        BlockLabel::try_from_u8(value).ok_or(())
    }
}
