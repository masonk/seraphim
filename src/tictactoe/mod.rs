use flexi_logger;
use search;
use search::GameResult;
use io;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Once, ONCE_INIT};
static _INIT: Once = ONCE_INIT;
use tensorflow as tf;
use protobuf;
use protobuf::Message;
use std::collections::HashMap;
mod gen;

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
    Io(::std::io::Error)
}

#[derive(Debug)]
pub struct TicTacToeError {
    pub msg: String,
    pub root_error: RootErrorCause
}

impl From<tf::Status> for TicTacToeError {
    fn from(e: tf::Status) -> TicTacToeError {
        TicTacToeError {
            msg: "Tensorflow returned an error.".to_string(),
            root_error: RootErrorCause::Tf(e)
        }
    }
}

impl From<::std::io::Error> for TicTacToeError {
    fn from(e: ::std::io::Error) -> TicTacToeError {
        TicTacToeError {
            msg: "IO error.".to_string(),
            root_error: RootErrorCause::Io(e)
        }
    }
}
pub struct TrainOptions {

}
impl TrainOptions {
    pub fn new() -> Self {
        TrainOptions {}
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Hash)]
#[repr(C)]
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
            // trace!("{} (Player {} won)\n", self, Self::to_mark(player));
            self.status = GameResult::LastPlayerWon;
            return Ok(());
        }
        // trace!("{} at {}\n{}\n", Self::to_mark(player), idx, self);
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


