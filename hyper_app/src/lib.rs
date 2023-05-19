#![warn(clippy::all, rust_2018_idioms)]
#![feature(result_option_inspect)]
#![feature(entry_insert)]
#![feature(drain_filter)]

mod app;

pub use app::{
    types::{Lang, Languages},
    HyperApp,
};