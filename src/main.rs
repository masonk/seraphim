extern crate golden;
extern crate gosgf;
extern crate serde;
extern crate serde_json;

use golden::core;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let jgdb = PathBuf::from("data/jgdb");
    let filefilename = "data/jgdb/all.txt";

    let filefile = File::open(filefilename).expect("Couldn't open filefile");
    for fname in BufReader::new(filefile).lines().take(100) {
        let path = jgdb.join(PathBuf::from(fname.unwrap()));
        do_one(path);
    }
    do_one(PathBuf::from("data/jgdb/./sgf/test/0000/00000093.sgf"));
}

fn do_one(path: PathBuf) {
    let file = File::open(path.clone()).expect(&format!("Couldn't open path {:?}", path));

    let mut buf = String::new();
    BufReader::new(file).read_to_string(&mut buf).unwrap();

    match gosgf::parse_sgf::parse_Collection(&buf) {
        Ok(parse) => {
            let mut board = core::State19::init_from_sgf(&parse[0]);
            for sgfmove in parse[0].main_line() {
                let turn = core::Turn::from_sgf(sgfmove);
                match board.play(turn.clone()) {
                    Err(err) => {
                        println!("----------------------------------------------------");
                        println!("{}", path.to_string_lossy());
                        println!("Move error {:?} for move {:?}", err, turn);
                        // println!("{:?}", parse[0]);
                        println!("{}", board);
                        println!("{}", board.serialize());
                        println!("----------------------------------------------------");

                        break;
                    }
                    _ => {}
                }
            }
        }
        Err(err) => {
            println!("----------------------------------------------------");
            println!("{}", path.to_string_lossy());
            println!("parse error {:?}", err);
            println!("----------------------------------------------------");
        }
    }
}
