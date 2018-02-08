extern crate gosgf;
use gosgf::parse_sgf;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let fname = "../data/jgdb/sgf/test/0000/00000000.sgf";
    let game = File::open(fname).unwrap();
    let mut buf = String::new();
    BufReader::new(game).read_to_string(&mut buf).unwrap();
    let parse = parse_sgf::parse_Collection(&buf);
    println!("{:?}", parse);
}
