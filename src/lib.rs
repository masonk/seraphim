#![feature(conservative_impl_trait, universal_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![feature(nll)]
#![feature(test)]

pub mod error;
pub mod game;
pub mod interactive;
pub mod inference;
pub mod io;
pub mod search;

pub mod tictactoe;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate structopt;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate flexi_logger;
extern crate left_pad;
extern crate regex;
extern crate serde;
extern crate vec_map;


extern crate bincode;
extern crate petgraph;
extern crate protobuf;
extern crate rand;
extern crate tensorflow;

