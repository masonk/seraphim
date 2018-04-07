use flexi_logger;
use indoc;
use search;
use search::GameResult;
use std::fmt;
use std::sync::{Once, ONCE_INIT};
static _INIT: Once = ONCE_INIT;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum MoveError {
    Occupied,
}

#[derive(Clone, Debug, PartialEq)]
struct ParseError {
    msg: String,
}

#[derive(Clone, Debug, PartialEq, Copy, Hash)]
pub struct State {
    pub board: [[bool; 9]; 2],
    pub next_player: usize,
    pub status: GameResult,
    pub plys: usize,
}
impl State {
    pub fn new_game() -> Self {
        Self {
            board: [[false; 9]; 2],
            next_player: 0,
            status: GameResult::InProgress,
            plys: 0,
        }
    }
    fn to_mark(player: usize) -> String {
        match player {
            0 => String::from("x"),
            _ => String::from("o")
        }
    }
    fn from_mark(c: char) -> usize {
        match c {
            'x' => 0,
            'o' => 1,
            _ => panic!("unknown char '{}'", c)
        }
    }
    fn from_str(s: &str) -> Result<Self, ParseError> {
        // whitespace is ignored, valid chars are 'x', 'o', "_"
        let mut val = Self::new_game();
        let mut plys = 0;
        let mut count = 0;
        let mut winner = GameResult::InProgress;
        for (i, c) in s.chars()
            .filter(|c| !c.is_whitespace() && *c != '|')
            .enumerate()
            .take(9)
        {
            match c {
                'x' => {
                    val.place_and_check_winner(i, 0)
                        .map_err(|err| ParseError {
                            msg: format!("{:?} when adding move {} @ {}", err, c, i),
                        })?;
                    if winner == GameResult::InProgress {
                        winner = val.status;
                    }
                    plys += 1;
                }
                'o' => {
                    val.place_and_check_winner(i, 1)
                        .map_err(|err| ParseError {
                            msg: format!("{:?} when parsing move {} @ {}", err, c, i),
                        })?;
                    if winner == GameResult::InProgress {
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
        val.next_player = plys % 2;

        Ok(val)
    }

    pub fn play(&mut self, idx: usize) -> Result<(), MoveError> {
        self.place_and_check_winner(idx, self.next_player)?;
        self.next_player = (self.next_player + 1) % 2;
        Ok(())
    }
    fn place_unchecked(&mut self, idx: usize, player: usize) -> Result<(), MoveError> {
        if self.board[0][idx] || self.board[1][idx]  {
            trace!(
                "Tried to place {} at {} but that was occupied \n{}",
                Self::to_mark(player),
                idx,
                self
            );
            return Err(MoveError::Occupied);
        }
        self.board[player][idx] = true;
        Ok(())
    }
    fn place_and_check_winner(&mut self, idx: usize, player: usize) -> Result<(), MoveError> {
        self.place_unchecked(idx, player)?;
        if self.check_winner(idx, player) {
            trace!("Player {} won the game \n{}", player, idx);
            self.status = GameResult::LastPlayerWon;
            return Ok(());
        }
        trace!("{} at {}\n{}", Self::to_mark(player), idx, self);
        self.plys += 1;
        self.status = match self.plys {
            9 => GameResult::TerminatedWithoutResult,
            _ => GameResult::InProgress,
        };
        Ok(())
    }

    // did this move win the game for the one who played it?
    fn check_winner(&self, idx: usize, player: usize) -> bool {
        self.check_row(idx, player) || self.check_col(idx, player) || self.check_diags(idx, player)
    }

    fn all(&self, t: &str, i: usize, first: usize, second: usize, third: usize) -> bool {
        let matches =
            self.board[i][first] && self.board[i][second] && self.board[i][third];
        matches
    }
    fn check_row(&self, idx: usize, player: usize) -> bool {
        let o = (idx / 3) * 3;
        self.all(&"row", player, 0 + o, 1 + o, 2 + o)
    }
    fn check_col(&self, idx: usize, player: usize) -> bool {
        let o = (idx + 3) % 3;

        self.all(&"col", player, 0 + o, 3 + o, 6 + o)
    }
    fn check_diags(&self, idx: usize, player: usize) -> bool {
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
                "|{}|{}|{}|\n",
                self.mark(o),
                self.mark(o + 1),
                self.mark(o + 2)
            ))?;
        }
        f.write_str("")
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct GameExpert {
    root_state: State,
}
impl GameExpert {
    pub fn new(root_state: State) -> Self {
        GameExpert { root_state }
    }
}
impl search::GameExpert<State, usize> for GameExpert {
    fn root(&self) -> State {
        self.root_state
    }

    fn hypotheses(&self, state: &State) -> search::Hypotheses<usize> {
        let actions = (0..9).into_iter()
            .filter(|&i| !(state.board[0][i] || state.board[1][i]))
            .collect::<Vec<usize>>();
        let len = actions.len() as f32;

        let move_probabilities = actions.iter().map(|_| 1.0 / len).collect::<Vec<f32>>();

        search::Hypotheses {
            actions,
            move_probabilities,
            to_win: 0.5,
        }
    }
    fn apply(&mut self, state: &State, action: &usize) -> State {
        let mut clone = state.clone();
        clone.play(*action).unwrap();
        clone
    }

    fn result(&self, state: &State) -> search::GameResult {
        state.status
    }
}

fn _setup_test() {
    _INIT.call_once(|| {
        _init_env_logger();
    });
}

fn _init_env_logger() {
    flexi_logger::Logger::with_env()
        .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap()
}

#[cfg(test)]
mod expert {
    use super::*;

    #[test]
    fn stress_test() {
        _setup_test();
        let mut draw = 0;
        let n = 500;
        for _ in 0..n {
            let game_expert = GameExpert::new(State::new_game());
            let mut game = State::new_game();
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1000;
            options.tempering_point = 1;
            options.cpuct = 2.0;
            let mut search = search::SearchTree::init_with_options(game_expert, options);
            loop {
                if let GameResult::InProgress = game.status {
                    let next = search.read_and_apply();
                    game.play(next).unwrap();
                } else {
                    if game.status == GameResult::TerminatedWithoutResult {
                        draw += 1;
                    }
                    break;
                }
            }
        }
        println!("drew {} / {} games", draw, n);
        assert!(
            (draw as f32) / (n as f32) > 0.95,
            "Most games should draw in a well-played game of Tic Tac Toe"
        );
    }

    #[test]
    fn increasing_readouts_improves_play() {
        _setup_test();
        let mut draws: Vec<usize> = Vec::new();
        let n = 100;
        let readouts = [50, 100, 200, 400, 800];
        for readouts in readouts.iter() {
            let mut draw = 0;
            for _ in 0..n {
                let game_expert = GameExpert::new(State::new_game());
                let mut game = State::new_game();
                let mut options = search::SearchTreeOptions::defaults();
                options.readouts = *readouts;
                options.tempering_point = 1; // start from a random position, then always play the best move
                options.cpuct = 3.0;
                let mut search = search::SearchTree::init_with_options(game_expert, options);
                loop {
                    if let GameResult::InProgress = game.status {
                        let next = search.read_and_apply();
                        game.play(next).unwrap();
                    } else {
                        if game.status == GameResult::TerminatedWithoutResult {
                            draw += 1;
                        }
                        break;
                    }
                }
            }
            draws.push(draw);
        }
        for i in 1..draws.len() {
            assert!(draws[i] >= draws [i-1], "Increasing readouts should increase the number of draws, but it didn't: {:?} draws for readout depths of {:?}", draws, readouts)
        }
    }

    #[test]
    fn search_blocks_immediate_loss() {
        let game = State::from_str(indoc!(
            "\
            |_|_|o|
            |o|x|_|
            |x|_|o|"
        )).expect("Couldn't parse board.");
        let game_expert = GameExpert::new(game.clone());
        let options = search::SearchTreeOptions {
            readouts: 1500,
            tempering_point: 0,
            cpuct: 0.5,
        };

        let mut search = search::SearchTree::init_with_options(game_expert, options);
        let next = search.read_and_apply();
        assert_eq!(next, 5);
    }

    #[test]
    fn play_10() {
        _setup_test();

        for _ in 0..10 {
            let game_expert = GameExpert::new(State::new_game());
            let mut game = State::new_game();
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1500;
            options.tempering_point = 1;
            options.cpuct = 0.1;
            let mut search = search::SearchTree::init_with_options(game_expert, options);
            loop {
                if let GameResult::InProgress = game.status {
                    let next = search.read_and_apply();
                    game.play(next).unwrap();
                } else {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod basic {
    use super::*;

    #[test]
    fn parse_empty_board() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            _ _ _
            _ _ _
            _ _ _"
        )).expect("Couldn't parse an empty board");

        println!("{}", state);
        println!("{:?}", state);
    }

    #[test]
    fn parse_a_board() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            o x o
            _ x _
            _ o _"
        )).expect("Couldn't parse");

        println!("{}", state);
    }

    #[test]
    fn o_wins_row() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            o x o
            _ x x
            o o o"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 1);
    }

    #[test]
    fn x_wins_col() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            o x o
            _ x _
            _ x _"
        )).expect("Couldn't parse");

        trace!("{}", state);
    }

    #[test]
    fn x_wins_nw_diag() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            x _ x
            o x o
            _ o x"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 1);
    }

    #[test]
    fn o_wins_ne_diag() {
        _setup_test();
        let state = State::from_str(indoc!(
            "\
            _ x o
            _ o _
            o x x"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 0);
    }
}
