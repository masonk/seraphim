#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate flexi_logger;
extern crate seraphim;
#[macro_use]
extern crate log;

use seraphim::search;
use seraphim::tictactoe as ttt;

extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, Seek};
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
// this script reads record move examples and trains the net using them
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
        .read(true)
        .open("src/tictactoe/gamedata/game1")
        .unwrap();
    let mut record = ::std::io::BufReader::new(game_file);
    let mut options = search::SearchTreeOptions::defaults();
    options.readouts = 1500;
    options.tempering_point = 2;
    options.cpuct = 1.5;

    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        let mut train_res = ge.train_next_example(ttt::TrainOptions::new(), &mut record);
        {
            // If the error is UnexpectedEof, seek the train file to the start and try again.
            let mut new_train_res = None;
            if is_eof_err(&train_res) {
                trace!("Got eof error, seeking to start");
                record.seek(::std::io::SeekFrom::Start(0));
                new_train_res = Some(ge.train_next_example(ttt::TrainOptions::new(), &mut record));
            }
            if let Some(new_train_res) = new_train_res {
                if is_eof_err(&new_train_res) {
                    error!("UnexpectedEof, and seeking didn't recover. Empty train file?");
                    break;
                }
                train_res = new_train_res;
            }
        }

        if let Err(e) = train_res {
            error!("Unhandled error:\n {:?}", e);
            break;
        }

        count += 1;
        if count % batch == 0 {
            ge.save_model(model_filename).unwrap();
            println!("{} games trained: checkpointing", count);
        }
    }

    ge.save_model(model_filename);
    println!("saved {} games", count);
}

fn is_eof_err(res: &Result<(), ttt::TicTacToeError>) -> bool {
    if let Err(ttt::TicTacToeError { ref root_error, .. }) = res {
        if let ttt::RootErrorCause::Io(ref e) = root_error {
            return e.kind() == ::std::io::ErrorKind::UnexpectedEof;
        }
    }
    false
}