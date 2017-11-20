#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate twitter_stream;
extern crate egg_mode;
#[macro_use] extern crate error_chain;

pub mod error;
mod twitter;
mod kaizo;

pub use twitter::{Token, kaizo_stream};
pub use kaizo::Kaizo;
