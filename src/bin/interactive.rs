// extern crate ctrlc;
// extern crate fs2;
// #[macro_use]
// extern crate structopt;
// extern crate rand;
// extern crate seraphim;

// use structopt::StructOpt;

// use fs2::FileExt;
// use std::fs::File;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;

// static MODEL_DIR_PREFIX: &'static str = "models";

// #[derive(Debug, StructOpt)]
// #[structopt(name = "interactive", about = "An interactive session of Tic Tac Toe.")]
// struct Config {
//     #[structopt(short = "d", long)]
//     debug: bool,

//     #[structopt(flatten)]
//     seraphim_config: seraphim::search::SeraphimConfig,

//     #[structopt(flatten)]
//     search_tree_options: seraphim::search::SearchTreeParamOverrides,
// }

// fn main() {
//     let config = Config::from_args();
//     start_game(config);
// }

// fn start_game(config: Config) {
//     let seraphim = config.seraphim_config;
//     let mut overrides = config.search_tree_options;
//     overrides.dirichlet_alpha.get_or_insert(0.6);
//     overrides.cpuct.get_or_insert(1.5);
//     let search_tree_options = seraphim::search::SearchTreeOptions::from_overrides(overrides);

//     let fq_model_dir = format!(
//         "{}/{}/{}/{}/{}",
//         seraphim.seraphim_data, MODEL_DIR_PREFIX, seraphim.model_name, "champion", "saved_model"
//     );
//     let lock_path = format!(
//         "{}/{}/{}/{}/{}",
//         seraphim.seraphim_data, MODEL_DIR_PREFIX, seraphim.model_name, "champion", "lock"
//     );

//     let lock = File::open(lock_path);
//     if let Ok(ref lock) = lock {
//         let _ = lock.lock_shared();
//     }

//     let ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
//         Ok(ge) => ge,
//         Err(e) => {
//             panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/train.py --init'\nError:\n{:?}", fq_model_dir,  e);
//         }
//     };
//     if let Ok(ref lock) = lock {
//         let _ = lock.unlock();
//     }

//     let running = Arc::new(AtomicBool::new(true));
//     let r = running.clone();

//     ctrlc::set_handler(move || {
//         r.store(false, Ordering::SeqCst);
//     })
//     .expect("Error setting Ctrl-C handler");

//     let mut session = seraphim::evaluation::interactive::InteractiveSession::new_with_options(
//         ge,
//         seraphim::tictactoe::State::new(),
//         search_tree_options.clone(),
//     );
//     if config.debug {
//         session.start_debug(running)
//     } else {
//         session.start_game(running)
//     }
// }
