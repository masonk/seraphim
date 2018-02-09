/* Core defines fundamental data structures.

Game state, and anything that would be part of a permanent record of a game belongs here. */
pub mod pos;
use std::fmt;
use left_pad;
use self::pos::Pos19;
use vec_map::VecMap;
use std::collections::HashMap;
use std::collections::BTreeSet;
use itertools::Itertools;
use gosgf;
use gosgf::Move as SgfMove;
use gosgf::PointColor as SgfColor;
use gosgf::Stone as SgfStone;
use serde_json;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
    Black,
    White,
    Empty,
}

type Board19 = [Color; 19 * 19];

fn hash(board: &Board19) -> Vec<u8> {
    board.iter().map(|v| *v as u8).collect::<Vec<u8>>()
}

#[derive(Debug)]
pub enum IllegalMoveError {
    PositionalSuperko,
    Occupied(Color),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Turn {
    Pass,
    Of(Pos19),
    Add(Color, Pos19),
}
impl Turn {
    pub fn from_sgf(sgf: SgfMove) -> Self {
        match sgf {
            SgfMove::Pass => Turn::Pass,
            SgfMove::Of(SgfStone { point, .. }) => {
                Turn::Of(Pos19::from_sgf_coords(point.0, point.1))
            }
            SgfMove::Add(SgfStone { color, point }) => Turn::Add(
                match color {
                    SgfColor::Black => Color::Black,
                    SgfColor::White => Color::White,
                    SgfColor::Empty => Color::Empty,
                },
                Pos19::from_sgf_coords(point.0, point.1),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnList {
    list: Vec<Turn>,
}

#[derive(Debug, PartialEq)]
pub struct Score {
    pub black: f64,
    pub white: f64,
}
/*
The rules encoded here are the Tromp-Taylor rules, which is a formulation of the Chinese rules that makes it easy for a computer to deterministically score the game. 

They:

* Use area scoring. A player's final score is the number of his stones on the board plus the number of empty spaces that are only reachable from his color of stones.

* Don't remove dead groups: players must play to kill all "dead" groups before passing. TODO: Add a dead group "marking and agreement" phase after the engine gets smart enough to know when a group is dead.

* Enforce positional superko: the game state may never be repeated.

* Allow suicide.

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

TODO:

don't hash positional superko?

*/
pub struct State19 {
    next_player: Player,
    boards: [Board19; 9], // The most recent 9 board states states. Zeroth board is the current state. This unorthodox layout is how the net likes to feed.
    komap: HashMap<Vec<u8>, bool>, // For detecting positional superkos. TODO: For speed we could just not check for this and only enforce the basic ko rule
    record: Vec<Turn>,             // All moves from the start of the game. Used for serialization.
    group_index: VecMap<usize>, // Which group each stone on the board belongs to. Indexed by board position. Meaningless if the position is Empty.
    groups: VecMap<Vec<usize>>, // Which stones each group owns. Indexed by group id.
    liberties: VecMap<BTreeSet<usize>>, // All of the liberties that a group has
    next_id: usize,
    komi: f64,
}

impl State19 {
    pub fn serialize(&self) -> String {
        let moves = serde_json::to_string_pretty(&self.record).unwrap();
        moves
    }
    pub fn new() -> Self {
        State19 {
            next_player: Player::Black,
            boards: [[Color::Empty; 19 * 19]; 9], // the most recent 9 boards. the 0th board is the current state
            record: Vec::with_capacity(600),
            komap: HashMap::with_capacity(19 * 19),
            group_index: VecMap::with_capacity(19 * 19),
            groups: VecMap::with_capacity(19 * 19),
            liberties: VecMap::with_capacity(19 * 19),
            next_id: 0,
            komi: 7.5,
        }
    }
    pub fn init_from_sgf(tree: &gosgf::GameTree) -> Self {
        let mut board = Self::new();
        board.komi = tree.komi;
        board.next_player = if tree.handicap == 0 {
            Player::Black
        } else {
            Player::White
        };
        board
    }
    fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn set_idx(&mut self, &Pos19(idx): &Pos19, state: Color) {
        self.boards[0][idx] = state;
    }
    fn set(&mut self, pos: &Pos19, state: Color) {
        self.set_idx(pos, state);
    }
    pub fn get_idx(&self, &Pos19(idx): &Pos19) -> Color {
        self.boards[0][idx]
    }
    pub fn get(&self, pos: &Pos19) -> Color {
        self.get_idx(pos)
    }
    pub fn score(&mut self) -> Score {
        let mut black = 0.0;
        let mut white = self.komi;
        let self_ptr = self as *mut Self;
        let mut scored = [false; 19 * 19]; // After a stone has been considered mark it as scored so we don't reconsider. Only used for the empty point "minesweeper" algorithm.
        for (idx, color) in self.boards[0].iter().enumerate() {
            if scored[idx] {
                continue;
            }
            match color {
                &Color::Black => black += 1.0,
                &Color::White => white += 1.0,
                &Color::Empty => {
                    let mut group = BTreeSet::new();
                    let mut reaches = BTreeSet::new();
                    unsafe {
                        (*self_ptr).flood_fill_group(
                            &Color::Empty,
                            &Pos19(idx),
                            &mut group,
                            &mut reaches,
                            &mut scored,
                        );
                    }

                    if reaches.len() == 1 {
                        if reaches.contains(&Color::White) {
                            white += group.len() as f64;
                        } else {
                            black += group.len() as f64;
                        }
                    }
                }
            }
        }
        Score { black, white }
    }
    // Merge neighboring allied groups into one group, because the stone we're placing connects them all.
    // If 0 allied groups, start a new group that contains only this stone.
    // Returns the group id of the resultant merged group.
    fn merge_groups(&mut self, color: &Color, pos: &Pos19) -> usize {
        let neighbors = pos.neighbors().collect::<Vec<Pos19>>();
        let allies = neighbors
            .iter()
            .filter(|p| self.get(p) == *color)
            .collect::<Vec<&Pos19>>();
        let &Pos19(stoneidx) = pos;
        let id: usize;
        if allies.len() > 0 {
            // merge all allied groups and the placed stone into the group with this id
            let &Pos19(idx) = allies[0];
            id = *self.group_index.get(idx).unwrap();

            for ally in allies {
                let &Pos19(idx) = ally;
                let gid = *self.group_index.get(idx).unwrap();
                if gid != id {
                    let mut source = self.groups.remove(gid).unwrap();
                    for idx in source.iter() {
                        self.group_index.insert(*idx, id);
                    }
                    let mut destination = self.groups.get_mut(id).unwrap();
                    destination.append(&mut source);

                    let mut liberties = self.liberties.remove(gid).unwrap();
                    let mut dest_liberties = self.liberties.get_mut(id).unwrap();
                    dest_liberties.append(&mut liberties);
                }
            }
            let destination = self.groups.get_mut(id).unwrap();
            destination.push(stoneidx);
            self.group_index.insert(stoneidx, id);

            let empty_neighbors = pos.neighbors()
                .filter(|p| self.get(p) == Color::Empty)
                .map(|Pos19(p)| p)
                .collect::<BTreeSet<usize>>();
            let mut dest_liberties = self.liberties.get_mut(id).unwrap();

            for lib in empty_neighbors {
                dest_liberties.insert(lib);
            }

            dest_liberties.remove(&stoneidx);
        } else {
            id = self.get_next_id();
            self.groups.insert(id, vec![stoneidx]);
            self.group_index.insert(stoneidx, id);
            let empty_neighbors = pos.neighbors()
                .filter(|p| self.get(p) == Color::Empty)
                .map(|Pos19(p)| p)
                .collect::<BTreeSet<usize>>();

            self.liberties.insert(id, empty_neighbors);
        }
        id
    }

    fn clear_group(&mut self, id: usize) {
        for idx in self.groups.get(id).unwrap() {
            // self.set_idx(idx, Color::Empty); // Borrow checker complains
            self.boards[0][*idx] = Color::Empty;
        }
        let self_ptr = self as *mut Self;
        for idx in self.groups.get(id).unwrap() {
            let neighboring_stones = Pos19(*idx)
                .neighbors()
                .filter(|n| self.get(n).clone() != Color::Empty);
            for Pos19(stone) in neighboring_stones {
                let groupid = self.group_index.get(stone).unwrap().clone();
                unsafe {
                    let mut liberties = (*self_ptr).liberties.get_mut(groupid).unwrap();
                    liberties.insert(*idx);
                }
            }
        }
        self.groups.get_mut(id).unwrap().clear();
    }

    fn flood_fill_group(
        &mut self,
        color: &Color,
        next: &Pos19,
        same: &mut BTreeSet<usize>,      // all the points in this group
        reachable: &mut BTreeSet<Color>, // all the other colors we reached while filling
        scored: &mut [bool; 19 * 19],
    ) {
        // Find the extent of a group that contains some point by recursively expanding the members of the group. (Imagine clicking a box in minesweeper and having it fill out the group).
        // In the process also find all the other colors that were reached.
        let same_ptr = same as *const BTreeSet<usize>;
        let self_ptr = self as *mut Self;
        unsafe {
            for n in next.neighbors().filter(|p| (*same_ptr).get(&p.0).is_none()) {
                let neighboring_color = self.get(&n);
                let Pos19(idx) = n;
                if neighboring_color == *color {
                    same.insert(n.0);
                    scored[idx] = true;
                    (*self_ptr).flood_fill_group(color, &n, same, reachable, scored);
                } else {
                    reachable.insert(neighboring_color);
                }
            }
        }
    }

    fn nearby_groups(&self, pos: &Pos19, color: Color) -> Vec<usize> {
        pos.neighbors()
            .filter(move |p| self.get(p) == color)
            .map(|Pos19(eidx)| self.group_index.get(eidx).unwrap().clone())
            .unique()
            .collect::<Vec<usize>>()
    }

    pub fn play(&mut self, turn: Turn) -> Result<(), IllegalMoveError> {
        match turn {
            Turn::Pass => {
                self.record.push(turn);
                self.next_player = self.next_player.other();
                Ok(())
            }
            Turn::Add(color, ref pos) => {
                self.set(pos, color);
                self.merge_groups(&self.next_player.color(), pos);
                self.record.push(turn);
                Ok(())
            }
            Turn::Of(ref pos) => {
                {
                    let cur = self.get(pos);
                    match cur {
                        Color::Empty => {}
                        _ => return Err(IllegalMoveError::Occupied(cur.clone())),
                    }
                }
                let point = self.next_player.color();
                self.set(pos, point);
                let kokey = hash(&self.boards[0]);
                let prev;

                let badpos = Pos19::parse("s 12");

                match self.komap.get(&kokey) {
                    Some(_) => prev = true,
                    _ => prev = false,
                }

                if prev {
                    self.set(&pos, Color::Empty);
                    return Err(IllegalMoveError::PositionalSuperko);
                }

                self.komap.insert(kokey, true);

                // The stone is placed. Now update indexes and perform clearing.

                // merge all allied groups into one
                let this_group_id = self.merge_groups(&self.next_player.color(), &pos);
                if this_group_id == 12 {
                    let mut these_liberties = self.liberties.get_mut(this_group_id).unwrap();
                    println!(
                        "libs after merge_groups: {:?}",
                        these_liberties
                            .iter()
                            .map(|v| Pos19(*v).pretty())
                            .collect::<Vec<String>>(),
                    );
                }
                if *pos == badpos {
                    println!("The s12 groupid: {}", this_group_id);
                }

                // Every group that counted this position as a liberty stops counting it.
                let &Pos19(thisidx) = pos;
                let enemygroups = self.nearby_groups(pos, self.next_player.other().color());

                if *pos == badpos {
                    println!("enemy groupids: {:?}", enemygroups);
                }
                for groupid in enemygroups {
                    let mut libs = self.liberties.get_mut(groupid).unwrap();

                    if *pos == badpos || groupid == 12 {
                        println!(
                            "libs before remove: {:?}",
                            libs.iter()
                                .map(|v| Pos19(*v).pretty())
                                .collect::<Vec<String>>(),
                        );
                        println!(
                            "Enemies in group {}: {:?}",
                            groupid,
                            self.groups
                                .get(groupid)
                                .unwrap()
                                .iter()
                                .map(|v| Pos19(*v).pretty())
                                .collect::<Vec<String>>()
                        );
                    }
                    libs.remove(&thisidx);
                    if *pos == badpos || groupid == 12 {
                        println!(
                            "libs after remove: {:?} (removed: {})",
                            libs.iter()
                                .map(|v| Pos19(*v).pretty())
                                .collect::<Vec<String>>(),
                            Pos19(thisidx).pretty()
                        );
                        println!("{} liberties for {}", libs.len(), groupid);
                    }
                    if libs.len() == 0 {
                        // enemy group is killed
                        self.clear_group(groupid);
                    }
                }

                if self.liberties.get(this_group_id).unwrap().len() == 0 {
                    self.clear_group(this_group_id);
                    // suicide
                }

                self.next_player = self.next_player.other();
                self.record.push(turn);
                Ok(())
            }
        }
    }

    pub fn play_str(&mut self, pos: &str) -> Result<(), IllegalMoveError> {
        self.play(Turn::Of(Pos19::parse(pos)))
    }
}

impl fmt::Debug for State19 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self);
        write!(f, "group_index: {:?}\n", self.group_index);
        write!(f, "groups: {:?}\n", self.groups);
        write!(f, "liberties: {:?}\n", self.liberties);
        write!(f, "next_id: {}\n", self.next_id)
    }
}

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
        for i in 0..19 {
            let row = format!(
                "{} |",
                left_pad::leftpad(format!("{}", i + 1), 2).to_owned()
            );
            f.write_str(&row)?;
            for j in 0..19 {
                let pos = Pos19::from_coords(j, i);
                let point = self.get(&pos);
                let val = match point {
                    Color::Black => " o",
                    Color::White => " x",
                    Color::Empty => " .",
                };
                f.write_str(val)?;
            }
            f.write_str("\n")?;
        }

