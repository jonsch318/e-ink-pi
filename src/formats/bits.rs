// Many Formats use bits and bitreader stuff

use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::io::{self, Write};

use num_traits::{FromBytes, FromPrimitive, Num, ToBytes, ToPrimitive};

pub trait BitSized {
    const BITS: u8;
    const BYTES: u8;
}

pub trait Primitive: BitSized + Num + FromPrimitive + ToPrimitive + FromBytes + ToBytes {}

macro_rules! declare_primitive {
    ($prim:ty, $bytes:expr) => {
        impl BitSized for $prim {
            const BITS: u8 = $bytes * 8;
            const BYTES: u8 = $bytes;
        }
        impl Primitive for $prim {}
    };
}
declare_primitive!(u8, 1);
declare_primitive!(i8, 1);
declare_primitive!(u16, 2);
declare_primitive!(i16, 2);
declare_primitive!(u32, 4);
declare_primitive!(i32, 4);
declare_primitive!(u64, 8);
declare_primitive!(i64, 8);
declare_primitive!(u128, 16);
declare_primitive!(i128, 16);
#[cfg(target_pointer_width = "64")]
declare_primitive!(usize, 8);
#[cfg(target_pointer_width = "32")]
declare_primitive!(usize, 4);
#[cfg(target_pointer_width = "64")]
declare_primitive!(isize, 8);
#[cfg(target_pointer_width = "32")]
declare_primitive!(isize, 4);
declare_primitive!(f32, 4);
declare_primitive!(f64, 8);

#[derive(Debug)]
pub enum BitsReadError {
    UnexpectedEOF,
    //tried to read into a type that cannot fit the data
    UnsufficiantTypeSize,
    //could not convert type from byte
    ConvertFromU8,
    OtherIo(io::Error),
}

impl Display for BitsReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitsReadError::UnexpectedEOF => write!(f, "could not read enough bits"),
            BitsReadError::UnsufficiantTypeSize => {
                write!(f, "the type to read into cannot fit that amount of bits")
            }
            BitsReadError::ConvertFromU8 => write!(f, "could not convert into type from a byte"),
            BitsReadError::OtherIo(err) => write!(f, "io error {}", err),
        }
    }
}
impl Error for BitsReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BitsReadError::OtherIo(err) => Some(err),
            _ => None,
        }
    }
}
impl From<BitsReadError> for io::Error {
    fn from(value: BitsReadError) -> Self {
        match value {
            BitsReadError::UnexpectedEOF => io::Error::from(io::ErrorKind::UnexpectedEof),
            BitsReadError::UnsufficiantTypeSize => io::Error::other(value),
            BitsReadError::ConvertFromU8 => io::Error::other(value),
            BitsReadError::OtherIo(err) => return err,
        }
    }
}

impl From<io::Error> for BitsReadError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::UnexpectedEof => BitsReadError::UnexpectedEOF,
            _ => BitsReadError::OtherIo(value),
        }
    }
}

pub trait BitReader {
    fn read_bit(&mut self) -> io::Result<bool>;
    fn read_n<U: Primitive>(&mut self, bits: u8) -> Result<U, BitsReadError>;

    fn skip(&mut self, bits: u8) -> io::Result<()>;
}

pub struct LittleEndianReader<R: Read> {
    reader: R,
    value: u8,
    bits: u8,
    read_buffer: [u8; 1],
}

impl<R: Read> LittleEndianReader<R> {
    pub fn new(reader: R) -> LittleEndianReader<R> {
        LittleEndianReader {
            reader,                // the underlying reader
            value: 0,              // sliding value. 1 is always the first value
            bits: 0, // bits stored in value. if 3 bits are stored the max of value would be 0b00000111;
            read_buffer: [0u8; 1], // buffer for read_exact of the underlying reader
        }
    }

    pub fn into_reader(self) -> R {
        self.reader
    }

    #[inline(always)]
    fn fill_buffer(&mut self) -> io::Result<()> {
        match self.reader.read_exact(&mut self.read_buffer) {
            Ok(_) => {
                self.value = self.read_buffer[0];
                self.bits = 8;
                Ok(())
            }
            Err(e) => {
                self.bits = 0;
                Err(e)
            }
        }
    }

    #[inline(always)]
    fn take_all_buffered(&mut self) -> u8 {
        self.take_n(self.bits)
    }

    /// Internal takes n bits from the reader regardless of meaning.
    /// If reader state is data: 0b00011101 | bits: 6.
    /// reader.take_n(4) would return 0b00001101
    /// the reader would result in data: 0b00000001 | bits: 2.
    ///
    /// take_n is safe for usage <= 8 bits aswell as 0 bits. `bits` > 8 bits results in panic
    ///
    ///  `bits`:
    fn take_n(&mut self, bits: u8) -> u8 {
        let mask = 1u8.wrapping_shl(bits as u32) - 1;
        let buf = self.value & mask;
        self.value >>= bits;
        self.bits -= bits;
        buf
    }
}

