#![warn(clippy::all, rust_2018_idioms)]
#![feature(result_option_inspect)]
#![feature(entry_insert)]
#![feature(extract_if)]
#![feature(exclusive_wrapper)]
#![feature(iter_intersperse)]

mod app;
pub use app::{types::Languages, HyperApp};
