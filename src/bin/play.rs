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
use std::io;
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

    let graph_filename = "src/tictactoe/saved_models/graph_01.pb";
    let model_filename = "src/tictactoe/saved_models/";

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

    'outer: while running.load(Ordering::SeqCst) {
        let (space, file) = get_writer().unwrap();
        let mut writer = ::std::io::BufWriter::new(file);
        ::std::fs::create_dir("src/tictactoe/gamedata");
        println!("{}", space);

        match do_some_games(&mut ge, space, writer, running.clone()) {
            Ok(c) => count += c,
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
    }

    println!("saved {} games", count);
}

fn do_some_games<W: Write>(
    ge: &mut seraphim::tictactoe::DnnGameExpert, 
    num: i64, 
    mut writer:  W,
    running: Arc::<AtomicBool>) -> Result<i64, io::Error> {
        
    let mut count = 0;
    let mut options = search::SearchTreeOptions::defaults();
    options.readouts = 1500;
    options.tempering_point = 2;
    options.cpuct = 1.5;

    while count < num {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        let initial_search_state = seraphim::tictactoe::State::new();
        let searcher = search::SearchTree::init_with_options(initial_search_state, options.clone());

        let res = ge.play_and_record_one_game(searcher, &mut writer);
        if let Err(err) = res {
            error!("Error while playing a game: {:?}", err);
            return Ok((count));
        }
        
        count += 1;
        if count % 1000 == 0 {

            writer.flush();
            println!("{} games played, flushing", count);
        }
    }
    Ok((count))
}

fn get_writer() -> Result<(i64, ::std::fs::File), io::Error> {
    let (space, filename) = seraphim::io::get_current_data_filename("src/tictactoe/gamedata", "ttt", 500_000)?;

    let file = ::std::fs::OpenOptions::new()
        .append(true)
        .create(true)   
        .open(filename)
        .unwrap();

    Ok((space, file))
}


