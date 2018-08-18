extern crate clap;
extern crate ctrlc;
extern crate fs2;

use clap::{App, Arg};
use fs2::FileExt;
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

extern crate seraphim;
static MODEL_DIR_PREFIX: &'static str = "src/tictactoe/models";

fn main() {
    let matches = App::new("Interactive TicTacToe")
            .about("Start an interactive session where the expert plays one side and the user plays the other half.")
            .arg(Arg::with_name("model_dir")
                .help("The name of a directory under src/tictactoe/models")
                .required(true))
            .arg(Arg::with_name("debug")
                .long("debug")
                .help("In this mode, far more information is given about the inner workings of the expert. Meant for debugging models and evaluating training"))
            .get_matches();
    let model_dir = matches.value_of("model_dir").unwrap();

    start_game(matches.is_present("debug"), model_dir.to_string());
}

fn start_game(debug: bool, model_dir: String) {
    let fq_model_dir = format!(
        "{}/{}/{}/{}",
        MODEL_DIR_PREFIX, model_dir, "champion", "saved_model"
    );
    let lock_path = format!(
        "{}/{}/{}/{}",
        MODEL_DIR_PREFIX, model_dir, "champion", "lock"
    );
    let lock = File::open(lock_path).unwrap();
    lock.lock_shared().unwrap();
    let ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
        Ok(ge) => ge,
        Err(e) => {
            panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/init.py {}'\nError:\n{:?}", fq_model_dir, model_dir,  e);
        }
    };
    lock.unlock().unwrap();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut session = seraphim::evaluation::interactive::InteractiveSession::new(
        ge,
        seraphim::tictactoe::State::new(),
    );
    session.start_game(running)
}
