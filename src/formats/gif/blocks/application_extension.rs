use std::io::{self, Read};

use crate::formats::gif::gif::Version;

use super::{decoding::read_subblock, Block, BlockLabel, LabeledBlock};

#[derive(Debug, Clone)]
pub struct ApplicationExtension {
    application_identifier: [u8; 8],
    application_code: [u8; 3],
    application_data: Box<[u8]>,
}

impl Block for ApplicationExtension {
    const BLOCK_SIZE: usize = 11;

    const VERSION: Version = Version::Version89a;
}

impl LabeledBlock for ApplicationExtension {
    const LABEL: super::BlockLabel = BlockLabel::ApplicationExtension;
}

impl ApplicationExtension {
    pub fn parse<R: Read>(
        mut reader: R,
    ) -> Result<ApplicationExtension, ApplicationExtensionParseError> {
        const READ_SIZE: usize = ApplicationExtension::BLOCK_SIZE + 1;
        let mut buf = [0u8; READ_SIZE];
        reader.read_exact(&mut buf)?;

        if buf[0] as usize != ApplicationExtension::BLOCK_SIZE {
            return Err(ApplicationExtensionParseError::InvalidBlockSize);
        }

        let application_data = read_subblock(reader)?.into_boxed_slice();
        Ok(ApplicationExtension {
            application_identifier: buf[1..10]
                .try_into()
                .expect("cannot transform Range to const array"),
            application_code: buf[10..]
                .try_into()
                .expect("cannot transform Range to const array"),
            application_data,
        })
    }
}

pub enum ApplicationExtensionParseError {
    Io(io::Error),
    InvalidBlockSize,
}

impl From<io::Error> for ApplicationExtensionParseError {
    fn from(arguments: io::Error) -> Self {
        return ApplicationExtensionParseError::Io(arguments);
    }
}
