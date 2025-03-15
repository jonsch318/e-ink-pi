//! Good error types for all. I strive to be concise and enable easy debugging while not leaking
//! implementation specific without contexts. Since I cannot use the "anyerror" create this will be
//! more verbose than it has to be.

use core::fmt::Display;
use std::{error::Error, io};

use super::blocks::{
    Block, ColorTableParseError, GraphicControlExtension, GraphicControlExtensionParseError,
    HeaderParseError, LogicalScreenDescriptorParseError, TableBasedImageParseError,
};

#[derive(Debug)]
pub enum GIFParseError {
    Io { reason: String, cause: io::Error },
    UnknownSignature(String),
    UnknownVersion(String),
    UnexpectedBlockDiscriminant(u8),
    UnexpectedExtensionLabel(u8),
    UnexpectedBlockSize { got: u8, expected: u8 },
    InvalidBlockTerminator,
    InvalidColorTable(String),
    InvalidLZWCode,
    ImageDataError,
}

impl Display for GIFParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GIFParseError::Io { reason, cause } => write!(f, "io error {}: {}", reason, cause),
            GIFParseError::UnknownSignature(sig) => write!(f, "unknown signature: {}", sig),
            GIFParseError::UnknownVersion(ver) => write!(f, "unknown version: {}", ver),
            GIFParseError::UnexpectedBlockDiscriminant(_) => {
                write!(f, "unexpected block discriminant")
            }
            GIFParseError::UnexpectedExtensionLabel(_) => write!(f, "unexpected extension label"),
            GIFParseError::UnexpectedBlockSize { got, expected } => write!(
                f,
                "unexpected block size expected {} but got {}",
                expected, got
            ),
            GIFParseError::InvalidBlockTerminator => write!(f, "invalid block terminator"),
            GIFParseError::InvalidColorTable(_) => write!(f, "invalid color table"),
            GIFParseError::InvalidLZWCode => write!(f, "invalid lzw code"),
            GIFParseError::ImageDataError => write!(f, "invalid image data"),
        }
    }
}

impl Error for GIFParseError {}

impl From<HeaderParseError> for GIFParseError {
    fn from(value: HeaderParseError) -> Self {
        match value {
            HeaderParseError::Signature(str) => GIFParseError::UnknownSignature(str),
            HeaderParseError::Version(str) => GIFParseError::UnknownVersion(str),
            HeaderParseError::Io(error) => GIFParseError::Io {
                reason: "io error during header decode".to_string(),
                cause: error,
            },
        }
    }
}
impl From<LogicalScreenDescriptorParseError> for GIFParseError {
    fn from(value: LogicalScreenDescriptorParseError) -> Self {
        match value {
            LogicalScreenDescriptorParseError::Io(error) => GIFParseError::Io {
                reason: "io error during header decode".to_string(),
                cause: error,
            },
        }
    }
}

impl From<ColorTableParseError> for GIFParseError {
    fn from(value: ColorTableParseError) -> Self {
        match value {
            ColorTableParseError::TooLarge => {
                GIFParseError::InvalidColorTable("color table to too small".to_string())
            }
            ColorTableParseError::NotEnoughData => {
                GIFParseError::InvalidColorTable("color table to small".to_string())
            }
            ColorTableParseError::Io(error) => GIFParseError::Io {
                reason: "io error during color table parse".to_string(),
                cause: error,
            },
        }
    }
}

impl From<TableBasedImageParseError> for GIFParseError {
    fn from(value: TableBasedImageParseError) -> Self {
        match value {
            TableBasedImageParseError::Io(error) => GIFParseError::Io {
                reason: "io error during image data block read".to_string(),
                cause: error,
            },
            TableBasedImageParseError::InvalidColorTable => {
                GIFParseError::InvalidColorTable("invalid local color table".to_string())
            }
            TableBasedImageParseError::InvalidLZWCode => GIFParseError::InvalidLZWCode,
        }
    }
}

impl From<io::Error> for GIFParseError {
    fn from(value: io::Error) -> Self {
        GIFParseError::Io {
            reason: "io error occurred at unknown location".to_string(),
            cause: value,
        }
    }
}

impl From<GraphicControlExtensionParseError> for GIFParseError {
    fn from(value: GraphicControlExtensionParseError) -> Self {
        match value {
            GraphicControlExtensionParseError::Io(error) => GIFParseError::Io {
                reason:
                    "io error (likely EOF) during graphic control extension block reading/parsing"
                        .to_string(),
                cause: error,
            },
            GraphicControlExtensionParseError::InvalidBlockSize(got) => {
                GIFParseError::UnexpectedBlockSize {
                    got,
                    expected: GraphicControlExtension::BLOCK_SIZE as u8,
                }
            }
            GraphicControlExtensionParseError::InvalidBlockTerminator => {
                GIFParseError::InvalidBlockTerminator
            }
        }
    }
}

#[derive(Debug)]
pub enum GIFDecodeError {
    /// The decode could not complete due to an io error. Most likely a ErrorKind::UnexpectedEOF
    /// but for debugging the whole Error is exported via cause
    Io {
        reason: String,
        cause: Option<std::io::Error>,
    },
    UnkownSignature(String),
    UnkownVersion(String),
    UnknownBlockType(u8),
    InvalidBlockSize {
        //block_type:
        expected: u8,
        size: u8,
    },
    ///encountered a block of some type the decoder cannot parse
    UnexpectedBlockType {
        reason: String,
        // expected: GIFGrammarState,
        // found: GIFGrammarState,
    },
    ///something with the actual parsing of the image
    ImageParse {
        reason: String,
        cause: ImageParseError,
    },
    Animation {
        reason: String,
    },
}

impl Display for GIFDecodeError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GIFDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            //TODO: this is fucking awfull
            GIFDecodeError::Io { reason: _, cause } => {
                cause.as_ref().map(|e| e as &(dyn Error + 'static))
            }
            GIFDecodeError::ImageParse { reason: _, cause } => Some(cause),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ImageParseError {
    LZWError { reason: String },
    ColorOutOfBounds(u8),
    DataOutOfBounds(u16, u16),
}

impl Display for ImageParseError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ImageParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ImageParseError::LZWError { reason: _ } => None,
            ImageParseError::ColorOutOfBounds(_) => None,
            ImageParseError::DataOutOfBounds(_, _) => None,
        }
    }
}
