#![feature(conservative_impl_trait, universal_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(nll)]
#![feature(test)]

pub mod search;
pub mod tictactoe;
pub mod io;

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
