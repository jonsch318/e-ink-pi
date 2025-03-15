use super::{BlockLabel, BlockLabelType};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ControlBlocks {
    GraphicsControlExtension = BlockLabel::GraphicControlExtension as u8,
    UnknownBlock = BlockLabel::Trailer as u8,
}

impl TryFrom<BlockLabel> for ControlBlocks {
    type Error = ();

    fn try_from(value: BlockLabel) -> Result<Self, Self::Error> {
        if BlockLabelType::from(value) != BlockLabelType::Control {
            return Err(());
        }

        match value {
            BlockLabel::GraphicControlExtension => Ok(ControlBlocks::GraphicsControlExtension),
            _ => Ok(ControlBlocks::UnknownBlock),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum GraphicRenderingBlocks {
    TableBasedImage = BlockLabel::ImageDescriptor as u8,
    PlainTextExtension = BlockLabel::PlainTextExtension as u8,
    UnknownBlock = BlockLabelType::Trailer as u8,
}

impl GraphicRenderingBlocks {
    pub fn is_extension(self) -> bool {
        return self == GraphicRenderingBlocks::TableBasedImage;
    }
}

impl TryFrom<BlockLabel> for GraphicRenderingBlocks {
    type Error = ();

    fn try_from(value: BlockLabel) -> Result<Self, Self::Error> {
        if BlockLabelType::from(value) != BlockLabelType::Graphic {
            return Err(());
        }

        Ok(match value {
            BlockLabel::ImageDescriptor => GraphicRenderingBlocks::TableBasedImage,
            BlockLabel::PlainTextExtension => GraphicRenderingBlocks::PlainTextExtension,
            _ => GraphicRenderingBlocks::UnknownBlock,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum SpecialPurposeBlocks {
    UnknownBlock = BlockLabel::Trailer as u8,
    CommentExtension = BlockLabel::CommentExtension as u8,
    ApplicationExtension = BlockLabel::ApplicationExtension as u8,
}

impl SpecialPurposeBlocks {
    pub fn is_extension(self) -> bool {
        return false;
    }
}

impl TryFrom<BlockLabel> for SpecialPurposeBlocks {
    type Error = ();

    fn try_from(value: BlockLabel) -> Result<Self, Self::Error> {
        if BlockLabelType::from(value) != BlockLabelType::SpecialPurpose {
            return Err(());
        }

        Ok(match value {
            BlockLabel::CommentExtension => SpecialPurposeBlocks::CommentExtension,
            BlockLabel::ApplicationExtension => SpecialPurposeBlocks::ApplicationExtension,
            _ => SpecialPurposeBlocks::UnknownBlock,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ExtensionType {
    GraphicControlExtension = BlockLabel::GraphicControlExtension as u8,
    ApplicationExtension = BlockLabel::ApplicationExtension as u8,
    CommentExtension = BlockLabel::CommentExtension as u8,
    PlainTextExtension = BlockLabel::PlainTextExtension as u8,
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
