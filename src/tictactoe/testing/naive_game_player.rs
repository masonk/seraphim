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
mod test {
    use super::{_setup_test, search, GameStatus, State};

    #[test]
    // #[ignore]
    fn increasing_readouts_improves_play() {
        _setup_test();
        let mut draws: Vec<usize> = Vec::new();
        let n = 100;
        let readouts = [50, 100, 200, 400, 800];
        for readouts in readouts.iter() {
            let mut draw = 0;
            for _ in 0..n {
                let mut game_expert = NaiveGameExpert::new();
                let mut game = State::new();
                let mut options = search::SearchTreeOptions::defaults();
                options.readouts = *readouts;
                options.tempering_point = 1; // start from a random position, then always play the best move
                options.cpuct = 3.0;
                let mut search = search::SearchTree::init_with_options(State::new(), options);
                loop {
                    if let GameStatus::InProgress = game.status {
                        let next = search.read_and_apply(&mut game_expert);
                        game.play(next).unwrap();
                    } else {
                        if game.status == GameStatus::TerminatedWithoutResult {
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
    // #[ignore]
    fn increasing_readouts_improves_play() {
        _setup_test();
        let mut draws: Vec<usize> = Vec::new();
        let n = 100;
        let readouts = [50, 100, 200, 400, 800];
        for readouts in readouts.iter() {
            let mut draw = 0;
            for _ in 0..n {
                let mut game_expert = NaiveGameExpert::new();
                let mut game = State::new();
                let mut options = search::SearchTreeOptions::defaults();
                options.readouts = *readouts;
                options.tempering_point = 1; // start from a random position, then always play the best move
                options.cpuct = 3.0;
                let mut search = search::SearchTree::init_with_options(State::new(), options);
                loop {
                    if let GameStatus::InProgress = game.status {
                        let next = search.read_and_apply(&mut game_expert);
                        game.play(next).unwrap();
                    } else {
                        if game.status == GameStatus::TerminatedWithoutResult {
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
        let game = State::from_str(
            "\
            |_|_|o|
            |o|x|_|
            |x|_|o|",
        ).expect("Couldn't parse board.");
        let mut game_expert = NaiveGameExpert::new();
        let options = search::SearchTreeOptions {
            readouts: 1500,
            tempering_point: 0,
            cpuct: 0.5,
        };

        let mut search = search::SearchTree::init_with_options(game.clone(), options);
        let next = search.read_and_apply(&mut game_expert);
        assert_eq!(next, 5);
    }

    #[test]
    fn play_10() {
        _setup_test();

        for _ in 0..10 {
            let mut game_expert = NaiveGameExpert::new();
            let mut game = State::new();
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1500;
            options.tempering_point = 1;
            options.cpuct = 0.1;
            let mut search = search::SearchTree::init_with_options(State::new(), options);
            loop {
                if let GameStatus::InProgress = game.status {
                    let next = search.read_and_apply(&mut game_expert);
                    game.play(next).unwrap();
                } else {
                    break;
                }
            }
        }
    }
}
