use flexi_logger;
use search;
use search::GameResult;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Once, ONCE_INIT};
static _INIT: Once = ONCE_INIT;
use tensorflow as tf;

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
    pub fn new() -> Self {
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
        let mut winner = GameResult::InProgress;
        for (i, c) in s.chars()
            .filter(|c| !c.is_whitespace() && *c != '|')
            .enumerate()
            .take(9)
        {
            match c {
                'x' => {
                    val.place_and_check_winner(i, 0).map_err(|err| ParseError {
                        msg: format!("{:?} when adding move {} @ {}", err, c, i),
                    })?;
                    if winner == GameResult::InProgress {
                        winner = val.status;
                    }
                    plys += 1;
                }
                'o' => {
                    val.place_and_check_winner(i, 1).map_err(|err| ParseError {
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
    fn place_unchecked(&mut self, idx: usize, player: usize) -> Result<(), MoveError> {
        if self.board[0][idx] || self.board[1][idx] {
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
        let matches = self.board[i][first] && self.board[i][second] && self.board[i][third];
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

type BoxError = Box<::std::error::Error>;

#[derive(Debug)]
pub struct DnnGameExpert {
    root_state: State,
    graph: tf::Graph,
    session: tf::Session,
}
impl DnnGameExpert {
    fn load_graph(filename: &str) -> Result<tf::Graph, BoxError> {
        if !Path::new(filename).exists() {
            return Err(Box::new(
                tf::Status::new_set(
                    tf::Code::NotFound,
                    &format!(
                        "source bin/activate && python src/tictactoe/simple_net.py \
                         to generate {} and try again.",
                        filename
                    ),
                ).unwrap(),
            ));
        }

        let mut proto = Vec::new();
        File::open(filename)?.read_to_end(&mut proto)?;
        let mut graph = tf::Graph::new();

        graph.import_graph_def(&proto, &tf::ImportGraphDefOptions::new())?;
        Ok(graph)
    }

    pub fn from_saved_model(graph_filename: &str, model_filename: &str) -> Result<Self, BoxError> {
        trace!("Attemping to load graph from '{}'", graph_filename);
        let graph = Self::load_graph(graph_filename)?;
        trace!("...success.");

        let mut session = tf::Session::new(&tf::SessionOptions::new(), &graph)?;
        let op_load = graph.operation_by_name_required("save/restore_all")?;
        let op_file_path = graph.operation_by_name_required("save/Const")?;
        let file_path_tensor: tf::Tensor<String> = tf::Tensor::from(String::from(model_filename));
        
        {
            let mut step = tf::StepWithGraph::new();
            step.add_input(&op_file_path, 0, &file_path_tensor);
            step.add_target(&op_load);
            trace!("Attemping to load model from '{}' ...", model_filename);
            session.run(&mut step)?;
            trace!("...success.");
        }

        Ok(DnnGameExpert {
            root_state: self::State::new(),
            graph: graph,
            session: session,
        })
    }
    pub fn save_model(graph: &tf::Graph, session: &mut tf::Session, model_filename: &str) -> Result<(), BoxError> {
        let op_save = graph.operation_by_name_required("save/control_dependency")?;
        let op_file_path = graph.operation_by_name_required("save/Const")?;
        let file_path_tensor: tf::Tensor<String> = tf::Tensor::from(String::from(model_filename));
        let mut saver_step = tf::StepWithGraph::new();
        saver_step.add_input(&op_file_path, 0, &file_path_tensor);
        saver_step.add_target(&op_save);

        trace!("Attemping to save model to '{}'", model_filename);
        session.run(&mut saver_step)?;
        trace!("...success)");
        Ok(())
    }

    pub fn init_with_random_weights(graph_filename: &str, model_filename: &str) -> Result<Self, BoxError> {
        trace!("Attemping to loading graph from '{}'", graph_filename);
        let graph = Self::load_graph(graph_filename)?;
        trace!("...success");

        let mut session = tf::Session::new(&tf::SessionOptions::new(), &graph)?;
        let op_init = graph.operation_by_name_required("init")?;
        let mut init_step = tf::StepWithGraph::new();
        init_step.add_target(&op_init);

        trace!("Attemping to initialize a new model at '{}'", model_filename);
        session.run(&mut init_step)?;
        trace!("...successs.");
    
        Self::save_model(&graph, &mut session, model_filename);
        Self::from_saved_model(graph_filename, model_filename)
    }

    fn state_tensor(state: &State) -> tf::Tensor<bool> {
        let mut x = tf::Tensor::new(&[1, 19]);
        for i in 0..2 {
            for j in 0..9 {
                x[i * 9 + j] = state.board[i][j];
            }
        }

        x[18] = match state.next_player {
            0 => false,
            _ => true,
        };
        x
    }

    pub fn train_and_save(&mut self, n: usize, model_filepath: &str) -> Result<(), BoxError> {
        trace!("Attemping to load training ops...");
        let op_x = self.graph.operation_by_name_required("x")?;
        let op_y_true = self.graph.operation_by_name_required("y_true")?;
        let op_train = self.graph.operation_by_name_required("train")?;
        trace!("..succcess.");

        let mut options = search::SearchTreeOptions::defaults();
        options.readouts = 1500;
        options.tempering_point = 1;
        options.cpuct = 0.1;
        trace!("Beginning search & train with {:?}", options);

        for i in 0..n {
            trace!("game {}", i);
            let initial_search_state = State::new();
            let mut game = initial_search_state.clone();
            let mut search = search::SearchTree::init_with_options(initial_search_state, options.clone());
            loop {
                if let search::GameResult::InProgress = game.status {
                    let next = search.read_and_apply(self);
                    game.play(next).unwrap();

                    // x is game state
                    // next goes into y_true for training
                    let x = Self::state_tensor(&game);
                    let mut y_true = tf::Tensor::new(&[1, 9]);
                    y_true[next] = 1.0f32;

                    let mut train_step = tf::StepWithGraph::new();
                    train_step.add_input(&op_x, 0, &x);
                    train_step.add_input(&op_y_true, 0, &y_true);
                    train_step.add_target(&op_train);
                    self.session.run(&mut train_step)?;
                } else {
                    break;
                }
            }
        }
        Self::save_model(&self.graph, &mut self.session, model_filepath)?;

        Ok(())
    }
}
impl search::GameExpert<State, usize> for DnnGameExpert {
    // TODO: The AGZ paper minibatches the request for expert policies
    // into batches of 8 hypotheses. There should be a way of batching these requests
    // This has to come after multi-threading the search, since threads block
    // while waiting for their batch to accumulate.
    fn hypotheses(&mut self, state: &State) -> search::Hypotheses<usize> {

        trace!("{}", state);
        let op_x = self.graph.operation_by_name_required("x").unwrap();
        let softmax = self.graph.operation_by_name_required("softmax").unwrap();
        trace!("...success.");

        let state_tensor = Self::state_tensor(state);
        let mut legal_actions : Vec<(usize, f32)> = vec![];
        {
            let mut inference_step = tf::StepWithGraph::new();
            inference_step.add_input(&op_x, 0, &state_tensor);
            let softmax_output_token = inference_step.request_output(&softmax, 0);

            self.session.run(&mut inference_step).expect("failed to run inference step");

            let inferences : tf::Tensor<f32> = inference_step.take_output(softmax_output_token).unwrap();
            trace!("raw inferences:");
            for i in 0..3 {
                let o = i * 3;
                trace!("{0: <w$.w$} | {1: <w$.w$} | {2: <w$.w$}", inferences[o], inferences[o+1], inferences[o+2], w=5);
            }

            let mut legal_probability = 0.0;
            for i in (0..9).into_iter() {
                let is_legal = !(state.board[0][i] || state.board[1][i]);
                
                if is_legal {
                    legal_actions.push((i, inferences[i]));
                    legal_probability += inferences[i];
                }
            }
            // ax = 1
            // a = 1 / x
            let scale = 1.0 / legal_probability; // there must always be at least one legal action
            for &mut(_, ref mut prior) in legal_actions.iter_mut() {
                *prior *= scale;
            }
            trace!("redistributed inferences:");
            let trcinf = (0..9).into_iter().map(|i| {
               if state.board[0][i] || state.board[1][i] {
                    0.0
               } else {
                    inferences[i] * scale
               }
            }).collect::<Vec<f32>>();

            for i in 0..3 {
                let o = i * 3;
                trace!("{0: <w$.w$} | {1: <w$.w$} | {2: <w$.w$}", trcinf[o], trcinf[o+1], trcinf[o+2], w = 3);
            }
        }

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

    fn result(&self, state: &State) -> search::GameResult {
        state.status
    }
}

/*
* This game expert suggests moves by assigning equal probability to all legal moves.
*/
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct DumbGameExpert {}
impl DumbGameExpert {
    pub fn new() -> Self {
        DumbGameExpert {}
    }
}
impl search::GameExpert<State, usize> for DumbGameExpert {
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
            let mut game_expert = DumbGameExpert::new();
            let mut game = State::new();
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1000;
            options.tempering_point = 1;
            options.cpuct = 2.0;
            let mut search = search::SearchTree::init_with_options(State::new(), options);
            loop {
                if let GameResult::InProgress = game.status {
                    let next = search.read_and_apply(&mut game_expert);
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
                let mut game_expert = DumbGameExpert::new();
                let mut game = State::new();
                let mut options = search::SearchTreeOptions::defaults();
                options.readouts = *readouts;
                options.tempering_point = 1; // start from a random position, then always play the best move
                options.cpuct = 3.0;
                let mut search = search::SearchTree::init_with_options(State::new(), options);
                loop {
                    if let GameResult::InProgress = game.status {
                        let next = search.read_and_apply(&mut game_expert);
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
        let game = State::from_str(
            "\
            |_|_|o|
            |o|x|_|
            |x|_|o|",
        ).expect("Couldn't parse board.");
        let mut game_expert = DumbGameExpert::new();
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
            let mut game_expert = DumbGameExpert::new();
            let mut game = State::new();
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1500;
            options.tempering_point = 1;
            options.cpuct = 0.1;
            let mut search = search::SearchTree::init_with_options(State::new(), options);
            loop {
                if let GameResult::InProgress = game.status {
                    let next = search.read_and_apply(&mut game_expert);
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
        let state = State::from_str(
            "\
            _ _ _
            _ _ _
            _ _ _",
        ).expect("Couldn't parse an empty board");
    }

    #[test]
    fn parse_a_board() {
        _setup_test();
        let state = State::from_str(
            "\
            o x o
            _ x _
            _ o _",
        ).expect("Couldn't parse");
    }

    #[test]
    fn o_wins_row() {
        _setup_test();
        let state = State::from_str(
            "\
            o x x
            _ x x
            o o o",
        ).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 0);
    }

    #[test]
    fn x_wins_col() {
        _setup_test();
        let state = State::from_str(
            "\
            o x o
            _ x _
            _ x _",
        ).expect("Couldn't parse");

        trace!("{}", state);
    }

    #[test]
    fn x_wins_nw_diag() {
        _setup_test();
        let state = State::from_str(
            "\
            x _ x
            o x o
            _ o x",
        ).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 1);
    }

    #[test]
    fn o_wins_ne_diag() {
        _setup_test();
        let state = State::from_str(
            "\
            _ x o
            _ o _
            o x x",
        ).expect("Couldn't parse");

        trace!("{}", state);

        assert_eq!(state.status, GameResult::LastPlayerWon);
        assert_eq!(state.next_player, 0);
    }
}
