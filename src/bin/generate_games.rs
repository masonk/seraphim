#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate clap;
extern crate flexi_logger;
extern crate fs2;
extern crate seraphim;

#[macro_use]
extern crate log;

use seraphim::search;
extern crate ctrlc;

use clap::{App, Arg, SubCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::process::exit;
use std::result::Result;

use fs2::FileExt;

static DEFAULT_GAMES_PER_FILE: i64 = 1000;
static DEFAULT_MAX_FILES: i64 = 50;
static DEFAULT_OUTPUT_DIR: &'static str = "src/tictactoe/gamedata";
static CONTROL_FILE: &'static str = "control";
static MODEL_DIR_PREFIX: &'static str = "src/tictactoe/models";

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
                            .arg(Arg::with_name("output_dir")
                                .help("Where does the generated data go")
                                .long("output_dir")
                                .takes_value(true))
                            .arg(Arg::with_name("games_per_file")
                                .help("How many games in each .tfrecord file")
                                .long("games_per_file")
                                .takes_value(true))
                            .arg(Arg::with_name("max_files")
                                .help("How many .tfrecord files to keep")
                                .long("max_files")
                                .takes_value(true))
                             .arg(Arg::with_name("exploration_coefficient")
                                .help("A coefficient that controls how tree search should balance the tradeoff between exploiting good moves and exploring undersampled moves. Try somewhere in the range of [0.1, 10]")
                                .long("exploration_coefficient")
                                .short("c")
                                .takes_value(true))
                            .get_matches();

    let model_dir = matches.value_of("model_dir").unwrap();
    let fq_model_dir = format!(
        "{}/{}/{}/{}",
        MODEL_DIR_PREFIX, model_dir, "champion", "saved_model"
    );
    let games_per_file = matches
        .value_of("games_per_file")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_GAMES_PER_FILE);
    let max_files = matches
        .value_of("max_files")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(DEFAULT_MAX_FILES);
    let output_dir = matches.value_of("output_dir").unwrap_or(DEFAULT_OUTPUT_DIR);
    let exploration_coefficient = matches
        .value_of("exploration_coefficient")
        .and_then(|c| c.parse::<f32>().ok())
        .unwrap_or(5.0);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut count = 0;

    'outer: while running.load(Ordering::SeqCst) {
        ::std::fs::create_dir_all(&output_dir).unwrap();

        let next_id = get_next_file_id(&output_dir).unwrap();
        println!("{}", next_id);
        let next_file_path = format!("{}/batch-{:07}", output_dir, next_id);
        let lock_path = format!(
            "{}/{}/{}/{}",
            MODEL_DIR_PREFIX, model_dir, "champion", "lock"
        );
        let lock = File::open(lock_path).unwrap();
        lock.lock_exclusive().unwrap();

        let file = ::std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(next_file_path.clone())
            .unwrap();

        let mut writer = ::std::io::BufWriter::new(file);

        let mut ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
            Ok(ge) => ge,
            Err(e) => {
                panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/init.py {}'\nError:\n{:?}", fq_model_dir, model_dir,  e);
            }
        };
        lock.unlock();
        match do_some_games(
            &mut ge,
            games_per_file,
            writer,
            exploration_coefficient,
            running.clone(),
        ) {
            Ok(c) => count += c,
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
        // changing files in gamedata is potentially racing with training processes that are reading
        // .tfrecord files.
        let mut stale_index = ::std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&format!("{}/stale_file_paths", output_dir))
            .unwrap();

        stale_index.lock_exclusive().unwrap();

        std::fs::rename(
            next_file_path.clone(),
            format!("{}.tfrecord", next_file_path),
        ).unwrap();

        if next_id - max_files >= 0 {
            stale_index
                .write_fmt(format_args!(
                    "{}/batch-{:07}.tfrecord\n",
                    output_dir,
                    next_id - max_files
                ))
                .unwrap();
        }
        stale_index.unlock().unwrap();
        lock.unlock().unwrap();
    }

    println!("saved {} games", count);
}

fn get_next_file_id(output_dir: &str) -> io::Result<i64> {
    let mut control = ::std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(&format!("{}/{}", output_dir, CONTROL_FILE))?;

    let mut buf = Vec::new();
    control.read_to_end(&mut buf)?;
    let mut val;

    if buf.len() == 0 {
        val = 0;
    } else {
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
    mut writer: W,
    exploration_coefficient: f32,
    running: Arc<AtomicBool>,
) -> Result<i64, io::Error> {
    let mut count = 0;
    let mut options = search::SearchTreeOptions::defaults();
    options.readouts = 1500;
    options.tempering_point = 1;
    options.cpuct = exploration_coefficient;

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
        }
    }
    Ok((count))
}
