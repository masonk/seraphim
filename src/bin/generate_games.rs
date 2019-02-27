#![cfg_attr(feature = "nightly", feature(alloc_system))]
#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate clap;
extern crate flexi_logger;
extern crate fs2;
extern crate seraphim;

#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

use seraphim::search;
extern crate ctrlc;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use fs2::FileExt;
use std::io;
use std::io::{Read, Seek, Write};
use std::result::Result;
use std::time;

use structopt::StructOpt;

static CONTROL_FILE: &'static str = "control";
static MODEL_DIR_PREFIX: &'static str = "models";

fn init_logger() {
    flexi_logger::Logger::with_env()
        // .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .duplicate_to_stderr(flexi_logger::Duplicate::Debug)
        .start()
        .unwrap();
}
// Generate new games of self-play from the champion of named model
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "interactive", about = "An interactive session of Tic Tac Toe.")]
struct Config {
    #[structopt(long, default_value = "100", help = "How many games in each tfrecord?")]
    games_per_file: i64,

    #[structopt(
        long,
        default_value = "50",
        help = "How many .tfrecord files to keep in output_dir."
    )]
    max_files: i64,

    #[structopt(short = "d", long)]
    debug: bool,

    #[structopt(
        long,
        help = "Write game data to this path instead of the default $SERAPHIM_DATA/gamedata/$SERAPHIM_MODEL_NAME"
    )]
    output_dir: Option<String>,

    #[structopt(flatten)]
    seraphim_config: seraphim::search::SeraphimConfig,

    #[structopt(flatten)]
    search_tree_options: seraphim::search::SearchTreeParamOverrides,
}

fn main() {
    init_logger();
    let opts = Config::from_args();
    let config = opts.seraphim_config;
    let mut overrides = opts.search_tree_options;
    overrides.dirichlet_alpha.get_or_insert(0.6);
    let search_tree_options = seraphim::search::SearchTreeOptions::from_overrides(overrides);

    let fq_model_dir = format!(
        "{}/{}/{}/{}/{}",
        config.seraphim_data, MODEL_DIR_PREFIX, &config.model_name, "champion", "saved_model"
    );
    let lock_path = format!(
        "{}/{}/{}/{}/{}",
        config.seraphim_data, MODEL_DIR_PREFIX, &config.model_name, "champion", "lock"
    );

    let output_dir = opts
        .output_dir
        .unwrap_or_else(|| format!("{}/gamedata/{}", config.seraphim_data, config.model_name));

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut count = 0;
    let mut draws = 0;
    'outer: while running.load(Ordering::SeqCst) {
        ::std::fs::create_dir_all(&output_dir).unwrap();

        let next_id = get_next_file_id(&output_dir).unwrap();
        println!("{}", next_id);
        let next_file_path = format!("{}/batch-{:07}", &output_dir, next_id);

        let lock = ::std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)
            .unwrap();

        let _ = lock.lock_exclusive();

        let file = ::std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(next_file_path.clone())
            .unwrap();

        let writer = ::std::io::BufWriter::new(file);

        let mut ge = match seraphim::tictactoe::DnnGameExpert::from_saved_model(&fq_model_dir) {
            Ok(ge) => ge,
            Err(e) => {
                panic!("Couldn't restore a model from '{}'. \nTry running 'src/tictactoe/train.py --init'\nError:\n{:?}", fq_model_dir,  e);
            }
        };
        let _ = lock.unlock();
        match do_some_games(
            &mut ge,
            opts.games_per_file,
            writer,
            &search_tree_options,
            running.clone(),
        ) {
            Ok((c, d)) => {
                count += c;
                draws += d;
            }
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
        )
        .unwrap();

        if next_id - opts.max_files >= 0 {
            stale_index
                .write_fmt(format_args!(
                    "{}/batch-{:07}.tfrecord\n",
                    output_dir,
                    next_id - opts.max_files
                ))
                .unwrap();
        }
        stale_index.unlock().unwrap();
        lock.unlock().unwrap();
    }

    println!("Drew {} / {} games", draws, count);
}

fn get_next_file_id(output_dir: &str) -> io::Result<i64> {
    let mut control = ::std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(&format!("{}/{}", output_dir, CONTROL_FILE))?;

    let mut buf = Vec::new();
    control.read_to_end(&mut buf)?;
    let val;

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
    options: &seraphim::search::SearchTreeOptions,
    running: Arc<AtomicBool>,
) -> Result<(i64, usize), io::Error> {
    let mut count = 0;
    let mut draws = 0;
    let now = time::Instant::now();

    while count < num {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        let initial_search_state = seraphim::tictactoe::State::new();
        let searcher = search::SearchTree::init_with_options(initial_search_state, options.clone());

        let res = ge.play_and_record_one_game(searcher, &mut writer);
        if let Err(err) = res {
            error!("Error while playing a game: {:?}", err);
            return Ok((count, draws));
        } else if let Ok(status) = res {
            match status {
                seraphim::search::GameStatus::Draw => {
                    draws += 1;
                }
                _ => {}
            }
        }

        count += 1;
        if count % 1000 == 0 {
            let _ = writer.flush();
        }
    }
    let _ = writer.flush();
    let elapsed = now.elapsed();
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    println!(
        "{} games in {:.2} sec ({:.2} games/sec)",
        count,
        sec,
        count as f64 / sec
    );
    println!("Drew {} / {}", draws, count);

    Ok((count, draws))
}
