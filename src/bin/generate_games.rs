#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate flexi_logger;
extern crate seraphim;
extern crate clap;

#[macro_use]
extern crate log;

use seraphim::search;
extern crate ctrlc;

use clap::{Arg, App, SubCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::error::Error;
use std::fs::File;
use std::fs;
use std::io::{Read, Write, Seek};
use std::io;
use std::path::Path;
use std::process::exit;
use std::result::Result;

static MODEL_DIR_PREFIX: &'static str =  "src/tictactoe/models";
static GAME_DATA: &'static str = "src/tictactoe/gamedata";
static CONTROL_FILE: &'static str = "src/tictactoe/gamedata/control";
static GAMES_PER_FILE: i64 = 1;
static MAX_RECORD_FILES: i64 = 1;
fn init_logger() {
    flexi_logger::Logger::with_env()
        .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap();
}
// Generate new games of self-play from the champion of named model
fn main() {
    init_logger();
    let matches = App::new("TicTacToe")
                            .about("Plays games of tictactoe using the AlphaGo Zero algorithm and records the games as training examples.")
                            .arg(Arg::with_name("model_dir")
                                .help("The name of a directory under src/tictactoe/models")
                                .required(true))
                            .get_matches();

    let model_dir = matches.value_of("model_dir").unwrap();
    let fq_model_dir = format!("{}/{}/{}/{}", MODEL_DIR_PREFIX, model_dir, "champion", "saved_model");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    let mut count = 0;

    'outer: while running.load(Ordering::SeqCst) {
        ::std::fs::create_dir_all("src/tictactoe/gamedata").unwrap();

        let next_id = get_next_file_id().unwrap();
        println!("{}", next_id);
        let next_file_path = format!("src/tictactoe/gamedata/batch-{}", next_id);
        let file = ::std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(next_file_path.clone())
            .unwrap();

        let mut writer = ::std::io::BufWriter::new(file);
        let mut ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
            Ok(ge) => {
                ge
            },
            Err(e) => {
                panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/init.py {}'\nError:\n{:?}", fq_model_dir, model_dir,  e);
            }
        };
        match do_some_games(&mut ge, GAMES_PER_FILE, writer, running.clone()) {
            Ok(c) => count += c,
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
        std::fs::rename(next_file_path.clone(), format!("{}.tfrecord", next_file_path)).unwrap();
        if next_id - MAX_RECORD_FILES >= 0 {
            let stale = format!("src/tictactoe/gamedata/batch-{}.tfrecord", next_id - MAX_RECORD_FILES);
            ::std::fs::remove_file(stale).unwrap();
        }
    }

    println!("saved {} games", count);
}

fn get_next_file_id() -> io::Result<i64> {
    let mut control = ::std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true) 
            .open(CONTROL_FILE)?;

    let mut buf = Vec::new();
    control.read_to_end(&mut buf)?;
    let mut val;

    if buf.len() == 0 {
        val = 0;
    }
    else {
        let valstr = std::str::from_utf8(&buf).unwrap();
        val = valstr.parse::<i64>().unwrap();
    }
    control.set_len(0)?;
    control.seek(std::io::SeekFrom::Start(0))?;
    write!(control, "{}", val + 1)?;
    Ok(val)
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