        f.write_str("")
    }
}

#[cfg(test)]
mod basic {
    use super::*;

    #[test]
    fn captures_are_cleared() {
        let moves = vec!["a1", "a2", "c9", "b1"];
        let emoves = vec!["a2", "c9", "b1"];
        let mut actual = State19::new();
        let mut expected = State19::new();
        for mv in moves {
            actual.play_str(mv).unwrap();
        }
        expected.play(Turn::Pass).unwrap();
        for mv in emoves {
            expected.play_str(mv).unwrap();
        }
        println!("{}\n\n{}", actual, expected);
        assert_eq!(format!("\n{}\n", actual), format!("\n{}\n", expected));
    }

    #[test]
    fn captures_free_liberties() {
        let moves = vec!["a1", "a2", "c9", "b1"];
        let emoves = vec!["a2", "c9", "b1"];
        let mut actual = State19::new();
        let mut expected = State19::new();
        for mv in moves {
            actual.play_str(mv).unwrap();
        }
        expected.play(Turn::Pass).unwrap();
        for mv in emoves {
            expected.play_str(mv).unwrap();
        }
        let Pos19(a2_usize) = Pos19::parse("a2");
        let a2_group = actual.group_index.get(a2_usize).unwrap().clone();
        let a2_liberties = actual.liberties.get(a2_group).unwrap();
        assert_eq!(a2_liberties.len(), 3);
    }

