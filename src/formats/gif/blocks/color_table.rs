use core::fmt;
use std::{
    error::Error,
    fmt::Display,
    io::{self, Read},
};

use crate::{colors::rgb::RGB, pixel::Pixel};

const MAX_COLOR_TABLE_SIZE: usize = 256;
const MAX_COLOR_TABLE_DATA_SIZE: usize = 3 * MAX_COLOR_TABLE_SIZE;

pub trait ColorTableLookup {
    fn lookup(&self, color_index: u8) -> Option<RGB<u8>>;
    fn lookup_fallback(&self, color_index: u8) -> RGB<u8>;
}

#[derive(Debug, Clone, Copy)]
pub struct ColorTable {
    pub data: [RGB<u8>; MAX_COLOR_TABLE_SIZE],
    pub size: usize,
    pub sorted: bool,
}

impl ColorTable {
    /// Calculate the of the color table from the size_flag of the image/logical descriptor
    #[inline]
    pub fn calculate_size(size_flag: u8) -> usize {
        3 * (2 << (size_flag as usize + 1))
    }

    pub fn try_from_reader<R: Read>(
        reader: &mut R,
        size_flag: u8,
        sorted: bool,
    ) -> Result<ColorTable, ColorTableParseError> {
        let mut table = ColorTable {
            size: Self::calculate_size(size_flag),
            sorted,
            data: [RGB::default(); MAX_COLOR_TABLE_SIZE],
        };

        if table.size > MAX_COLOR_TABLE_SIZE {
            return Err(ColorTableParseError::TooLarge);
        }

        let mut color_table_data = [0u8; MAX_COLOR_TABLE_DATA_SIZE];

        let mut limited_reader = reader.take(table.size as u64);
        match limited_reader.read(&mut color_table_data) {
            Ok(0) => Ok(table), // this means read to end
            Ok(n) if n == table.size => {
                // Fill it with RGB values from the data
                for i in (0..n).step_by(3) {
                    let index = i / 3;
                    table.data[index] = RGB::from([
                        color_table_data[i],
                        color_table_data[i + 1],
                        color_table_data[i + 2],
                    ]);
                }
                Ok(table)
            }
            Ok(n) if n < table.size => Err(ColorTableParseError::NotEnoughData),
            Ok(_) => {
                panic!("color table reader read more than possible")
            }
            Err(err) => Err(ColorTableParseError::Io(err)),
        }
    }
}

impl ColorTableLookup for ColorTable {
    fn lookup(&self, color_index: u8) -> Option<RGB<u8>> {
        if color_index as usize >= self.size {
            return None;
        }

        Some(self.data[color_index as usize])
    }

    fn lookup_fallback(&self, color_index: u8) -> RGB<u8> {
        if color_index as usize >= self.size {
            return RGB::default();
        }

        self.data[color_index as usize]
    }
}

impl Default for ColorTable {
    fn default() -> Self {
        Self {
            data: [RGB::<u8>::DEFAULT_MIN_VALUE; MAX_COLOR_TABLE_SIZE],
            size: MAX_COLOR_TABLE_SIZE,
            sorted: false,
        }
    }
}

impl fmt::Display for ColorTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[", self.size)?;

        let mut iter = self.data[..self.size].iter().peekable();
        while let Some(col) = iter.next() {
            write!(f, "{}", col)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

#[derive(Debug)]
pub enum ColorTableParseError {
    TooLarge,
    NotEnoughData,
    Io(io::Error),
}

impl Display for ColorTableParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorTableParseError::TooLarge => write!(
                f,
                "size specified was larger than the maximum color table size"
            ),
            ColorTableParseError::NotEnoughData => {
                write!(f, "not enough data could be read to fill the color table")
            }
            ColorTableParseError::Io(error) => {
                write!(f, "io error during color table fill {}", error)
            }
        }
    }
}
impl Error for ColorTableParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ColorTableParseError::Io(error) => Some(error),
            _ => None,
        }
    }
}
