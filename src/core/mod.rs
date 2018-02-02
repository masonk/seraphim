/* Core defines fundamental data structures.

Game state, and anything that would be part of a permanent record of a game belongs here. */
pub mod pos;
use std::fmt;
use left_pad;
use self::pos::Pos;
use vec_map::VecMap;
use std::collections::HashMap;
use itertools::Itertools;

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
    Empty,
}

type Board19 = [Color; 19 * 19];

fn hash(board: &Board19) -> Vec<u8> {
    board.iter().map(|v| *v as u8).collect::<Vec<u8>>()
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

// A group of stones of a single color
struct Group {
    color: Color,
    stones: Vec<Pos>,
}

/*
The rules encoded here are the Tromp-Taylor rules, which is a formulation of the Chinese rules which makes it easy for a computer to deterministically score the game.

https://en.wikibooks.org/wiki/Computer_Go/Tromp-Taylor_Rules

1. Go is played on a 19x19 square grid of points, by two players called Black and White.
2. Each point on the grid may be colored black, white or empty.
3. A point P, not colored C, is said to reach C, if there is a path of (vertically or horizontally) adjacent points of P's color from P to a point of color C.
4. Clearing a color is the process of emptying all points of that color that don't reach empty.
5. Starting with an empty grid, the players alternate turns, starting with Black.
6. A turn is either a pass; or a move that doesn't repeat an earlier grid coloring.
7. A move consists of coloring an empty point one's own color; then clearing the opponent color, and then clearing one's own color.
8. The game ends after two consecutive passes.
9. A player's score is the number of points of her color, plus the number of empty points that reach only her color.
10. The player with the higher score at the end of the game is the winner. Equal scores result in a tie.

TODO: Perf hacks:

index liberties?
    - Is there an efficient way to merge two groups' liberties (any two groups that are merging due to a new placement share at least that one liberties at the placed stone, and might share more.)
    - Is there an efficient way to update liberties on capture?

change repr of Pos into a usize instead of a tuple
*/

pub struct State19 {
    next_player: Player,
    boards: [Board19; 9], // The most recent 9 board states states. Zeroth board is the current state. This unorthodox layout is how the net likes to feed.
    komap: HashMap<Vec<u8>, bool>, // For detecting positional superkos. TODO: For speed we could just not check for this and only enforce the basic ko rule
    record: Vec<Turn>,             // All moves from the start of the game. Used for serialization.
    group_index: VecMap<usize>, // Which group each stone on the board belongs to. Indexed by board position. Meaningless if the position is Empty.
    groups: VecMap<Vec<usize>>, // Which stones each group owns. Indexed by group id.
    // liberties: VecMap<usize>, // TODO: Maintain an index of how many liberties each group has. Indexed by group id.
    next_id: usize,
}

impl State19 {
    pub fn new() -> Self {
        State19 {
            next_player: Player::Black,
            boards: [[Color::Empty; 19 * 19]; 9], // the most recent 9 boards. the 0th board is the current state
            record: Vec::with_capacity(600),
            komap: HashMap::with_capacity(19 * 19),
            group_index: VecMap::with_capacity(19 * 19),
            groups: VecMap::with_capacity(19 * 19),
            // liberties: VecMap::with_capacity(19 * 19),
            next_id: 0,
        }
    }
    fn idx(&Pos(r, c): &Pos) -> usize {
        r as usize + c as usize * 19
    }
    fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn set_idx(&mut self, idx: &usize, state: Color) {
        self.boards[0][*idx] = state;
    }
    fn set(&mut self, pos: &Pos, state: Color) {
        self.set_idx(&Self::idx(pos), state);
    }
    pub fn get_idx(&self, idx: &usize) -> &Color {
        &self.boards[0][*idx]
    }
    pub fn get(&self, pos: &Pos) -> &Color {
        self.get_idx(&Self::idx(pos))
    }

    fn clear(&mut self, &Move { ref who, ref pos }: &Move) -> u32 {
        0
    }

    // Merge neighboring allied groups into one group, because the stone we're placing connects them all.
    // If 0 allied groups, start a new group that contains only this stone.
    // Returns the group id of the resultant merged group.
    fn merge_groups(&mut self, color: &Color, pos: &Pos) -> usize {
        let neighbors = pos.neighbors().collect::<Vec<Pos>>();
        let allies = neighbors
            .iter()
            .map(|p| (p, self.get(pos)))
            .filter(|&(_, v)| v == color)
            .map(|(p, _)| p)
            .collect::<Vec<&Pos>>();

        let stoneidx = Self::idx(pos);
        let id: usize;
        if allies.len() > 0 {
            // merge all allied groups and the placed stone into the group with this id
            id = *self.group_index.get(Self::idx(allies[0])).unwrap();

            for ally in allies {
                let gid = *self.group_index.get(Self::idx(ally)).unwrap();
                if gid != id {
                    let mut source = self.groups.remove(gid).unwrap();
                    for idx in source.iter() {
                        self.group_index.insert(*idx, id);
                    }
                    let mut destination = self.groups.get_mut(id).unwrap();
                    destination.append(&mut source);
                }
            }
            let destination = self.groups.get_mut(id).unwrap();
            destination.push(stoneidx);
            self.group_index.insert(stoneidx, id);
        } else {
            id = self.get_next_id();
            self.groups.insert(id, vec![stoneidx]);
            self.group_index.insert(stoneidx, id);
        }
        id
    }

    fn clear_group(&mut self, id: usize) {
        for idx in self.groups.get(id).unwrap() {
            // self.set_idx(idx, Color::Empty); // Borrow checker complains
            self.boards[0][*idx] = Color::Empty;
        }
        self.groups.get_mut(id).unwrap().clear();
    }

    pub fn play(&mut self, pos: &Pos) -> Result<(), IllegalMoveError> {
        {
            let cur = self.get(pos);
            match cur {
                &Color::Empty => {}
                _ => return Err(IllegalMoveError::Occupied(cur.clone())),
            }
        }
        let point = self.next_player.color();
        self.set(pos, point);
        let kokey = hash(&self.boards[0]);
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

        // The stone is placed. Now update indexes and perform clearing.

        self.merge_groups(&self.next_player.color(), pos);

        {
            let raw = self as *mut Self;
            // All enemy neighbors need to be checked for capture
            unsafe {
                let enemies = pos.neighbors()
                    .map(|p| (p, (*raw).get(pos)))
                    .filter(|&(_, v)| *v == (*raw).next_player.other().color());

                let groupids = enemies
                    .map(|(ref p, _)| (*raw).group_index.get(Self::idx(p)).unwrap())
                    .unique();

                for id in groupids {
                    let mut clear = true;
                    for idx in (*raw).groups.get(*id).unwrap() {
                        let empty = pos.neighbors()
                            .map(|p| (*raw).get(pos))
                            .any(|&c| c == Color::Empty);
                        if empty {
                            clear = false;
                            break;
                        }
                    }
                    if clear {
                        (*raw).clear_group(*id);
                    }
                }
                // find at least one neighbor that's Color::Empty
            }
        }

        // This stone's group needs to be checked for suicide

        self.next_player = self.next_player.other();
        // TODO: update the "recent states" buffers
        // for i in 0..8 {
        //     if i >= self.record.len() {
        //         break;
        //     }
        //     let mut board = self.recent[i];
        //     if let &Turn::Of(Move {
        //         ref who,
        //         pos: ref p,
        //     }) = &self.record[i]
        //     {
        //         board[Self::idx(p)] = who.color();
        //     }
        // }

        let mv = Turn::Of(Move {
            who: self.next_player,
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