    #[test]
    fn suicide_cleared() {
        let moves = vec!["a2", "c9", "b1", "a 1"];
        let emoves = moves[0..3]
            .iter()
            .clone()
            .map(|p| Turn::Of(Pos19::parse(p)));
        let mut actual = State19::new();
        let mut expected = State19::new();
        for mv in moves.iter() {
            actual.play_str(mv).unwrap();
        }

        for mv in emoves {
            expected.play(mv).unwrap();
        }
        println!("{}\n\n{}", actual, expected);
        assert_eq!(format!("\n{}\n", actual), format!("\n{}\n", expected));
    }

    #[test]
    fn basic_score() {
        let moves = vec!["a2", "c9", "b1", "a 1"];
        let mut actual = State19::new();
        for mv in moves.iter() {
            actual.play_str(mv).unwrap();
        }
        let actual_score = actual.score();

        assert_eq!(
            actual_score,
            Score {
                black: 2.0,
                white: 1.0 + actual.komi,
            },
        );
    }
    #[test]
    fn scoring_expands_around_a_wall() {
        let black_moves = (1..19).map(|i| format!("d {}", i));
        let white_moves = (1..19).map(|i| format!("l {}", i));
        let moves = black_moves.zip(white_moves).flat_map(|(a, b)| vec![a, b]);

        let mut actual = State19::new();
        for mv in moves {
            actual.play_str(&mv).unwrap();
        }
        let actual_score = actual.score();

        println!("{}", actual);
        assert_eq!(
            actual_score,
            Score {
                black: 18.0,
                white: 18.0 + actual.komi,
            },
        );
    }
    #[test]
    fn scoring_counts_captured_territory() {
        let black_moves = (1..20).map(|i| format!("d {}", i));
        let white_moves = (1..20).map(|i| format!("l {}", i));
        let moves = black_moves.zip(white_moves).flat_map(|(a, b)| vec![a, b]);

        let mut actual = State19::new();
        for mv in moves {
            actual.play_str(&mv).unwrap();
        }
        let actual_score = actual.score();
        println!("{}", actual);

        assert_eq!(
            actual_score,
            Score {
                black: (19.0 * 4.0),
                white: (19.0 * 8.0) + actual.komi,
            },
        );
    }
}

