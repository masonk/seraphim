extern crate golden;

use golden::core;

fn main() {
    let mut board = core::State19::new();
    println!("{}", board);
    for mv in ["e 4", "p 4", "c 16", "q 9"].iter() {
        board.play_str(mv);

        println!("{}\n{}", mv, board);
    }
}
