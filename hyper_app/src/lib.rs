#![warn(clippy::all, rust_2018_idioms)]
#![feature(entry_insert)]
#![feature(extract_if)]
#![feature(exclusive_wrapper)]
#![feature(iter_intersperse)]

mod app;
pub use app::HyperApp;
pub use app::Languages;

mod command;
mod command_palette;
mod platform;
