/* Core defines fundamental data structures.

Game state, and anything that would be part of a permanent record of a game belongs here. */
pub mod pos;
use std::fmt;
use left_pad;
use self::pos::Pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Point {
    Black,
    White,
    Empty,
}

type Board19 = [Point; 361];

pub struct State19 {
    board: Board19,
    next: Player,
    previous: [Option<Board19>; 8],
}

impl State19 {
    pub fn new() -> Self {
        State19 {
            next: Player::White,
            previous: [None; 8],
            board: [Point::Empty; 361],
        }
    }
    fn idx(&Pos(&r, &c): &Pos) -> usize {
        (r as usize + (c as usize * 19))
    }
    pub fn get(&self, pos: &Pos) -> &Point {
        &self.board[State19::idx(pos)]
    }
}

/*
Looks like this:

     a b c d e f g h i j k l m n o p q r s t u v
    --------------------------------------------
19 | . . . . . . . . . . . . . . . . . . . . . .
18 | . . . . . . . . . . . . . . . . . . . . . .
17 | . . . . . . . . . . . . . . . . . . . . . .
16 | . . . . . . . . . . . . . . . . . . . . . .
15 | . . . . . . . . . . . . . . . . . . . . . .
14 | . . . . . . . . . . . . . . . . . . . . . .
13 | . . . . . . . . . . . . . . . . . . . . . .
12 | . . . . . . . . . . . . . . . . . . . . . .
11 | . . . . . . . . . . . . . . . . . . . . . .
10 | . . . . . . . . . . . . . . . . . . . . . .
 9 | . . . . . . . . . . . . . . . . . . . . . .
 8 | . . . . . . . . . . . . . . . . . . . . . .
 7 | . . . . . . . . . . . . . . . . . . . . . .
 6 | . . . . . . . . . . . . . . . . . . . . . .
 5 | . . . . . . . . . . . . . . . . . . . . . .
 4 | . . . . . . . . . . . . . . . . . . . . . .
 3 | . . . . . . . . . . . . . . . . . . . . . .
 2 | . . . . . . . . . . . . . . . . . . . . . .
 1 | . . . . . . . . . . . . . . . . . . . . . .
 
 */
impl fmt::Display for State19 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lazy_static! {
            static ref HEADER: String = {
                let letters : String = "abcdefghijklmnopqrs"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                format!("     {}", letters)
            };
            static ref HR: String = {
                let hr = ::std::iter::repeat('-').take(19*2).collect::<String>();
                format!("    {}", hr)
            };
        }

        f.write_str(&HEADER)?;
        f.write_str("\n")?;
        f.write_str(&HR)?;
        f.write_str("\n")?;
        for i in (0..19).rev() {
            let row = format!(
                "{} |",
                left_pad::leftpad(format!("{}", i + 1), 2).to_owned()
            );
            f.write_str(&row)?;
            for j in 0..19 {
                let pos = Pos(&i, &j);
                let point = self.get(&pos);
                let val = match point {
                    &Point::Black => " o",
                    &Point::White => " x",
                    &Point::Empty => " .",
                };
                f.write_str(val);
            }
            f.write_str("\n");
        }

        f.write_str("")
    }
}
