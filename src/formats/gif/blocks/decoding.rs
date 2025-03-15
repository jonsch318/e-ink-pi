use std::io::{self, Read};

pub enum FixedSizeBlockParseError {
    InvalidSize { expected: usize, found: usize },
}

pub fn read_n_byte<R, const n: usize>(reader: &mut R) -> io::Result<[u8; n]>
where
    R: Read,
{
    if n == 0 {
        return Ok([0u8; n]);
    }
    let mut buf = [0u8; n];
    let read_byte_count = reader.read(&mut buf)?;
    if read_byte_count != n {
        // We have not read enough bytes (maybe even n >= !?)
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }
    Ok(buf)
}

pub fn read_subblock<R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();

    loop {
        //expect 1st byte to be the size of the data subblock
        let mut size_buf = [0u8; 1];
        reader.read_exact(&mut size_buf)?;

        let size = size_buf[0];

        if size == 0 {
            //read null terminator. Subblock is now ended
            break;
        }

        //PERF: Optimize collection
        let mut subblock_buf = vec![0u8; size as usize];
        reader.read_exact(&mut subblock_buf)?;
        buf.extend(subblock_buf.iter())
    }

    Ok(buf)
}

pub fn skip_subblock<R: Read>(mut reader: R) -> io::Result<()> {
    loop {
        //expect 1st byte to be the size of the data subblock
        let mut size_buf = [0u8; 1];
        reader.read_exact(&mut size_buf)?;

        let size = size_buf[0];

        if size == 0 {
            //read null terminator. Subblock is now ended
            break;
        }

        let mut subblock_buf = vec![0u8; size as usize];
        reader.read_exact(&mut subblock_buf)?;
    }

    Ok(())
}

pub fn read_lzw_subblock<R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    todo!()
}
