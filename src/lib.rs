#![feature(conservative_impl_trait, universal_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![feature(entry_and_modify)]
#![feature(nll)]

pub mod core;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate left_pad;
extern crate regex;
extern crate vec_map;
