// Types that represent the state of a game of tic tac toe

use std::fmt;
use tensorflow as tf;
use crate::{game, game::GameStatus};

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum MoveError {
    Occupied,
}

#[derive(Clone, Debug, PartialEq)]
struct ParseError {
    msg: String,
}

#[derive(Debug)]
pub enum RootErrorCause {
    Tf(tf::Status),
    Io(::std::io::Error),
}

#[derive(Debug)]
pub struct TicTacToeError {
    pub msg: String,
    pub root_error: RootErrorCause,
}

impl From<tf::Status> for TicTacToeError {
    fn from(e: tf::Status) -> TicTacToeError {
        TicTacToeError {
            msg: "Tensorflow returned an error.".to_string(),
            root_error: RootErrorCause::Tf(e),
        }
    }
}

impl From<::std::io::Error> for TicTacToeError {
    fn from(e: ::std::io::Error) -> TicTacToeError {
        TicTacToeError {
            msg: "IO error.".to_string(),
            root_error: RootErrorCause::Io(e),
        }
    }
}

pub struct TicTacToe {
    all_actions: Vec<usize>,
}
impl TicTacToe {
    fn new() -> Self {
        TicTacToe {
            all_actions: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }
}
impl game::Game for TicTacToe {
    type State = State;

    // All possible actions that can be played in a game
    // This will be called once at the start of the game and used throughout the game
    fn action_count(&self) -> usize {
        9
    }
    // All the action indexes that are legal for a given State
    // nonlegal actions will be forced to 0 probability by the search engine
    fn legal_actions(&self, state: &Self::State) -> Vec<bool> {
        let mut x = Vec::with_capacity(9);
        for i in 0..9 {
            x[i] = !state.board[0][i] && !state.board[1][i];
        }
        x
    }
    // The given a state and an action on that state, the successor state
    fn successor(&self, state: &Self::State, action: usize) -> Self::State {
        let mut clone = state.clone();
        clone.play(action).unwrap();
        clone
    }

    fn status(&self, state: &Self::State) -> game::GameStatus {
        state.status
    }

    // All the symmetrical training examples, these will be packed into the training example output
    // fn symmetries(&self, hypotheses: &Vec<f32>) -> Vec<(Vec<u8>, Vec<f32>)> {
    //     vec![]
    // }
}

pub struct FeatureBytesIter<'a> {
    i: usize,
    state_ref: &'a State,
}

impl<'a> std::iter::Iterator for FeatureBytesIter<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let i = self.i;
        self.i += 1;
        match self.i {
            0...8 => Some(self.state_ref.board[0][i as usize] as u8),
            9...17 => Some(self.state_ref.board[1][i as usize] as u8),
            18 => Some(self.state_ref.next_player),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Default)]
#[repr(C)]
pub struct State {
    pub board: [[bool; 9]; 2],
    pub next_player: u8,
    pub status: GameStatus,
    pub plys: usize,
}
impl State {
    pub fn new() -> Self {
        Self {
            board: [[false; 9]; 2],
            next_player: 0,
            status: GameStatus::InProgress,
            plys: 0,
        }
    }
    fn to_mark(player: u8) -> String {
        match player {
            0 => String::from("x"),
            _ => String::from("o"),
        }
    }
    fn from_mark(c: char) -> usize {
        match c {
            'x' => 0,
            'o' => 1,
            _ => panic!("unknown char '{}'", c),
        }
    }
    fn from_str(s: &str) -> Result<Self, ParseError> {
        // whitespace is ignored, valid chars are 'x', 'o', "_"
        let mut val = Self::new();
        let mut plys = 0;
        let mut count = 0;
        let mut winner = GameStatus::InProgress;
        for (i, c) in s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '|')
            .enumerate()
            .take(9)
        {
            match c {
                'x' => {
                    val.place_and_check_winner(i, 0).map_err(|err| ParseError {
                        msg: format!("{:?} when adding move {} @ {}", err, c, i),
                    })?;
                    if winner == GameStatus::InProgress {
                        winner = val.status;
                    }
                    plys += 1;
                }
                'o' => {
                    val.place_and_check_winner(i, 1).map_err(|err| ParseError {
                        msg: format!("{:?} when parsing move {} @ {}", err, c, i),
                    })?;
                    if winner == GameStatus::InProgress {
                        winner = val.status;
                    }
                    plys += 1;
                }
                '_' => {}
                _ => {
                    return Err(ParseError {
                        msg: format!("didn't recognize character {}", c),
                    });
                }
            }
            count += 1;
        }
        if count < 9 {
            return Err(ParseError {
                msg: format!("{} only contained {} marks", s, count),
            });
        }

