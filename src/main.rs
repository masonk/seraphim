extern crate flexi_logger;
#[macro_use]
extern crate log;
extern crate seraph;

// use seraph::core;
use std::path::PathBuf;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::BufReader;

fn init_env_logger() {
    flexi_logger::Logger::with_env()
        .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap()
}
fn main() {
    init_env_logger();
    seraph::core::sgf_replays::do_one(PathBuf::from("data/jgdb/./sgf/test/0004/00004648.sgf"))
        .unwrap();
}
