#![feature(conservative_impl_trait)]

extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;
extern crate twitter_stream;
extern crate egg_mode;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;

pub mod error;
mod twitter;
mod kaizo;

pub use twitter::Twitter;
pub use kaizo::Kaizo;