        val.plys = plys;
        val.status = winner;
        val.next_player = (plys % 2) as u8;
        trace!(
            "{} plys have been played. NExt player is {}",
            val.plys,
            val.next_player
        );

        Ok(val)
    }

    pub fn play(&mut self, idx: usize) -> Result<(), MoveError> {
        self.place_and_check_winner(idx, self.next_player)?;
        self.next_player = (self.next_player + 1) % 2;
        Ok(())
    }
    fn place_unchecked(&mut self, idx: usize, player: u8) -> Result<(), MoveError> {
        if self.board[0][idx] || self.board[1][idx] {
            trace!(
                "Tried to place {} at {} but that was occupied \n{}",
                Self::to_mark(player),
                idx,
                self
            );
            return Err(MoveError::Occupied);
        }
        self.board[player as usize][idx] = true;
        Ok(())
    }
    fn place_and_check_winner(&mut self, idx: usize, player: u8) -> Result<(), MoveError> {
        self.place_unchecked(idx, player)?;
        if self.check_winner(idx, player) {
            // trace!("{} (Player {} won)\n", self, Self::to_mark(player));
            self.status = GameStatus::LastPlayerWon;
            return Ok(());
        }
        // trace!("{} at {}\n{}\n", Self::to_mark(player), idx, self);
        self.plys += 1;
        self.status = match self.plys {
            9 => GameStatus::Draw,
            _ => GameStatus::InProgress,
        };
        Ok(())
    }

    // did this move win the game for the one who played it?
    fn check_winner(&self, idx: usize, player: u8) -> bool {
        self.check_row(idx, player) || self.check_col(idx, player) || self.check_diags(idx, player)
    }

    fn all(&self, _t: &str, i: u8, first: usize, second: usize, third: usize) -> bool {
        let matches = self.board[i as usize][first]
            && self.board[i as usize][second]
            && self.board[i as usize][third];
        matches
    }
    fn check_row(&self, idx: usize, player: u8) -> bool {
        let o = (idx / 3) * 3;
        self.all(&"row", player, 0 + o, 1 + o, 2 + o)
    }
    fn check_col(&self, idx: usize, player: u8) -> bool {
        let o = (idx + 3) % 3;

        self.all(&"col", player, 0 + o, 3 + o, 6 + o)
    }
    fn check_diags(&self, idx: usize, player: u8) -> bool {
        if (idx + 4) % 4 == 0 && self.all(&"nw-se", player, 0, 4, 8) {
            return true;
        }
        match idx {
            2 | 4 | 6 => self.all(&"sw-ne", player, 2, 4, 6),
            _ => false,
        }
    }
    fn mark(&self, idx: usize) -> String {
        if self.board[0][idx] {
            return Self::to_mark(0);
        }
        if self.board[1][idx] {
            return Self::to_mark(1);
        }
        String::from(" ")
    }
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..3 {
            let o = i * 3;
            f.write_str(&format!(
                "|{}|{}|{}|{}",
                self.mark(o),
                self.mark(o + 1),
                self.mark(o + 2),
                if i < 2 { "\n" } else { "" },
            ))?;
        }
        f.write_str("")
    }
}

impl crate::game::GameState for State {
    fn feature_bytes<'a>(&'a self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(19);
        for b in &self.board {
            for &p in b {
                vec.push(p as u8);
            }
        }
        vec.push(self.next_player);

        vec
    }

}