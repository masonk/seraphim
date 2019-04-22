// use super::gen;
// use super::state::TicTacToeError;
// use super::State;
// use flexi_logger;
// use io;
// use search;

// use std::collections::HashMap;
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// use ::game::GameStatus;

// use protobuf::Message;
// use tensorflow as tf;

// impl ::Game::GameAction for usize {}

// #[derive(Debug)]
// pub struct DnnGameExpert {
//     root_state: State,
//     graph: tf::Graph,
//     session: tf::Session,
// }
// impl DnnGameExpert {
//     pub fn from_saved_model(model_filename: &str) -> Result<Self, TicTacToeError> {
//         trace!("Attemping to load saved model from '{}'", model_filename);

//         let mut graph = tf::Graph::new();

//         let tags: [&str; 0] = [];
//         let session = tf::Session::from_saved_model(
//             &tf::SessionOptions::new(),
//             &["serve"],
//             &mut graph,
//             model_filename,
//         )?;
//         trace!("...success.");

//         Ok(DnnGameExpert {
//             root_state: super::State::new(),
//             graph: graph,
//             session: session,
//         })
//     }

//     pub fn play_and_record_one_game<W: ::std::io::Write>(
//         &mut self,
//         mut searcher: search::SearchTree<State, usize>,
//         dest: &mut W,
//     ) -> Result<GameStatus, TicTacToeError> {
//         let mut writer = io::tf::RecordWriter::new(dest);
//         loop {
//             if let GameStatus::InProgress = searcher.current_state_ref().status {
//                 let results: search::SearchResultsInfo<usize> = searcher.read(self);

//                 let mut posteriors = vec![0.0; 9];
//                 for (i, p) in results.results.iter().enumerate() {
//                     posteriors[i] = *p;
//                 }

//                 let state_feature = Self::game_to_feature(searcher.current_state_ref());

//                 let mut choice_feature = Self::move_to_feature(posteriors);
//                 let mut features_map = HashMap::new();

//                 // println!("From This Board Position:\n{}", game);
//                 // println!("Chose This Action:\n{}", next);
//                 features_map.insert("game".to_string(), state_feature);
//                 features_map.insert("choice".to_string(), choice_feature);

//                 let mut features = gen::feature::Features::new();
//                 features.set_feature(features_map);

//                 let mut example = gen::example::Example::new();
//                 example.set_features(features);
//                 // println!("{:?}", example);
//                 let proto_bytes = example.write_to_bytes().unwrap();
//                 writer.write_one_record(&proto_bytes);
//                 searcher.apply_search_results(&results);
//             } else {
//                 break;
//             }
//         }
//         Ok(searcher.current_state_ref().status.clone())
//     }

//     fn state_tensor(state: &State) -> tf::Tensor<u8> {
//         let mut x = tf::Tensor::new(&[1, 19]);
//         for i in 0..2 {
//             for j in 0..9 {
//                 x[i * 9 + j] = state.board[i][j] as u8;
//             }
//         }

//         x[18] = match state.next_player {
//             0 => 0,
//             _ => 1,
//         };
//         x
//     }

//     fn game_to_feature(game: &State) -> gen::feature::Feature {
//         let mut vec = Vec::with_capacity(19);
//         for i in 0..2 {
//             for v in 0..9 {
//                 vec.push(game.board[i][v] as u8);
//             }
//         }
//         vec.push(game.next_player as u8);

//         let mut repeated_field = ::protobuf::RepeatedField::<Vec<u8>>::new();
//         repeated_field.push(vec);

//         let mut bytes_list: gen::feature::BytesList = gen::feature::BytesList::new();
//         bytes_list.set_value(repeated_field);

//         let mut feature = gen::feature::Feature::new();
//         feature.set_bytes_list(bytes_list);
//         feature
//     }

//     fn move_to_feature(probs: Vec<f32>) -> gen::feature::Feature {
//         let mut float_list = gen::feature::FloatList::new();
//         float_list.set_value(probs);

//         let mut feature = gen::feature::Feature::new();
//         feature.set_float_list(float_list);
//         feature
//     }

//     // Training is now done in Python. Saving this in case I ever want to try training Rust again one day.
//     // But there is much more framework support in Python as of 08/2018.
//     // // synchronously play and train
//     // pub fn train(&mut self, n: usize) -> Result<(), TicTacToeError> {
//     //     trace!("Attemping to load training ops...");
//     //     let op_x = self.graph.operation_by_name_required("example")?;
//     //     let op_y_true = self.graph.operation_by_name_required("label")?;
//     //     let op_train = self.graph.operation_by_name_required("train")?;
//     //     trace!("..succcess.");

//     //     let mut options = search::SearchTreeOptions::defaults();
//     //     options.readouts = 1500;
//     //     options.tempering_point = 1;
//     //     options.cpuct = 1.5;
//     //     trace!("Beginning search & train with {:?}", options);

//     //     for i in 0..n {
//     //         trace!("game {}", i);
//     //         let initial_search_state = State::new();
//     //         let mut game = initial_search_state.clone();
//     //         let mut search =
//     //             search::SearchTree::init_with_options(initial_search_state, options.clone());
//     //         loop {
//     //             if let GameStatus::InProgress = game.status {
//     //                 let next = search.read_and_apply(self);