impl<R: Read> BitReader for LittleEndianReader<R> {
    fn read_bit(&mut self) -> io::Result<bool> {
        if self.bits == 0 {
            self.fill_buffer()?;
        }

        let res = self.value & 1;
        self.value >>= 1;
        self.bits -= 1;
        Ok(res == 1)
    }

    fn read_n<U: Primitive>(&mut self, bits: u8) -> Result<U, BitsReadError> {
        if bits > U::BITS {
            return Err(BitsReadError::UnsufficiantTypeSize);
        }

        // we must read exactly `bits` many bits into intermediate

        if self.bits >= bits {
            //we have enough bits read already in the bit buffer so we just create the result type
            let intermediate = self.take_n(bits);
            return U::from_u8(intermediate).ok_or(BitsReadError::ConvertFromU8);
        }

        // we dont have enough bits in the reader buffer. Or we read more than 8bits

        let mut pos = self.bits;
        let mut intermediate = self.take_all_buffered() as u128; //we clear the reader (take_n
                                                                 //safe even for 0 bits and self.bits is
                                                                 //garantueed <= 8)

        while bits - pos >= 8 {
            // we are missing at least 8 bits so we can read full bytes and add them to
            // intermediate
            self.reader.read_exact(&mut self.read_buffer)?;
            intermediate |= (self.read_buffer[0] as u128).wrapping_shl(pos.into());
            pos += 8;
        }

        if bits - pos != 0 {
            //one last read is neccessary
            self.fill_buffer()?;
            intermediate |= (self.take_n(bits - pos) as u128).wrapping_shl(pos.into());
        } else {
            self.bits = 0;
        }

        //TODO: for floats use unsafe assignment
        U::from_u128(intermediate).ok_or(BitsReadError::ConvertFromU8)
    }

    fn skip(&mut self, bits: u8) -> io::Result<()> {
        let mut skipped_bits = bits;
        while skipped_bits > 64 {
            self.read_n::<u64>(bits)?;
            skipped_bits -= u64::BITS as u8;
        }
        self.read_n::<u64>(bits)?;
        Ok(())
    }
}

//Writer

pub trait BitWriter {
    fn write_bit(&mut self, bit: bool) -> io::Result<()>;
    fn write(&mut self, data: &[u8], bits: u8) -> io::Result<()>;
    fn write_n<U: Primitive>(&mut self, data: U, n: u8) -> io::Result<()>;
}

pub struct LittleEndianWriter<W: Write> {
    writer: W,
    buf: [u8; 1],
    pos: u8,
}
impl<W: Write> LittleEndianWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buf: [0u8],
            pos: 0,
        }
    }
    #[inline]
    fn write_out_internal(&mut self) -> io::Result<()> {
        if self.pos > 0 {
            self.writer.write_all(&self.buf)?;
            self.buf[0] = 0;
            self.pos = 0;
        }
        Ok(())
    }
    fn write_remaining_internal(&mut self, data: u8) -> io::Result<u8> {
        let remaining = u8::BITS - self.pos as u32;
        let mask = (1u8.wrapping_shl(remaining)) - 1;
        self.buf[0] |= (data & mask) << self.pos;
        self.write_out_internal()?;
        Ok(remaining as u8)
    }
    #[inline]
    fn write_internal(&mut self, mut data: u8, mut bits: u8) -> io::Result<()> {
        let remaining = u8::BITS as u8 - self.pos; // remaining bits to fill the buffer

        // there are enough bits to fill the buffer at least once
        if bits >= remaining {
            let mask: u8 = (1u8 << remaining) - 1;
            self.buf[0] |= (data & mask) << self.pos;
            self.write_out_internal()?;
            data >>= remaining;
            bits -= remaining;
        }

        //write out any remaining bits
        if bits > 0 {
            self.buf[0] |= data << self.pos;
            self.pos += bits;
            if self.pos == 8 {
                self.write_out_internal()?;
            }
        };
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.write_out_internal()
    }
}

