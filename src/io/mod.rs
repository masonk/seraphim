use std::io;
use std::fs;
use regex::Regex;
pub mod tf;

pub fn get_current_data_filename(dir: &str, prefix: &str, max: i64) -> Result<(i64, String), io::Error> {
    let paths = fs::read_dir(dir)?;
    // todo: index these by id and rotate to a new file every "max" training examples

    let mut pre = String::new();
    if prefix.len() > 0 {
        pre.push_str(prefix);
        pre.push('_');
    }
    Ok((max, format!("src/tictactoe/gamedata/{}0.tfrecord", pre)))
    // let re = Regex::new(&format!("{}(\\d+)_(\\d+).tfrecord", pre)).unwrap();

    // for path in paths {
    //     let path = path.unwrap().path();
    //     if let Some(string) = path.to_str() {
    //         let cap = re.captures(string);
    //         if let Some(cap) = cap {
    //             let id = cap[1].parse::<u64>().unwrap();
    //             let num = cap[2].parse::<i64>().unwrap();

    //             if num < max {
    //                 return Ok((max - num, string.to_string()));
    //             }
    //         }
    //     }
    // }
}


