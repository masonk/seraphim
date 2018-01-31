/* Core defines fundamental data structures.

Game state, and anything that would be part of a permanent record of a game belongs here. */
pub mod pos;
use std::fmt;
use left_pad;
use self::pos::Pos;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            &Player::Black => Player::White,
            &Player::White => Player::Black,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            &Player::Black => Color::Black,
            &Player::White => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
    Empty,
}

type Board19 = [Color; 361];

fn hash(board: &Board19) -> Vec<u8> {
    board.iter().map(|v| *v as u8).collect::<Vec<u8>>()
}

pub struct State19 {
    board: Board19,
    next: Player,
    recent: [Board19; 8], // The most recent 8 prior states, For feeding to the neural net
    komap: HashMap<Vec<u8>, bool>, // For detecting positional superkos
    record: Vec<Turn>,    // All moves from the start of the game
}

pub enum IllegalMoveError {
    PositionalSuperko,
    Occupied(Color),
}

enum Turn {
    Pass,
    Of(Move),
}
struct Move {
    who: Player,
    pos: Pos,
}

impl State19 {
    pub fn new() -> Self {
        State19 {
            next: Player::Black,
            recent: [[Color::Empty; 361]; 8], // todo: are these laid out contiguously?
            board: [Color::Empty; 361],
            komap: HashMap::new(),
            record: vec![],
        }
    }
    fn idx(&Pos(r, c): &Pos) -> usize {
        r as usize + c as usize * 19
    }
    fn set(&mut self, pos: &Pos, state: Color) {
        self.board[Self::idx(pos)] = state;
    }
    pub fn get(&self, pos: &Pos) -> &Color {
        &self.board[Self::idx(pos)]
    }

    pub fn play(&mut self, pos: &Pos) -> Result<(), IllegalMoveError> {
        {
            let cur = self.get(pos);
            match cur {
                &Color::Empty => {}
                _ => return Err(IllegalMoveError::Occupied(*cur)),
            }
        }
        let point = self.next.color();
        self.set(pos, point);
        let kokey = hash(&self.board);
        let prev;
        {
            match self.komap.get(&kokey) {
                Some(_) => prev = true,
                _ => prev = false,
            }
        }
        if prev {
            self.set(pos, Color::Empty);
            return Err(IllegalMoveError::PositionalSuperko);
        }

        self.komap.insert(kokey, true);
        self.next = self.next.other();
        // update the "recent states" buffers
        for i in 0..8 {
            if i >= self.record.len() {
                break;
            }
            let mut board = self.recent[i];
            if let &Turn::Of(Move {
                ref who,
                pos: ref p,
            }) = &self.record[i]
            {
                board[Self::idx(p)] = who.color();
            }
        }

        let mv = Turn::Of(Move {
            who: self.next,
            pos: pos.clone(),
        });
        self.record.push(mv);
        Ok(())
    }

    pub fn play_str(&mut self, pos: &str) -> Result<(), IllegalMoveError> {
        self.play(&Pos::parse(pos))
    }
}

/*
Looks like this:

     a b c d e f g h i j k l m n o p q r s
    --------------------------------------
19 | . . . . . . . . . . . . . . . . . . .
18 | . . . . . . . . . . . . . . . . . . .
17 | . . . . . . . . . . . . . . . . . . .
16 | . . x . . . . . . . . . . . . . . . .
15 | . . . . . . . . . . . . . . . . . . .
14 | . . . . . . . . . . . . . . . . . . .
13 | . . . . . . . . . . . . . . . . . . .
12 | . . . . . . . . . . . . . . . . . . .
11 | . . . . . . . . . . . . . . . . . . .
10 | . . . . . . . . . . . . . . . . . . .
 9 | . . . . . . . . . . . . . . . . o . .
 8 | . . . . . . . . . . . . . . . . . . .
 7 | . . . . . . . . . . . . . . . . . . .
 6 | . . . . . . . . . . . . . . . . . . .
 5 | . . . . . . . . . . . . . . . . . . .
 4 | . . . . x . . . . . . . . . . o . . .
 3 | . . . . . . . . . . . . . . . . . . .
 2 | . . . . . . . . . . . . . . . . . . .
 1 | . . . . . . . . . . . . . . . . . . .
 
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
                let pos = Pos(i, j);
                let point = self.get(&pos);
                let val = match point {
                    &Color::Black => " o",
                    &Color::White => " x",
                    &Color::Empty => " .",
                };
                f.write_str(val)?;
            }
            f.write_str("\n")?;
        }

        f.write_str("")
    }
}
