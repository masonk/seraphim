#![cfg_attr(feature = "clippy", feature(plugin))]

pub mod error;
pub mod game;

pub mod inference;
pub mod interactive;
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
