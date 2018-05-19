
use std::io;
use std::io::prelude::*;
extern crate byteorder;
extern crate crc32c;
use self::byteorder::WriteBytesExt;

pub struct RecordWriter<W: Write> {
//  A TFRecords file contains a sequence of strings 
// with CRC32C (32-bit CRC using the Castagnoli polynomial) hashes. Each record has the format

// uint64 length
// uint32 masked_crc32_of_length
// byte   data[length]
// uint32 masked_crc32_of_data
// and the records are concatenated together to produce the file. CRCs are described here [1], and the mask of a CRC is
// [1] https://en.wikipedia.org/wiki/Cyclic_redundancy_check
// masked_crc = ((crc >> 15) | (crc << 17)) + 0xa282ead8ul
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