#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate itertools;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod go::sgf;
pub mod go;
pub use self::gosgf::*;
pub mod go::parse_sgf;
