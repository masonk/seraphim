use std::fmt;
use indoc;
use search;
use flexi_logger;
use std::sync::{Once, ONCE_INIT};
use search::GameExpert;
use search::GameResult;
static _INIT: Once = ONCE_INIT;

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
enum Mark {
    Circle,
    Cross,
    Empty,
}
impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Mark::Circle => f.write_str("o"),
            &Mark::Cross => f.write_str("x"),
            &Mark::Empty => f.write_str(" "),
        }
    }
}
// impl Mark {
//     fn player(&self) -> Player {
//         match self {
//             &Mark::Circle => Player::Circle,
//             &Mark::Cross => Player::Cross,
//             _ => panic!("No player for Empty"),
//         }
//     }
// }
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
enum Player {
    Circle,
    Cross,
}
impl Player {
    fn other(&self) -> Self {
        match self {
            &Player::Circle => Player::Cross,
            &Player::Cross => Player::Circle,
        }
    }
    fn mark(&self) -> Mark {
        match self {
            &Player::Circle => Mark::Circle,
            &Player::Cross => Mark::Cross,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
enum MoveError {
    Occupied,
}

#[derive(Clone, Debug, PartialEq)]
struct ParseError {
    msg: String,
}

#[derive(Clone, Debug, PartialEq, Copy, Hash)]
struct TicTacToeState {
    board: [Mark; 9],
    next_player: Player,
    status: GameResult,
    plys: usize,
}
impl TicTacToeState {
    // whitespace is ignored, valid chars are 'x', 'o', "_"
    pub fn new_game() -> Self {
        Self {
            board: [Mark::Empty; 9],
            next_player: Player::Circle,
            status: GameResult::InProgress,
            plys: 0,
        }
    }
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut val = Self::new_game();
        let mut plys = 0;
        let mut count = 0;
        let mut winner = GameResult::InProgress;
        for (i, c) in s.chars().filter(|c| !c.is_whitespace()).enumerate().take(9) {
            match c {
                'x' => {
                    val.place_and_check_winner(i, Mark::Cross)
                        .map_err(|err| ParseError {
                            msg: format!("{:?} when adding move {} @ {}", err, c, i),
                        })?;
                    if winner == GameResult::InProgress {
                        winner = val.status;
                    }
                    plys += 1;
                }
                'o' => {
                    val.place_and_check_winner(i, Mark::Circle)
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
        if plys % 2 == 0 {
            val.next_player = Player::Cross;
        } else {
            val.next_player = Player::Circle;
        }

        Ok(val)
    }

    fn play(&mut self, idx: usize) -> Result<(), MoveError> {
        self.place_and_check_winner(idx, self.next_player.mark())?;
        self.next_player = self.next_player.other();
        Ok(())
    }
    fn place_unchecked(&mut self, idx: usize, mark: Mark) -> Result<(), MoveError> {
        if self.board[idx] != Mark::Empty {
            trace!(
                "Tried to place {} at {} but that was occupied by {}\n{}",
                mark,
                idx,
                self.board[idx],
                self
            );
            return Err(MoveError::Occupied);
        }
        self.board[idx] = mark;
        Ok(())
    }
    fn place_and_check_winner(&mut self, idx: usize, mark: Mark) -> Result<(), MoveError> {
        self.place_unchecked(idx, mark)?;
        if self.check_winner(idx, mark) {
            trace!("{} at {} won the game \n{}", mark, idx, self);
            self.status = GameResult::LastPlayerWon;
            return Ok(());
        }
        trace!("{} at {}\n{}", mark, idx, self);
        self.plys += 1;
        self.status = match self.plys {
            9 => GameResult::TerminatedWithoutResult,
            _ => GameResult::InProgress,
        };
        Ok(())
    }

    fn get(&self, idx: usize) -> Mark {
        self.board[idx]
    }

    fn matches(&self, idx: usize, mark: Mark) -> bool {
        self.get(idx) == mark
    }
    // did this move win the game for the one who played it?
    fn check_winner(&self, idx: usize, mark: Mark) -> bool {
        self.check_row(idx, mark) || self.check_col(idx, mark) || self.check_diags(idx, mark)
    }

    fn match_three(&self, t: &str, mark: Mark, first: usize, second: usize, third: usize) -> bool {
        let matches =
            self.matches(first, mark) && self.matches(second, mark) && self.matches(third, mark);

        let winner = if matches { "[Winner]:" } else { "" };
        trace!(
            "{}{} {}: {} {} {}",
            winner,
            t,
            mark,
            self.get(first),
            self.get(second),
            self.get(third)
        );

        matches
    }
    fn check_row(&self, idx: usize, mark: Mark) -> bool {
        let o = (idx / 3) * 3;
        self.match_three(&"row", mark, 0 + o, 1 + o, 2 + o)
    }
    fn check_col(&self, idx: usize, mark: Mark) -> bool {
        let o = (idx + 3) % 3;

        self.match_three(&"col", mark, 0 + o, 3 + o, 6 + o)
    }
    fn check_diags(&self, idx: usize, mark: Mark) -> bool {
        if (idx + 4) % 4 == 0 && self.match_three(&"nw-se", mark, 0, 4, 8) {
            return true;
        }
        match idx {
            2 | 4 | 6 => self.match_three(&"sw-ne", mark, 2, 4, 6),
            _ => false,
        }
    }
}
impl fmt::Display for TicTacToeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..3 {
            let o = i * 3;
            f.write_str(&format!(
                "|{}|{}|{}|\n",
                self.board[o],
                self.board[o + 1],
                self.board[o + 2]
            ))?;
        }
        f.write_str("")
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct TTTGe {}
impl GameExpert<TicTacToeState, usize> for TTTGe {
    fn root(&self) -> TicTacToeState {
        TicTacToeState::new_game()
    }

    fn legal_actions(&self, state: &TicTacToeState) -> (Vec<usize>, Vec<f32>) {
        let actions = state
            .board
            .iter()
            .enumerate()
            .filter(|&(_, s)| match s {
                &Mark::Empty => true,
                _ => false,
            })
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        let len = actions.len() as f32;
        // For the game of tic tac toe, no real expertise is needed.
        // Just consider all actions equally probable.
        // MCTS can fully exhaust the state space of TTT in readouts of 20-30 games per move.
        let probs = actions.iter().map(|_| 1.0 / len).collect::<Vec<f32>>();

        (actions, probs)
    }
    fn apply(&mut self, state: &TicTacToeState, action: &usize) -> TicTacToeState {
        let mut clone = state.clone();
        if *state != clone {}
        match clone.play(*action) {
            Ok(__) => {}
            Err(err) => {
                println!("{}->\n{}\naction: {}", state, clone, action);
                panic!("{:?}", err);
            }
        }
        clone
    }
    fn to_win(&self, _: &TicTacToeState) -> f32 {
        0.5
    }
    fn result(&self, state: &TicTacToeState) -> search::GameResult {
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
    fn init_game_expert() {
        _setup_test();
        let game_expert = TTTGe {};
        let mut game = TicTacToeState::new_game();
        let mut options = search::SearchTreeOptions::defaults();
        options.readouts = 100;
        options.tempering_point = 0;
        let mut search = search::SearchTree::init_with_options(game_expert, options);

        loop {
            if let GameResult::InProgress = game.status {
                let next = search.read_and_apply();
                game.play(next).unwrap();
                println!("{}", game);
            } else {
                trace!("{:?}", game.status);
                break;
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
        let state = TicTacToeState::from_str(indoc!(
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
        let state = TicTacToeState::from_str(indoc!(
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
        let state = TicTacToeState::from_str(indoc!(
            "\
            o x o
            _ x x
            o o o"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, Player::Cross);
    }

    #[test]
    fn x_wins_col() {
        _setup_test();
        let state = TicTacToeState::from_str(indoc!(
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
        let state = TicTacToeState::from_str(indoc!(
            "\
            x _ x
            o x o
            _ o x"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, Player::Circle);
    }

    #[test]
    fn o_wins_ne_diag() {
        _setup_test();
        let state = TicTacToeState::from_str(indoc!(
            "\
            _ x o
            _ o _
            o x x"
        )).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, Player::Cross);
    }
}