#[cfg(test)]
mod sfg_replays {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use test::Bencher;
    use super::*;
    use serde_json;
    use serde;
    use gosgf;

    #[test]
    fn game95_throws_no_errors() {
        do_one(PathBuf::from("data/jgdb/./sgf/test/0000/0000095.sgf")).unwrap();
    }

    #[test]
    fn game189_has_6_handicap() {
        let mut file = File::open("data/jgdb/./sgf/test/0000/00000189.sgf").expect(&format!(
            "Couldn't open data/jgdb/./sgf/test/0000/00000189.sgf"
        ));

        let mut buf = String::new();
        BufReader::new(file).read_to_string(&mut buf).unwrap();

        let parse = gosgf::parse_sgf::parse_Collection(&buf).unwrap();
        assert_eq!(parse[0].handicap, 6);
    }
    #[test]
    fn game189_throws_no_errors() {
        do_one(PathBuf::from("data/jgdb/./sgf/test/0000/00000189.sgf")).unwrap();
    }

    #[test]
    fn game95_matches_expectation() {
        let moves = File::open("src/core/test/game95_moves.json").unwrap();
        let parse: Vec<Turn> = serde_json::from_reader(moves).unwrap();
        let mut expectation = String::new();
        BufReader::new(File::open("src/core/test/game95_expectation").unwrap())
            .read_to_string(&mut expectation)
            .unwrap();

        let mut game = State19::new();
        game.next_player = Player::White;
        for turn in parse {
            println!("{:?}", turn);
            game.play(turn);
            println!("{}", game);
        }
        let actual = format!("{}", game);
        println!("{}", actual);
        println!("{}", expectation);

        assert_eq!(actual, expectation);
    }

