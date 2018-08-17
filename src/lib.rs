#![feature(conservative_impl_trait, universal_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(nll)]
#![feature(test)]

pub mod evaluation;
pub mod io;
pub mod search;
pub mod tictactoe;

#[macro_use]
extern crate lazy_static;
extern crate left_pad;
extern crate regex;
extern crate test;
extern crate vec_map;
extern crate flexi_logger;

#[macro_use]
extern crate log;
extern crate petgraph;
extern crate rand;
extern crate protobuf;
extern crate tensorflow;
extern crate clap;