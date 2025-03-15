// This is a gif compatible lzw variable length decoder. The logic of this is designed to work
// with a given input stream and a resulting decoded buffer.
// This lets us easily safe memory and still good decompression rates and should be fine enough for gif decoding.

use std::{
    error::Error,
    fmt::Display,
    io::{self, BufRead},
};

use crate::formats::bits::{BitReader, BitsReadError, LittleEndianReader};

/// The value to use on codes. Due to variable length coding anywhere from 2-12 bits are used.
type Code = u16;

#[derive(Debug)]
pub enum LZWDecodeError {
    UnexpectedEOF,
    InvalidMinimumCodeSize,
    TooLargeCode { found: Code, table_size: Code },
    PrefixMismatch { reason: &'static str },
    BitRead { source: String, cause: io::Error },
}

impl Display for LZWDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LZWDecodeError::UnexpectedEOF => write!(f, "unexpected end of file"),
            LZWDecodeError::InvalidMinimumCodeSize => {
                write!(f, "minimum_code_size <= 1 or >= 11 invalid")
            }
            LZWDecodeError::TooLargeCode { found, table_size } => {
                write!(f, "code {} >(=) table_size {}", found, table_size)
            }
            LZWDecodeError::PrefixMismatch { reason } => {
                write!(f, "internal state mismatch {}", reason)
            }
            LZWDecodeError::BitRead { source, cause: _ } => write!(f, "bit read error {}", source),
        }
    }
}

impl Error for LZWDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LZWDecodeError::BitRead { source: _, cause } => Some(cause),
            _ => None,
        }
    }
}

impl From<LZWDecodeError> for io::Error {
    fn from(value: LZWDecodeError) -> Self {
        match value {
            LZWDecodeError::UnexpectedEOF => io::Error::from(io::ErrorKind::UnexpectedEof),
            LZWDecodeError::InvalidMinimumCodeSize => {
                io::Error::new(io::ErrorKind::InvalidInput, value.to_string())
            }
            LZWDecodeError::TooLargeCode {
                found: _,
                table_size: _,
            } => io::Error::new(io::ErrorKind::InvalidData, value.to_string()),
            LZWDecodeError::PrefixMismatch { reason: _ } => io::Error::other(value.to_string()),
            LZWDecodeError::BitRead { source: _, cause } => cause,
        }
    }
}

const MAX_TABLE_SIZE: usize = 4096;
const MAX_CODE_VALUE: Code = 0xFFF; // 12 bit is the max code

// codebook logic
// To safe memory and keep high speeds we use references to the already decoded output of old codes.
// Thus we dont need to safe a copy of each decoded word but we also dont need to construct it
// character by character. Of course this only works due to our buffered output
#[derive(Debug, Copy, Clone)]
struct CodeBook {
    /// prefix is the reference to a slice of the ouput. so that `prefix[code] ++ suffix[code] =
    /// decoded_word`.
    /// The lifetime of course needs to be long enough so that each reference is valid until the
    /// codebook is not used any more
    prefix: [Option<(usize, usize)>; MAX_TABLE_SIZE],
    /// suffix character, the addition of the 'old codeword'. `prefix[code] ++ suffix[code] =
    /// decoded_word`
    suffix: [u8; MAX_TABLE_SIZE],
    /// the index to the next incomplete codeword. next_index - 1 is the last translated code.
    next_index: Option<usize>,
    /// the starting dictionary size: all bits from 0..(1<<minimum_code_size) are given
    //minimum_code_size: Code,
    /// constant for some minimum_code_size explained in gif specs
    clear_code: Code,
    /// constant for some minimum_code_size explained in gif specs
    end_of_information_code: Code,
}
impl CodeBook {
    pub fn new(minimum_code_size: Code) -> Self {
        let clear_code: Code = 1u16 << minimum_code_size;
        let mut res = Self {
            prefix: [None; MAX_TABLE_SIZE],
            suffix: [0u8; MAX_TABLE_SIZE],
            //minimum_code_size,
            clear_code,
            end_of_information_code: clear_code + 1,
            next_index: None,
        };
        res.init();
        res
    }
    fn init(&mut self) {
        for code in 0..self.clear_code {
            self.suffix[code as usize] = code as u8;
            self.prefix[code as usize] = None;
        }
    }
    fn increment_next_index(&mut self) -> bool {
        if self.is_full() {
            return false;
        }
        match self.next_index {
            None => {
                self.next_index = Some(self.clear_code as usize + 2);
                false
            }
            Some(x) => {
                let new = x + 1;
                self.next_index = Some(new);
                new.is_power_of_two()
            }
        }
    }
    fn is_full(&self) -> bool {
        self.next_index
            .is_some_and(|x| x >= MAX_CODE_VALUE as usize)
    }
    fn clear(&mut self) {
        self.next_index = None;
        //ASK:do i need this?
        for _ in (self.clear_code as usize + 2)..MAX_TABLE_SIZE {
            self.prefix = [None; MAX_TABLE_SIZE];
        }
    }
    fn size(&self) -> u16 {
        u16::try_from(self.next_index.unwrap_or(0)).unwrap()
    }
}

