extern crate golden;
extern crate gosgf;

use golden::core;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let mut board = core::State19::new();
    let fname = "data/jgdb/sgf/test/0000/00000000.sgf";
    let game = File::open(fname).unwrap();
    let mut buf = String::new();
    BufReader::new(game).read_to_string(&mut buf).unwrap();
    let parse = gosgf::parse_sgf::parse_Collection(&buf).unwrap();

    for sgfmove in parse[0].main_line() {
        let turn = core::Turn::from_sgf(sgfmove);
        println!("{}", board);
        println!("{:?}", turn);

        board.play(turn).unwrap();
    }
    println!("{}", board);
}
