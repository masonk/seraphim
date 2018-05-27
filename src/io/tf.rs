
use std::io;
use std::io::prelude::*;
extern crate byteorder;
extern crate crc32c;
use self::byteorder::WriteBytesExt;

// TODO: this is merged into rust-tensorflow now, so delete this and use it from rust-tensorflow
// (Blocked until they release a new crate version)
pub struct RecordWriter<W: Write> {
    writer: W
}

impl<W> RecordWriter<W> where W: Write {
    pub fn new(writer: W) -> Self {
        RecordWriter { writer }
    }
    pub fn write_one_record(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut len_bytes = vec![];
        len_bytes.write_u64::<byteorder::LittleEndian>(bytes.len() as u64).unwrap();

        let masked_len_crc32c = Self::mask(crc32c::crc32c(&len_bytes));
        let mut len_crc32c_bytes: Vec<u8> = vec![];
        len_crc32c_bytes.write_u32::<byteorder::LittleEndian>(masked_len_crc32c).unwrap();

        let masked_bytes_crc32c = Self::mask(crc32c::crc32c(&bytes));
        let mut bytes_crc32_bytes: Vec<u8> = vec![];
        bytes_crc32_bytes.write_u32::<byteorder::LittleEndian>(masked_bytes_crc32c).unwrap();

        self.writer.write(&len_bytes)?;
        self.writer.write(&len_crc32c_bytes)?;
        self.writer.write(bytes)?;
        self.writer.write(&bytes_crc32_bytes)?;
        Ok(16 + bytes.len())
    }

    fn mask(crc: u32) -> u32 {
        ((crc >> 15) | (crc << 17)).wrapping_add(0xa282ead8u32)
    }
}