pub struct LZWDecoder {}

pub trait LZW {
    fn decode<R: BufRead>(data: R, minimum_code_size: u8) -> Result<Vec<u8>, LZWDecodeError>;
}
impl LZWDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LZWDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl LZW for LZWDecoder {
    fn decode<R: BufRead>(data: R, minimum_code_size: u8) -> Result<Vec<u8>, LZWDecodeError> {
        // create codebook
        let mut codebook = CodeBook::new(minimum_code_size.into());
        let mut output: Vec<u8> = Vec::new();
        let mut bit_reader = LittleEndianReader::new(data);

        if minimum_code_size <= 1 || minimum_code_size >= 11 {
            return Err(LZWDecodeError::InvalidMinimumCodeSize);
        }

        // the code are variable in length starting from `minimum_code_size + 1` (due to
        // clear code and eoi code) up to 12 bits per
        // code. Whenever a value exceeds the code_bits size it is increased by one.
        let mut current_code_bits = minimum_code_size + 1;

        loop {
            let code: Code = match bit_reader.read_n::<Code>(current_code_bits) {
                Ok(x) => x,
                Err(BitsReadError::UnexpectedEOF) => return Err(LZWDecodeError::UnexpectedEOF),
                Err(err) => {
                    return Err(LZWDecodeError::BitRead {
                        source: err.to_string(),
                        cause: err.into(),
                    });
                }
            };

            let code_index = code as usize;

            /////////////////////
            //  Control Codes  //
            /////////////////////

            if code == codebook.end_of_information_code {
                return Ok(output);
            }
            if code == codebook.clear_code {
                current_code_bits = minimum_code_size + 1;
                codebook.clear();
                continue;
            }

            if codebook.is_full() {
                if codebook.next_index.is_some_and(|x| code_index >= x)
                    || (codebook.next_index.is_none() && code >= codebook.clear_code)
                {
                    return Err(LZWDecodeError::TooLargeCode {
                        found: code,
                        table_size: codebook.size(),
                    });
                }
                let last_char = codebook.suffix[code_index];
                if let Some((offset, length)) = codebook.prefix[code_index] {
                    let slice = &output[offset..offset + length].to_vec();
                    output.extend(slice);
                }
                output.push(last_char);
                continue;
            }

            //////////////////////////
            //  Normal Decode Flow  //
            //////////////////////////
            // The codebook is not full yet so we create a new code each time we read a code.

            let mut current_decoded_length = 1usize; // we need to know what the length of
            // the code is, that we are currently decoding, to generate the next one.
            if codebook.next_index.is_none() {
                //This is the first code after a new lzw stream or a clear code. The only possible
                //reads are predefined codes. After this we create the first uncomplete code
                if code >= codebook.clear_code {
                    return Err(LZWDecodeError::TooLargeCode {
                        found: code,
                        table_size: codebook.size(),
                    });
                }
                output.push(codebook.suffix[code_index]);
            } else {
                let current_incomplete_code = codebook.next_index.unwrap();
                match code_index.cmp(&current_incomplete_code) {
                    std::cmp::Ordering::Less => {
                        // we have seen this code already. Now we must reconstruct it and paste it to
                        // output
                        let last_char = codebook.suffix[code_index];
                        let mut first_char = last_char; // this is correct in case we are in a predefined code

                        if let Some((offset, length)) = codebook.prefix[code_index] {
                            first_char = output[offset];
                            let slice = &output[offset..offset + length].to_vec();
                            output.extend(slice);
                            current_decoded_length += length;
                        }

                        output.push(last_char);
                        codebook.suffix[current_incomplete_code] = first_char;
                        //complete last code
                    }
                    std::cmp::Ordering::Equal => {
                        // code we have begun in the last loop iteration, thus it is incomplete and
                        // is missing the last_char.
                        // Steps: get the last (uncompleted) prefix ++ first char of last prefix.
                        match codebook.prefix[current_incomplete_code] {
                            None => {
                                return Err(LZWDecodeError::PrefixMismatch {
                                    reason: "no prefix from last code",
                                });
                            }
                            Some((offset, length)) => {
                                if length == 0 {
                                    return Err(LZWDecodeError::PrefixMismatch {
                                        reason: "zero length prefix from last code",
                                    });
                                }
                                let first_char = output[offset];
                                codebook.suffix[current_incomplete_code] = first_char;

                                let slice = &output[offset..offset + length].to_vec();
                                output.extend(slice);
                                output.push(first_char);
                                current_decoded_length += length;
                            }
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        return Err(LZWDecodeError::TooLargeCode {
                            found: code,
                            table_size: codebook.size(),
                        });
                    }
                }
            }

            // We have decoded the current code. Now move on the next and start define it as the
            // current incomplete one.
            if codebook.increment_next_index() {
                // the new code needs
                current_code_bits += 1;
            }

            // Begin new code
            if !codebook.is_full() {
                codebook.prefix[codebook.next_index.unwrap()] = Some((
                    output.len() - current_decoded_length,
                    current_decoded_length,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode() {
        // We want to code the input: 00 00 01 10 11 00 00 -end using the minimum_code_size of 2
        // thus we have: 000 000 001 0010 0011 0110 - end(0101) the bit increase is due the code
        // size needing 4 bits
        #[allow(clippy::unusual_byte_groupings)]
        let mut data = &[0b01_000_000u8, 0b011_0010_0, 0b101_0110_0, 0b0000000_0] as &[u8];
        // we must use little endian so lowest bit comes first. wrap your head around
        //let expected = [0b00u8, 0b01u8, 0b00u8, 0b01u8, 0b11u8];
        let expected = [0b00, 0b00, 0b01, 0b10, 0b11, 0b00, 0b00];

        let res = LZWDecoder::decode(&mut data, 2);
        let unwrapped = res.unwrap();
        for (i, _) in unwrapped.iter().enumerate() {
            assert_eq!(
                unwrapped[i], expected[i],
                "elements at index {} should be equal",
                i
            );
        }
    }
    //@credit follow test have been stolen from github.com/redwarp/lzw as this I know these are
    //correct.
    #[test]
    fn decode_4color_data() {
        // bytes: 0b10_001_100, 0b0010_110_1, 0b1001_1001
        // as you can see it first clears then outputs 001, then uses the incomplete code 110 which
        // will then be completed for a reuse. Then transitions to 4 bits due to code size.
        let data = [
            0x8C, 0x2D, 0x99, 0x87, 0x2A, 0x1C, 0xDC, 0x33, 0xA0, 0x2, 0x55, 0x0,
        ];

        let res = LZWDecoder::decode(&data[..], 2);
        if res.is_err() {
            panic!("failed to decode {}", res.unwrap_err())
        }

        assert_eq!(
            res.unwrap(),
            [
                1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2,
                2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2,
            ]
        );
    }
    #[test]
    fn decode_lorem_ipsum() {
        let data = include_bytes!("../../../test-assets/lorem_ipsum_encoded.bin");
        let expected = include_bytes!("../../../test-assets/lorem_ipsum.txt");

        let res = LZWDecoder::decode(&data[..], 7);
        if res.is_err() {
            panic!("failed to decode {}", res.unwrap_err())
        }

        assert_eq!(res.unwrap(), expected);
    }
}
