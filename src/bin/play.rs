#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate flexi_logger;
extern crate seraphim;
#[macro_use]
extern crate log;

use seraphim::search;
extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use std::result::Result;

fn init_logger() {
    flexi_logger::Logger::with_env()
        .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap();
}
// this script plays many games and records all of its move examples
fn main() {
    
    init_logger();

    let graph_filename = "src/tictactoe/simple_net.pb";
    let model_filename = "src/tictactoe/simple_model/";

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(graph_filename, model_filename) {
        Ok(ge) => {
            ge
        },
        Err(e) => {
            trace!("Could not open saved model at '{}'. Error: \n{:?}\nAttempting to initialize a new model with random weights.", model_filename, e);
            let res = seraphim::tictactoe::DnnGameExpert::init_with_random_weights(graph_filename, model_filename);
            match res {
                Ok(ge) => {
                    ge
                },
                Err(e) => panic!("Couldn't initialize a new model at '{}'. Error:\n{:?}", model_filename, e),
            }
        }
    };

    let mut count = 0;
    let batch = 1000;
    let game_file = ::std::fs::OpenOptions::new()
        .append(true)
        .create(true)   
        .open("src/tictactoe/gamedata/game1")
        .unwrap();
    let mut record = ::std::io::BufWriter::new(game_file);
    let mut options = search::SearchTreeOptions::defaults();
    options.readouts = 1500;
    options.tempering_point = 2;
    options.cpuct = 1.5;

    'outer: while running.load(Ordering::SeqCst) {
        let initial_search_state = seraphim::tictactoe::State::new();
        let searcher = search::SearchTree::init_with_options(initial_search_state, options.clone());
        ::std::fs::create_dir("src/tictactoe/gamedata");

        let res = ge.play_and_record_one_game(searcher, &mut record);
        if let Err(e) = res {
            error!("Error while playing:\n{:?}", e);
        }
        count += 1;
        if count % batch == 0 {
            record.flush();
            println!("{} games played, flushing", count);
        }
        break;
    }

    println!("saved {} games", count);
}

// fn main() {}
