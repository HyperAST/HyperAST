#![feature(min_specialization)]
pub mod compat;
pub mod full;
pub mod hashed;
pub mod java_tree_gen;
pub mod java_tree_gen2;
pub mod java_tree_gen_full_compress;
pub mod java_tree_gen_no_compress;
pub mod java_tree_gen_no_compress_arena;
pub mod nodes;
pub mod spaces;
pub mod tree_gen;
pub mod utils;
pub mod vec_map_store;

pub mod impact;
pub mod store;

#[cfg(test)]
mod tests;