    #[test]
    fn game189_matches_expectation() {
        let moves = File::open("src/core/test/game189_moves.json").unwrap();
        let parse: Vec<Turn> = serde_json::from_reader(moves).unwrap();
        let mut expectation = String::new();
        BufReader::new(File::open("src/core/test/game189_expectation").unwrap())
            .read_to_string(&mut expectation)
            .unwrap();

        let mut game = State19::new();
        game.next_player = Player::White;
        for turn in parse {
            println!("{:?}", turn);
            game.play(turn).unwrap();
            println!("{}", game);
        }
        let actual = format!("{}", game);
        println!("{}", actual);
        println!("{}", expectation);

        // assert_eq!(actual, expectation);
    }

    #[test]
    fn many_sgf_games_no_illegal_move_errors() {
        let jgdb = PathBuf::from("data/jgdb");
        let filefilename = "data/jgdb/all.txt";

        let filefile = File::open(filefilename).expect("Couldn't open filefile");
        for fname in BufReader::new(filefile).lines().take(10000) {
            let path = jgdb.join(PathBuf::from(fname.unwrap()));
            do_one(path).unwrap();
        }
    }

    fn do_one(path: PathBuf) -> Result<(), String> {
        let mut file = File::open(path.clone()).expect(&format!("Couldn't open path {:?}", path));

        let mut buf = String::new();
        BufReader::new(file).read_to_string(&mut buf).unwrap();

        match gosgf::parse_sgf::parse_Collection(&buf) {
            Ok(parse) => {
                let mut board = State19::init_from_sgf(&parse[0]);
                let turns = parse[0]
                    .main_line()
                    .into_iter()
                    .map(|sgfmove| Turn::from_sgf(sgfmove))
                    .collect::<Vec<Turn>>();

                println!("{:?}", turns);
                for (i, turn) in turns.iter().enumerate() {
                    println!("{}. {:?}", i + 1, turn);
                    println!("{}", board);

                    match board.play(turn.clone()) {
                        Err(err) => {
                            println!("----------------------------------------------------");
                            println!("{}", path.to_string_lossy());
                            println!("Move error {:?} for move {:?}", err, turn);
                            // println!("{:?}", parse[0]);
                            println!("{}", board);
                            println!("{}", board.serialize());
                            println!("----------------------------------------------------");

                            return Err(format!("Move error {:?} for move {:?}", err, turn));
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
                // return Err(format!("parse error {:?}", err));
            }
        }
        Ok(())
    }

    #[bench]
    fn replay_1000_games(b: &mut Bencher) {
        let jgdb = PathBuf::from("data/jgdb");
        let filefilename = "data/jgdb/all.txt";

        let filefile = File::open(filefilename).expect("Couldn't open filefile");
        for fname in BufReader::new(filefile).lines().take(10) {
            let path = jgdb.join(PathBuf::from(fname.unwrap()));
            do_one_bench(b, path);
        }
    }

    fn do_one_bench(b: &mut Bencher, path: PathBuf) -> Result<(), String> {
        let mut file = File::open(path.clone()).expect(&format!("Couldn't open path {:?}", path));

        let mut buf = String::new();
        BufReader::new(file).read_to_string(&mut buf).unwrap();

        match gosgf::parse_sgf::parse_Collection(&buf) {
            Ok(parse) => {
                let mut board = State19::init_from_sgf(&parse[0]);
                let turns = parse[0]
                    .main_line()
                    .into_iter()
                    .map(|sgfmove| Turn::from_sgf(sgfmove))
                    .collect::<Vec<Turn>>();

                let mut ret: Result<(), String> = Ok(());
                b.iter(|| {
                    for turn in &turns {
                        match board.play(turn.clone()) {
                            Err(err) => {
                                println!("----------------------------------------------------");
                                println!("{}", path.to_string_lossy());
                                println!("Move error {:?} for move {:?}", err, turn);
                                // println!("{:?}", parse[0]);
                                println!("{}", board);
                                println!("{}", board.serialize());
                                println!("----------------------------------------------------");

                                ret = Err(format!("Move error {:?} for move {:?}", err, turn));
                                break;
                            }
                            _ => {}
                        }
                    }
                });
            }
            Err(err) => {
                println!("----------------------------------------------------");
                println!("{}", path.to_string_lossy());
                println!("parse error {:?}", err);
                println!("----------------------------------------------------");
                return Err(format!("parse error {:?}", err));
            }
        }
        Ok(())
    }

}
