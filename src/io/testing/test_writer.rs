#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate flexi_logger;
extern crate seraphim;
#[macro_use]
extern crate log;

use seraphim::io;
extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use std::result::Result;

fn init_logger() {
    flexi_logger::Logger::with_env()
        .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap();
}
fn main() {
    init_logger();

    let test_file = ::std::fs::OpenOptions::new()
        .write(true)
        .create(true)   
        .open("src/io/testing/test.tfrecord")
        .unwrap();

    let mut w = ::std::io::BufWriter::new(test_file);
    let mut record_writer = io::tf::RecordWriter::new(w);
    record_writer.write_one_record("The Quick Brown Fox".as_bytes());
   
}

// fn main() {}