impl<W: Write> BitWriter for LittleEndianWriter<W> {
    fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        // we could use write_internal, but this is simple enough and we spare us a extra branching
        // if
        self.buf[0] |= (bit as u8) << self.pos;
        self.pos += 1;
        if self.pos == 8 {
            self.write_out_internal()?;
        }
        Ok(())
    }

    fn write(&mut self, data: &[u8], bits: u8) -> io::Result<()> {
        assert!(!data.is_empty());
        if self.pos == 0 {
            //write out each full byte until the
            if bits % 8 == 0 {
                self.writer.write_all(data)?;
            } else {
                self.writer.write_all(&data[..data.len() - 1])?;
                self.write_internal(data[data.len() - 1], bits)?;
            }
        } else {
            let mut index = 0;
            let mut total_bits = (u8::BITS as usize * (data.len() - 1)) + bits as usize;

            let remaining = u8::BITS as u8 - self.pos;
            let mask = (1u8.wrapping_shl(remaining as u32)) - 1;
            while index < data.len() - 1 && total_bits > u8::BITS as usize {
                let mut split_byte = data[index];
                self.buf[0] |= (split_byte & mask) << self.pos;
                self.writer.write_all(&self.buf)?;
                split_byte >>= remaining;
                self.buf[0] = split_byte;
                index += 1;
                total_bits -= u8::BITS as usize;
            }
            self.write_internal(data[index], bits)?;
        }
        Ok(())
        // now send all until <= 8 bits remain

        // now add the remaining to the buffer
    }

    fn write_n<U: Primitive>(&mut self, _data: U, bits: u8) -> io::Result<()> {
        assert!(bits <= U::BITS);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn assert_result_eq<T: std::fmt::Debug + PartialEq>(
        result: Result<T, BitsReadError>,
        expected: T,
    ) {
        match result {
            Ok(value) => {
                assert_eq!(
                    value, expected,
                    "Expected value: {:?}, but got: {:?}",
                    expected, value
                );
            }
            Err(e) => {
                panic!("Expected Ok({:?}), but got Err({:?})", expected, e);
            }
        }
    }

    #[test]
    fn test_little_endian_read_n_u8() {
        let mut data = &[0b0000_0110u8, 0b00000111] as &[u8];

        let mut bit_reader = LittleEndianReader::new(&mut data);

        assert_result_eq(bit_reader.read_n::<u8>(1), 0u8);
        assert_result_eq(bit_reader.read_n::<u8>(1), 1u8);
        assert_result_eq(bit_reader.read_n::<u8>(2), 1u8);
        assert_result_eq(bit_reader.read_n::<u8>(4), 0u8);

        assert_result_eq(bit_reader.read_n::<u8>(4), 0b0111u8);
        assert_result_eq(bit_reader.read_n::<u8>(4), 0u8);
    }
    #[test]
    fn test_little_endian_read_n_u32() {
        let data_pre = 2848593921u32;
        let mut data = &data_pre.to_le_bytes() as &[u8];

        let mut bit_reader = LittleEndianReader::new(&mut data);

        assert_result_eq(bit_reader.read_n::<u32>(u32::BITS as u8), data_pre);
    }
    #[test]
    fn test_little_endian_read_n_u64() {
        let data_pre = 28485939212312u64;
        let mut data = &data_pre.to_le_bytes() as &[u8];

        let mut bit_reader = LittleEndianReader::new(&mut data);

        assert_result_eq(bit_reader.read_n::<u64>(u64::BITS as u8), data_pre);
    }

    // #[test]
    // fn test_little_endian_read_n_f64() {
    //     type T = f64;
    //     let data_pre: T = 1238.23f64;
    //     let mut data = &data_pre.to_le_bytes() as &[u8];
    //
    //     let mut bit_reader = LittleEndianReader::new(&mut data);
    //
    //     assert_result_eq(bit_reader.read_n::<T>(T::BITS as u8), data_pre);
    // }

    #[test]
    fn test_little_endian_read_eof() {
        let mut data = &[0b0000_0110u8] as &[u8];

        let mut bit_reader = LittleEndianReader::new(&mut data);

        assert_result_eq(bit_reader.read_n::<u8>(1), 0u8);
        assert_result_eq(bit_reader.read_n::<u8>(1), 1u8);
        assert_result_eq(bit_reader.read_n::<u8>(2), 1u8);
        assert_result_eq(bit_reader.read_n::<u8>(4), 0u8);

        assert!(bit_reader.read_n::<u8>(8).is_err())
    }

    #[test]
    fn test_little_endian_writer_write_internal() {
        let mut data: Vec<u8> = Vec::new();
        let mut bit_writer = LittleEndianWriter::new(&mut data);

        bit_writer.write_internal(0b0000_1011, 4).unwrap();
        bit_writer.write_internal(0b1100_1101, 8).unwrap();
        bit_writer.flush().unwrap();

        assert_eq!(data, [0b1101_1011, 0b0000_1100]);
    }
    #[test]
    fn test_little_endian_writer_write_internal_temp() {
        let mut data: Vec<u8> = Vec::new();
        let mut bit_writer = LittleEndianWriter::new(&mut data);

        bit_writer.write_internal(0b0000_1100, 4).unwrap();
        bit_writer.write_internal(0b0000_1111, 4).unwrap();
        bit_writer.flush().unwrap();

        assert_eq!(data, [0b1111_1100]);
    }
    #[test]
    fn test_little_endian_writer_write() {
        let mut data: Vec<u8> = Vec::new();
        let mut bit_writer = LittleEndianWriter::new(&mut data);

        bit_writer.write(&[0b0000_1011], 4).unwrap();
        bit_writer.write(&[0b1100_1101, 0b0000_1111], 4).unwrap();
        bit_writer.flush().unwrap();

        assert_eq!(data, [0b1101_1011, 0b1111_1100]);
    }
}
