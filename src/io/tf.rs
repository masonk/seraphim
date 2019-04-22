extern crate byteorder;
extern crate crc32c;
extern crate fallible_iterator;

use std::io;
use std::io::prelude::*;

use self::byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};

// TODO: this is merged into rust-tensorflow now, so delete this and use it from rust-tensorflow
// (Blocked until they release a new crate version)
pub struct RecordWriter<W: Write> {
    writer: W,
}

fn masked_crc32(bytes: &[u8]) -> u32 {
    do_mask(crc32c::crc32c(&bytes))
}
fn do_mask(crc: u32) -> u32 {
    ((crc >> 15) | (crc << 17)).wrapping_add(0xa282ead8u32)
}

impl<W> RecordWriter<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        RecordWriter { writer }
    }
    pub fn write_one_record(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut len_bytes = vec![];
        len_bytes
            .write_u64::<byteorder::LittleEndian>(bytes.len() as u64)
            .unwrap();

        let masked_len_crc32c = masked_crc32(&len_bytes);
        let mut len_crc32c_bytes: Vec<u8> = vec![];
        len_crc32c_bytes
            .write_u32::<byteorder::LittleEndian>(masked_len_crc32c)
            .unwrap();

        let masked_bytes_crc32c = masked_crc32(&bytes);
        let mut bytes_crc32_bytes: Vec<u8> = vec![];
        bytes_crc32_bytes
            .write_u32::<byteorder::LittleEndian>(masked_bytes_crc32c)
            .unwrap();

        self.writer.write(&len_bytes)?;
        self.writer.write(&len_crc32c_bytes)?;
        self.writer.write(bytes)?;
        self.writer.write(&bytes_crc32_bytes)?;
        Ok(16 + bytes.len())
    }
}

pub struct RecordReader<R: Read> {
    reader: R,
}
impl<R> RecordReader<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        RecordReader { reader }
    }
    // Read one record from a .tfrecord file.
    // Performs cyclic redundancy checking and returns Ok(Vec<u8>) on the record's data segment.
    // Returns Ok(Vec<u8>)
    // Returns Ok(None) upon reaching EOF.
    // Returns Err(io::Error) for data corruption or other unexpected problems.
    pub fn read_one(&mut self) -> io::Result<Option<Vec<u8>>> {
        /**
         * TFRecord format:
         * u64 length
         * u32 masked_crc32_of_length
         * [u8]   data[length]
         * u32 masked_crc32_of_data
         */
        let mut len_buf: [u8; 8] = [0; 8];
        let len_bytes_read = self.reader.read(&mut len_buf)?;
        if len_bytes_read == 0 {
            return Ok(None);
        } else if len_bytes_read < 8 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Corrupted file. There are {} extra bytes at the end of the file:\n {:?}",
                    len_bytes_read, len_buf
                ),
            ));
        }
        let len: u64 = LittleEndian::read_u64(&len_buf);

        let mut crclen_buf: [u8; 4] = [0; 4];
        let crclen_bytes_read = self.reader.read(&mut crclen_buf)?;
        if crclen_bytes_read < 4 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Corrupted file. There are {} extra bytes at the end of the file.",
                    crclen_bytes_read + len_bytes_read
                ),
            ));
        }
        let crclen: u32 = LittleEndian::read_u32(&crclen_buf);

        if crclen != masked_crc32(&len_buf) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Cyclic redundancy check failed for length segment. The record is corrupted.",
            ));
        }

        let mut data_buf: Vec<u8> = Vec::with_capacity(len as usize);
        unsafe {
            data_buf.set_len(len as usize);
        }

        let data_bytes_read = self.reader.read(&mut data_buf[..])?;
        if (data_bytes_read as u64) < len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Corrupted record: There are {} bytes in the data segment, but expected {}.",
                    data_bytes_read, len
                ),
            ));
        }

        let mut datacrc_buf: [u8; 4] = [0; 4];
        let datacrc_bytes_read = self.reader.read(&mut datacrc_buf)?;
        if datacrc_bytes_read < 4 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Corrupted record: Invalid CRC signature for data segment.",
            ));
        }
        let datacrc = byteorder::LittleEndian::read_u32(&datacrc_buf);

        if datacrc != masked_crc32(&data_buf) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Corrupted record: Cyclic redundancy check failed for data segment.",
            ));
        }

        Ok(Some(data_buf))
    }
}

impl<R> IntoIterator for RecordReader<R>
where
    R: Read,
{
    type Item = <self::RecordIter<R> as Iterator>::Item;
    type IntoIter = RecordIter<R>;
    fn into_iter(self) -> Self::IntoIter {
        RecordIter { reader: self }
    }
}

impl<R> fallible_iterator::IntoFallibleIterator for RecordReader<R> 
where R: Read,
{    
    type IntoFallibleIter = FallibleRecordIter<R>;
    type Item = <self::FallibleRecordIter<R> as fallible_iterator::FallibleIterator>::Item;
    type Error = <self::FallibleRecordIter<R> as fallible_iterator::FallibleIterator>::Error;
    fn into_fallible_iter(self) -> FallibleRecordIter<R> {
        FallibleRecordIter {
            reader: self,
        }
    }
}

pub struct FallibleRecordIter<R> where R: Read {
    reader: RecordReader<R>
}
impl<R> fallible_iterator::FallibleIterator for FallibleRecordIter<R> where R: Read{
    type Item = Vec<u8>;
    type Error = io::Error;
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.reader.read_one()
    }
}

pub struct RecordIter<R>
where
    R: Read,
{
    reader: RecordReader<R>,
}
impl<R> Iterator for RecordIter<R>
where
    R: Read,
{
    type Item = io::Result<Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.reader.read_one();
        match res {
            Err(e) => Some(Err(e)),
            Ok(res) => match res {
                Some(r) => Some(Ok(r)),
                None => None,
            },
        }
    }
}