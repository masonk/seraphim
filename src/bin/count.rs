extern crate clap;
extern crate protobuf;
extern crate seraphim;
#[macro_use]
extern crate structopt;
extern crate fallible_iterator;

use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

use fallible_iterator::{FallibleIterator, IntoFallibleIterator};
use protobuf::Message;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "interactive", about = "An interactive session of Tic Tac Toe.")]
struct Config {
    #[structopt(flatten)]
    seraphim_config: seraphim::search::SeraphimConfig,
}

type Probs = [[f32; 9]; 9];

fn main() -> io::Result<()> {
    let opts = Config::from_args();
    let seraphim_config = opts.seraphim_config;

    let gamedata_prefix = format!(
        "{}/gamedata/{}",
        &seraphim_config.seraphim_data, &seraphim_config.model_name
    );

    println!("{}", gamedata_prefix);
    let mut ps: Probs = [[0f32; 9]; 9];
    let mut game_count = 0;
    for entry in fs::read_dir(gamedata_prefix)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "tfrecord" {
                let file = std::fs::File::open(&path)?;
                println!("{:?}", entry.path());
                let mut records = seraphim::io::tf::RecordReader::new(file);
                let mut iter = records.into_fallible_iter();
                let mut ply: usize = 0;

                while let Some(record) = iter.next()? {
                    let example = parse_ttt_example(record);
                    let features = example.get_features().get_feature();
                    let choice: &seraphim::tictactoe::gen::feature::Feature =
                        features.get("choice").unwrap();
                    if let Some(
                        seraphim::tictactoe::gen::feature::Feature_oneof_kind::float_list(
                            ref probs,
                        ),
                    ) = choice.kind
                    {
                        for (i, p) in probs.get_value().iter().enumerate() {
                            ps[ply][i] += p;
                        }
                    }
                    if ply == 8 {
                        game_count += 1;
                    }
                    ply = (ply + 1) % 9;
                }
            }
        }
    }
    println!("Parsed {} games", game_count);
    for (i, ply) in ps.iter().enumerate() {
        println!("--- ply: {} ----", i);

        let normalized: &Vec<f32> = &ply[..]
            .iter()
            .map(|p| p / game_count as f32)
            .collect::<Vec<f32>>();
        for i in 0..3 {
            println!(
                "{} {} {}",
                normalized[i * 3],
                normalized[i * 3 + 1],
                normalized[i * 3 + 2]
            )
        }
    }

    Ok(())
}

fn parse_ttt_example(buf: Vec<u8>) -> seraphim::tictactoe::gen::example::Example {
    let mut example = seraphim::tictactoe::gen::example::Example::new();
    let mut cursor = std::io::Cursor::new(buf);
    let mut is = protobuf::stream::CodedInputStream::new(&mut cursor);
    example.merge_from(&mut is).unwrap();
    example
}