#[derive(Debug)]
pub struct DnnGameExpert {
    root_state: State,
    graph: tf::Graph,
    session: tf::Session,
}
impl DnnGameExpert {
    fn load_graph(filename: &str) -> Result<tf::Graph, TicTacToeError> {
        if !Path::new(filename).exists() {
            return Err(TicTacToeError::from(
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

    pub fn from_saved_model(graph_filename: &str, model_filename: &str) -> Result<Self, TicTacToeError> {
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
    fn save_model_(graph: &tf::Graph, session: &mut tf::Session, model_filename: &str) -> Result<(), TicTacToeError> {
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
    pub fn save_model(&mut self, model_filename: &str) -> Result<(), TicTacToeError> {
        Self::save_model_(&self.graph, &mut self.session, model_filename)
    }


    pub fn init_with_random_weights(graph_filename: &str, model_filename: &str) -> Result<Self, TicTacToeError> {
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
    
        Self::save_model_(&graph, &mut session, model_filename);
        Self::from_saved_model(graph_filename, model_filename)
    }

    fn state_tensor(state: &State) -> tf::Tensor<u8> {
        let mut x = tf::Tensor::new(&[1, 19]);
        for i in 0..2 {
            for j in 0..9 {
                x[i * 9 + j] = state.board[i][j] as u8;
            }
        }

        x[18] = match state.next_player {
            0 => 0,
            _ => 1,
        };
        x
    }

    pub fn play_one_game(&mut self, mut searcher: search::SearchTree<State, usize>) -> Result<State, TicTacToeError> {
        let mut game = State::new();
        loop {
            if let search::GameResult::InProgress = game.status {
                let next = searcher.read_and_apply(self);
                game.play(next).unwrap();
                info!("Search chose {}@\n{}", next, game);
            } else {
                break;
            }
        }
        Ok(game)
    }

    fn game_to_feature(game: &State) -> gen::feature::Feature {
        let mut vec = Vec::with_capacity(19);
        for i in 0..2 {
            for v in 0..9 {
                vec.push(game.board[i][v] as u8);
            }
        }
        vec.push(game.next_player as u8);

        let mut repeated_field = ::protobuf::RepeatedField::<Vec<u8>>::new();
        repeated_field.push(vec);

        let mut bytes_list: gen::feature::BytesList = gen::feature::BytesList::new();
        bytes_list.set_value(repeated_field);

        let mut feature = gen::feature::Feature::new();
        feature.set_bytes_list(bytes_list);
        feature
    }
    fn move_to_feature(probs: Vec<f32>) -> gen::feature::Feature {

        let mut float_list = gen::feature::FloatList::new();
        float_list.set_value(probs);

        let mut feature = gen::feature::Feature::new();
        feature.set_float_list(float_list);
        feature
    }
    /*
        features {
            feature {
                game: 
                choice:
            }
        }

    */
    pub fn play_and_record_one_game<W: ::std::io::Write>(&mut self, 
        mut searcher: search::SearchTree<State, usize>, 
        dest: &mut W) -> Result<State, TicTacToeError> {
        let mut game = State::new();
        let mut writer = io::tf::RecordWriter::new(dest); 
        let mut f = File::create("examples.pb").unwrap();
        loop {
            let mut count = 0;
            if let search::GameResult::InProgress = game.status {
                let next = searcher.read_and_apply(self);
                game.play(next).unwrap();
                let state_feature = Self::game_to_feature(&game);
                let mut probs = Vec::<f32>::with_capacity(9);
                for i in 0..9 {
                    if next == i {
                        probs.push(1.0);
                    }
                    else {
                        probs.push(0.0);
                    }
                }
                let mut choice_feature = Self::move_to_feature(probs);
                let mut features_map = HashMap::new();
                features_map.insert("game".to_string(), state_feature);
                features_map.insert("choice".to_string(), choice_feature);

                let mut features = gen::feature::Features::new();
                features.set_feature(features_map);

                let mut example = gen::example::Example::new();
                example.set_features(features);
                // println!("{:?}", example);
                let proto_bytes = example.write_to_bytes().unwrap();
                writer.write_one_record(&proto_bytes);
                f.write(&proto_bytes);
                // unsafe {
                //     let bytes = ::std::mem::transmute::<[[bool; 9]; 2], [u8; 18]>(game.board);
                //     gen::example::
                //     dest.write(&bytes)?;
                // }
                // dest.write(&[game.next_player as u8])?;
                // let mut choice = [0u8; 9];
                // choice[next] = 1;
                // dest.write(&choice)?;
                break;
            } else {
                break;
            }
        }
        Ok(game)
    }

    pub fn train_next_example<R: ::std::io::Read>(&mut self, 
        options: TrainOptions, 
        source: &mut R) -> Result<(), TicTacToeError> {
        let mut state = tf::Tensor::new(&[1, 19]);
        
        source.read_exact(&mut state)?;
        let mut choice_buf = [0; 9];
        let mut choice = tf::Tensor::new(&[1, 9]);
        source.read_exact(&mut choice_buf)?;
        for i in 0..9 {
            choice[i] = choice_buf[i] as f32;
        }
        
        let op_x = self.graph.operation_by_name_required("x")?;
        let op_y_true = self.graph.operation_by_name_required("y_true")?;
        let op_train = self.graph.operation_by_name_required("train")?;
        let mut train_step = tf::StepWithGraph::new();
        train_step.add_input(&op_x, 0, &state);
        train_step.add_input(&op_y_true, 0, &choice);
        train_step.add_target(&op_train);
        self.session.run(&mut train_step)?;
        Ok(())
    }

    // synchronously play and train
    pub fn train(&mut self, n: usize) -> Result<(), TicTacToeError> {
        trace!("Attemping to load training ops...");
        let op_x = self.graph.operation_by_name_required("x")?;
        let op_y_true = self.graph.operation_by_name_required("y_true")?;
        let op_train = self.graph.operation_by_name_required("train")?;
        trace!("..succcess.");

        let mut options = search::SearchTreeOptions::defaults();
        options.readouts = 1500;
        options.tempering_point = 1;
        options.cpuct = 1.5;
        trace!("Beginning search & train with {:?}", options);

        for i in 0..n {
            trace!("game {}", i);
            let initial_search_state = State::new();
            let mut game = initial_search_state.clone();
            let mut search = search::SearchTree::init_with_options(initial_search_state, options.clone());
            loop {
                if let search::GameResult::InProgress = game.status {
                    let next = search.read_and_apply(self);

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
                    game.play(next).unwrap();
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}
impl search::GameExpert<State, usize> for DnnGameExpert {
    // TODO: The AGZ paper minibatches the request for expert policies
    // into batches of 8 hypotheses. There should be a way of batching these requests
    // This has to come after multi-threading the search, since threads block
    // while waiting for their batch to accumulate.
    fn hypotheses(&mut self, state: &State) -> search::Hypotheses<usize> {

        debug!("{}", state);
        let op_x = self.graph.operation_by_name_required("x").unwrap();
        let softmax = self.graph.operation_by_name_required("softmax").unwrap();

        let state_tensor = Self::state_tensor(state);
        let mut legal_actions : Vec<(usize, f32)> = vec![];
        {
            let mut inference_step = tf::StepWithGraph::new();
            inference_step.add_input(&op_x, 0, &state_tensor);
            let softmax_output_token = inference_step.request_output(&softmax, 0);

            self.session.run(&mut inference_step).expect("failed to run inference step");

            let inferences : tf::Tensor<f32> = inference_step.take_output(softmax_output_token).unwrap();
            debug!("raw inferences:");
            for i in 0..3 {
                let o = i * 3;
                debug!("{0: <w$.w$} | {1: <w$.w$} | {2: <w$.w$}", inferences[o], inferences[o+1], inferences[o+2], w=5);
            }
            debug!("");

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
            trace!("");
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
    fn dnn_strength_test() {
        _setup_test();
        let mut draw = 0;
        let n = 1;
        for _ in 0..n {
            let graph_filename = "src/tictactoe/simple_net.pb";
            let model_filename = "src/tictactoe/simple_model/";

            let mut ge = match super::DnnGameExpert::from_saved_model(graph_filename, model_filename) {
                Ok(ge) => {
                    ge
                },
                Err(e) => {
                    trace!("Could not open saved model at '{}'. Error: \n{:?}\nAttempting to initialize a new model with random weights.", model_filename, e);
                    let res = super::DnnGameExpert::init_with_random_weights(graph_filename, model_filename);
                    match res {
                        Ok(ge) => {
                            ge
                        },
                        Err(e) => panic!("Couldn't initialize a new model at '{}'. Error:\n{:?}", model_filename, e),
                    }
                }
            };
            
            let mut options = search::SearchTreeOptions::defaults();
            options.readouts = 1500;
            options.tempering_point = 2;
            options.cpuct = 1.5;

            let initial_search_state = State::new();
            let searcher = search::SearchTree::init_with_options(initial_search_state, options.clone());

            let game = ge.play_one_game(searcher).unwrap();
            if game.status == GameResult::TerminatedWithoutResult {
                draw += 1;
            }
        }

        println!("drew {} / {} games", draw, n);
        assert!(
            (draw as f32) / (n as f32) >= 1.0,
            "Most games should draw in a well-played game of Tic Tac Toe"
        );
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
