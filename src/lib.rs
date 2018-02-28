#![feature(conservative_impl_trait, universal_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(nll)]
#![feature(test)]

pub mod go;
pub mod search;

extern crate gosgf;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate left_pad;
extern crate regex;
extern crate test;
extern crate vec_map;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate petgraph;
