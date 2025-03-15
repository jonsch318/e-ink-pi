use std::{
    io::{self, Read},
    string::FromUtf8Error,
};

use crate::formats::gif::gif::Version;

use super::{Block, BlockLabel, LabeledBlock, decoding::read_subblock};

pub struct CommentExtension {
    comment_data: String,
}

impl Block for CommentExtension {
    const BLOCK_SIZE: usize = 0;

    const VERSION: Version = Version::Version89a;
}

impl LabeledBlock for CommentExtension {
    const LABEL: super::BlockLabel = BlockLabel::CommentExtension;
}

impl CommentExtension {
    pub fn parse<R: Read>(reader: &mut R) -> Result<CommentExtension, CommentExtensionParseError> {
        let comment_data_bytes = read_subblock(reader)?;
        Ok(CommentExtension {
            comment_data: String::from_utf8(comment_data_bytes)?,
        })
    }
}

pub enum CommentExtensionParseError {
    Io(io::Error),
    InvalidASCII,
}

impl From<io::Error> for CommentExtensionParseError {
    fn from(arguments: io::Error) -> Self {
        return CommentExtensionParseError::Io(arguments);
    }
}

impl From<FromUtf8Error> for CommentExtensionParseError {
    fn from(_arguments: FromUtf8Error) -> Self {
        CommentExtensionParseError::InvalidASCII
    }
}
