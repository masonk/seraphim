// extern crate ctrlc;
// extern crate fs2;
// extern crate rand;
// extern crate structopt;

// use structopt::StructOpt;

// use seraphim::search;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// static MODEL_DIR_PREFIX: &'static str = "models";
// extern crate flexi_logger;

// #[derive(Debug, StructOpt)]
// #[structopt(name = "interactive", about = "An interactive session of Tic Tac Toe.")]
// struct Config {
//     #[structopt()]
//     count: usize,

//     #[structopt(long)]
//     out: Option<String>,

//     #[structopt(flatten)]
//     seraphim_config: seraphim::search::SeraphimConfig,

//     #[structopt(flatten)]
//     search_tree_options: seraphim::search::SearchTreeParamOverrides,
// }

// #[derive(Clone, Debug)]
// struct Ply {
//     played: [u8; 9],
//     avg_samples: [(usize, f64); 9],
// }
// impl std::default::Default for Ply {
//     fn default() -> Self {
//         Ply {
//             played: [0u8; 9],
//             avg_samples: [(0, 0.0f64); 9],
//         }
//     }
// }
// #[derive(Clone, Debug)]
// struct Counts {
//     plys: [Ply; 9],
// }
// impl std::default::Default for Counts {
//     fn default() -> Self {
//         let plys: [Ply; 9] = Default::default();
//         Counts { plys }
//     }
// }

// fn init_logger() {
//     flexi_logger::Logger::with_env()
//         // .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
//         .duplicate_to_stderr(flexi_logger::Duplicate::Debug)
//         .start()
//         .unwrap();
// }

// fn main() {
//     init_logger();
//     let config = Config::from_args();
//     let out = config.out.unwrap_or(format!(
//         "{}/histograms/{}/hist.csv",
//         &config.seraphim_config.seraphim_data, &config.seraphim_config.model_name
//     ));
//     let fq_model_dir = format!(
//         "{}/{}/{}/{}/{}",
//         config.seraphim_config.seraphim_data,
//         MODEL_DIR_PREFIX,
//         config.seraphim_config.model_name,
//         "champion",
//         "saved_model"
//     );

//     let mut overrides = config.search_tree_options;
//     overrides.dirichlet_alpha.get_or_insert(0.6);
//     overrides.cpuct.get_or_insert(1.5);

//     let search_tree_options = seraphim::search::SearchTreeOptions::from_overrides(overrides);

//     let running = Arc::new(AtomicBool::new(true));
//     let r = running.clone();

//     ctrlc::set_handler(move || {
//         r.store(false, Ordering::SeqCst);
//     })
//     .expect("Error setting Ctrl-C handler");

//     let mut ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
//         Ok(ge) => ge,
//         Err(e) => {
//             panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/train.py --init'\nError:\n{:?}", fq_model_dir,  e);
//         }
//     };
//     let mut counts = Counts::default();
//     let mut i = 0;
//     while running.load(Ordering::SeqCst) {
//         if i >= config.count {
//             break;
//         }
//         do_one_game(search_tree_options.clone(), &mut ge, &mut counts);
//         i += 1;
//     }

//     println!("played on each ply");
//     for ply in &counts.plys {
//         println!(
//             "{}",
//             ply.played
//                 .iter()
//                 .map(|i| format!("{}", i))
//                 .collect::<Vec<String>>()
//                 .join(", ")
//         );
//     }
//     println!("sampled on each ply");
//     for ply in &counts.plys {
//         println!(
//             "{}",
//             ply.avg_samples
//                 .iter()
//                 .map(|s| format!("{:.0}", s.1))
//                 .collect::<Vec<String>>()
//                 .join(", ")
//         );
//     }
// }

// fn do_one_game(
//     search_tree_options: search::SearchTreeOptions,
//     ge: &mut seraphim::tictactoe::DnnGameExpert,
//     counts: &mut Counts,
// ) {
//     let mut game_state = seraphim::tictactoe::State::new();
//     let mut searcher =
//         search::SearchTree::init_with_options(game_state.clone(), search_tree_options);

//     let mut i = 0;
//     loop {
//         if searcher.current_state_ref().status != search::GameStatus::InProgress {
//             break;
//         }
//         let debug = searcher.read_debug(ge);
//         searcher.apply_search_results(&debug.results);

//         // println!("{:?}", debug.results);
//         let selection = debug.results.selection;
//         let ply = &mut counts.plys[i];
//         ply.played[selection] += 1;
//         for can in debug.candidates {
//             let avg = &mut ply.avg_samples[can.action];
//             avg.0 += 1;
//             avg.1 = avg.1 + (can.total_visits as f64 - avg.1) / avg.0 as f64;

//             // println!(
//             //     "{:?} {:?} {:?} {:?}",
//             //     can.action, can.prior, can.visits_in_last_read, can.total_visits
//             // );
//         }
//         i += 1;
//     }
// }
