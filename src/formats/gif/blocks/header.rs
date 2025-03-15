use std::{
    error::Error,
    fmt::Display,
    io::{self, Read},
};

use crate::formats::gif::{
    consts::{GIF_CONST_SIGNATURE, GIF_CONST_VERSION_87A, GIF_CONST_VERSION_89A},
    gif::Version,
};

use super::Block;

#[derive(Debug)]
pub enum HeaderDecodeError {
    /// Unknown Signature / Magic Number
    Signature(String),
    /// Unkown GIF Version. The GIF specification says the the decoder should try it's best anyway. But if
    /// decoder is in strict mode this error will be returned.
    Version(String),
}

impl Display for HeaderDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderDecodeError::Signature(found) => {
                write!(f, "Unknown gif signature. Expected 'GIF' found '{}'", found)
            }
            HeaderDecodeError::Version(found) => {
                write!(
                    f,
                    "Unkown gif version. Epected '87a' or '89a' found {}",
                    found
                )
            }
        }
    }
}
impl Error for HeaderDecodeError {}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub version: Version,
}

impl Block for Header {
    const BLOCK_SIZE: usize = 3 + 3;

    const VERSION: Version = Version::Version87a;
}

impl Header {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Header, HeaderParseError> {
        let mut buf = [0u8; Self::BLOCK_SIZE];
        reader.read_exact(&mut buf)?;

        Header::try_from(&buf)
    }
}

impl TryFrom<&[u8; Self::BLOCK_SIZE]> for Header {
    type Error = HeaderParseError;

    fn try_from(value: &[u8; Self::BLOCK_SIZE]) -> Result<Self, Self::Error> {
        let signature_bytes: &[u8; 3] = value[0..3]
            .try_into()
            .expect("expecting fixed range to fit into fixed array");

        if signature_bytes != GIF_CONST_SIGNATURE {
            return Err(HeaderParseError::Signature(
                std::str::from_utf8(signature_bytes)
                    .unwrap_or("")
                    .to_string(),
            ));
        }

        let version_bytes: &[u8; 3] = value[3..6]
            .try_into()
            .expect("expecting fixed range to fit into fixed array");

        let version = match version_bytes {
            GIF_CONST_VERSION_87A => Version::Version87a,
            GIF_CONST_VERSION_89A => Version::Version89a,
            _ => {
                return Err(HeaderParseError::Signature(
                    std::str::from_utf8(version_bytes).unwrap_or("").to_string(),
                ));
            }
        };

        Ok(Header { version })
    }
}

pub enum HeaderParseError {
    Signature(String),
    Version(String),
    Io(io::Error),
}

impl From<io::Error> for HeaderParseError {
    fn from(value: io::Error) -> Self {
        HeaderParseError::Io(value)
    }
}
