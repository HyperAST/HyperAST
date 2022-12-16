#![feature(min_specialization)]
#![feature(exact_size_is_empty)]
#![feature(slice_index_methods)]

pub mod compat;
pub mod cyclomatic;
pub mod filter;
pub mod full;
pub mod hashed;
pub mod impact;
pub mod nodes;
pub mod position;
pub mod store;
pub mod tree_gen;
pub mod types;
pub mod usage;
pub mod utils;

#[cfg(test)]
mod tests;
