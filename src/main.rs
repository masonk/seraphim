extern crate gosgf;
extern crate seraph;
extern crate serde;
extern crate serde_json;

use seraph::core;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    seraph::core::sgf_replays::do_one(PathBuf::from("data/jgdb/./sgf/test/0000/00000836.sgf"))
        .unwrap();
}
