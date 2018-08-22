// This is a minor script that I built to debug my examples files when the net wasn't training properly.
// I just needed to confirm that my training examples were surviving the disk roundtrip intact.
extern crate clap;
extern crate seraphim;
extern crate protobuf;
use protobuf::Message;

static DEFAULT_DIR: &'static str = "src/tictactoe/gamedata";

fn main() {
    let matches = clap::App::new("tfrecord_viewer")
        .about("Dump records inside of a .tfrecord file")
        .arg(
            clap::Arg::with_name("file")
                .help("A .tfrecord file")
                .takes_value(true)
                .required(true)
        )
        .get_matches();

    let path = matches.value_of("file").unwrap();
    let file = match  std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let mut records = seraphim::io::tf::RecordReader::new(file);

    println!("Looping through records. q quits");
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input);
        let trimmed = input.trim();
        if trimmed == "q" {
            break;
        }
        let record = records.read_one();
        match record {
            Ok(r) => {
                match r {
                    Some(r) => {
                        println!("{:?}", r);
                        let feat = parse_ttt_feature(r);
                        let featuremap = feat.get_feature();

                        println!("{:?}", featuremap);
                    }
                    _ => {
                        println!("EOF");
                        break;
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }
    }
}

fn parse_ttt_feature(buf: Vec<u8>) -> seraphim::tictactoe::gen::feature::Features {
    let mut features = seraphim::tictactoe::gen::feature::Features::new();
    let mut cursor = std::io::Cursor::new(buf);
    let mut is = protobuf::stream::CodedInputStream::new(&mut cursor);
    features.merge_from(&mut is).unwrap();
    features

}