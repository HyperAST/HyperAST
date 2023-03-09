#![warn(clippy::all, rust_2018_idioms)]
#![feature(result_option_inspect)]
#![feature(entry_insert)]

mod app;

pub use app::{
    types::{Lang, Languages},
    HyperApp,
};
