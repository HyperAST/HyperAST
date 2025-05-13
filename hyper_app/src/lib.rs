#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::HyperApp;
pub use app::Languages;

mod command;
mod command_palette;
mod platform;