//     //                 // x is game state
//     //                 // next goes into y_true for training
//     //                 let x = super::State_tensor(&game);
//     //                 let mut y_true = tf::Tensor::new(&[1, 9]);
//     //                 y_true[next] = 1.0f32;

//     //                 let mut train_step = tf::SessionRunArgs::new();
//     //                 train_step.add_feed(&op_x, 0, &x);
//     //                 train_step.add_feed(&op_y_true, 0, &y_true);
//     //                 train_step.add_target(&op_train);
//     //                 self.session.run(&mut train_step)?;
//     //                 game.play(next).unwrap();
//     //             } else {
//     //                 break;
//     //             }
//     //         }
//     //     }

//     //     Ok(())
//     // }
//     // Loading from a graph (as opposed to a SavedModel) is necessary for training in Rust
//     //     fn load_graph(filename: &str) -> Result<tf::Graph, TicTacToeError> {
//     //     if !Path::new(filename).exists() {
//     //         return Err(TicTacToeError::from(
//     //             tf::Status::new_set(
//     //                 tf::Code::NotFound,
//     //                 &format!(
//     //                     "source bin/activate && python src/tictactoe/simple_net.py \
//     //                      to generate {} and try again.",
//     //                     filename
//     //                 ),
//     //             ).unwrap(),
//     //         ));
//     //     }

//     //     let mut proto = Vec::new();
//     //     File::open(filename)?.read_to_end(&mut proto)?;
//     //     let mut graph = tf::Graph::new();

//     //     graph.import_graph_def(&proto, &tf::ImportGraphDefOptions::new())?;
//     //     Ok(graph)
//     // }

//     // pub fn play_one_game(
//     //     &mut self,
//     //     mut searcher: search::SearchTree<State, usize>,
//     // ) -> Result<State, TicTacToeError> {
//     //     let mut game = State::new();
//     //     loop {
//     //         if let GameStatus::InProgress = game.status {
//     //             let debug = searcher.read_and_apply_debug(self);
//     //             game.play(debug.results.selection).unwrap();
//     //             info!("Search chose {:?}@\n{}", debug, game);
//     //         } else {
//     //             break;
//     //         }
//     //     }
//     //     Ok(game)
//     // }
// }


// #[cfg(test)]
// mod expert {
//     use super::{_setup_test, search, GameStatus, State};

//     #[test]
//     fn dnn_strength_test() {
//         _setup_test();
//         let mut draw = 0;
//         let n = 1;
//         for _ in 0..n {
//             let model_filename = "src/tictactoe/simple_model/test_model";

//             let mut ge = match super::DnnGameExpert::from_saved_model(model_filename) {
//                 Ok(ge) => ge,
//                 Err(e) => {
//                     trace!("Could not open saved model at '{}'. Error: \n{:?}\nAttempting to initialize a new model with random weights.", model_filename, e);
//                     let res = super::DnnGameExpert::init_with_random_weights(
//                         graph_filename,
//                         model_filename,
//                     );
//                     match res {
//                         Ok(ge) => ge,
//                         Err(e) => panic!(
//                             "Couldn't initialize a new model at '{}'. Error:\n{:?}",
//                             model_filename, e
//                         ),
//                     }
//                 }
//             };

//             let mut options = search::SearchTreeOptions::defaults();
//             options.readouts = 1500;
//             options.tempering_point = 2;
//             options.cpuct = 1.5;

//             let initial_search_state = State::new();
//             let searcher =
//                 search::SearchTree::init_with_options(initial_search_state, options.clone());

//             let game = ge.play_one_game(searcher).unwrap();
//             if game.status == GameStatus::NullResult {
//                 draw += 1;
//             }
//         }

//         println!("drew {} / {} games", draw, n);
//         assert!(
//             (draw as f32) / (n as f32) >= 1.0,
//             "Most games should draw in a well-played game of Tic Tac Toe"
//         );
//     }
// }

// #[cfg(test)]
// mod basic {
//     use super::*;

//     #[test]
//     fn parse_empty_board() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             _ _ _
//             _ _ _
//             _ _ _",
//         )
//         .expect("Couldn't parse an empty board");
//     }

//     #[test]
//     fn parse_a_board() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             o x o
//             _ x _
//             _ o _",
//         )
//         .expect("Couldn't parse");
//     }

//     #[test]
//     fn o_wins_row() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             o x x
//             _ x x
//             o o o",
//         )
//         .expect("Couldn't parse");

//         trace!("{}", state);

//         assert_eq!(state.status, GameStatus::LastPlayerWon);
//         assert_eq!(state.next_player, 0);
//     }

//     #[test]
//     fn x_wins_col() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             o x o
//             _ x _
//             _ x _",
//         )
//         .expect("Couldn't parse");

//         trace!("{}", state);
//     }

//     #[test]
//     fn x_wins_nw_diag() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             x _ x
//             o x o
//             _ o x",
//         )
//         .expect("Couldn't parse");

//         trace!("{}", state);

//         assert_eq!(state.status, GameStatus::LastPlayerWon);
//         assert_eq!(state.next_player, 1);
//     }

//     #[test]
//     fn o_wins_ne_diag() {
//         _setup_test();
//         let state = State::from_str(
//             "\
//             _ x o
//             _ o _
//             o x x",
//         )
//         .expect("Couldn't parse");

//         trace!("{}", state);

//         assert_eq!(state.status, GameStatus::LastPlayerWon);
//         assert_eq!(state.next_player, 0);
//     }
// }
