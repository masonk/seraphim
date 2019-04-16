use flexi_logger;
use std::sync::{Once, ONCE_INIT};
static _INIT: Once = ONCE_INIT;

use search;
use search::GameStatus;
use tictactoe;
use tictactoe::state::State;
/*
* This game expert suggests moves by assigning equal probability to all legal moves.
*/
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct NaiveGameExpert {}
impl NaiveGameExpert {
    pub fn new() -> Self {
        NaiveGameExpert {}
    }
}
impl search::GameExpert<State, usize> for NaiveGameExpert {
    fn hypotheses(&mut self, state: &State) -> search::Hypotheses<usize> {
        let prob = 1.0 / (9 - state.plys) as f32;

        let legal_actions = (0..9)
            .into_iter()
            .filter(|&i| !(state.board[0][i] || state.board[1][i]))
            .map(|i| (i, prob))
            .collect::<Vec<(usize, f32)>>();

        search::Hypotheses {
            legal_actions,
            to_win: 0.5,
        }
    }
    fn next(&mut self, state: &State, action: &usize) -> State {
        let mut clone = state.clone();
        clone.play(*action).unwrap();
        clone
    }
    fn max_actions(&mut self) -> usize {
        9
    }
}
