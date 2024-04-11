// #![feature(min_specialization)]
#![feature(exact_size_is_empty)]
#![feature(slice_index_methods)]

pub mod compat;
#[cfg(feature = "legion")]
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

pub trait PrimInt: num::PrimInt + num::traits::NumAssign + std::fmt::Debug {}
impl<T> PrimInt for T where T: num::PrimInt + num::traits::NumAssign + std::fmt::Debug {}

#[cfg(test)]
mod tests;
