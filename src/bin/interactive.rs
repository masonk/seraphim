extern crate clap;
extern crate ctrlc;
extern crate fs2;

use clap::{App, Arg};
use fs2::FileExt;
use std::env;
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
extern crate seraphim;
static MODEL_DIR_PREFIX: &'static str = "/models";

fn main() {
    let matches = App::new("Interactive TicTacToe")
            .about("Play a session against the AI..")
            .arg(Arg::with_name("model_dir")
                .help("The name of a directory under $SERAPHIM/models")
                .required(true))
            .arg(Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("In this mode, debug information is printed for every move. You can chose the action for each ply. Meant for debugging models and evaluating training"))
            .arg(Arg::with_name("exploration_coefficient")
                .help("A coefficient that controls how tree search should balance the tradeoff between exploiting good moves and exploring undersampled moves. Try somewhere in the range of [0.1, 10]")
                .long("exploration_coefficient")
                .short("c")
                .takes_value(true))
            .get_matches();
    let model_dir = matches.value_of("model_dir").unwrap();
    let exploration_coefficient = matches
        .value_of("exploration_coefficient")
        .and_then(|c| c.parse::<f32>().ok())
        .unwrap_or(5.0);
    start_game(
        matches.is_present("debug"),
        model_dir.to_string(),
        exploration_coefficient,
    );
}

fn start_game(debug: bool, model_dir: String, exploration_coefficient: f32) {
    let seraphim_dir = env::var("SERAPHIM").unwrap();

    let fq_model_dir = format!(
        "{}/{}/{}/{}/{}",
        seraphim_dir, MODEL_DIR_PREFIX, model_dir, "champion", "saved_model"
    );
    let lock_path = format!(
        "{}/{}/{}/{}/{}",
        seraphim_dir, MODEL_DIR_PREFIX, model_dir, "champion", "lock"
    );

    let lock = File::open(lock_path);
    if let Ok(ref lock) = lock {
        lock.lock_shared();
    }

    let ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
        Ok(ge) => ge,
        Err(e) => {
            panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/init.py {}'\nError:\n{:?}", fq_model_dir, model_dir,  e);
        }
    };
    if let Ok(ref lock) = lock {
        lock.unlock();
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut options = seraphim::search::SearchTreeOptions::defaults();
    options.cpuct = exploration_coefficient;
    options.tempering_point = 1;
    let mut session = seraphim::evaluation::interactive::InteractiveSession::new_with_options(
        ge,
        seraphim::tictactoe::State::new(),
        options,
    );
    if (debug) {
        session.start_debug(running)
    } else {
        session.start_game(running)
    }
}
