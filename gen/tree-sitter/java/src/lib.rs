#![feature(min_specialization)]
pub mod tree_gen;
pub mod java_tree_gen;
pub mod java_tree_gen2;
pub mod java_tree_gen_full_compress;
pub mod java_tree_gen_no_compress;
pub mod java_tree_gen_no_compress_arena;
pub mod spaces;
pub mod vec_map_store;
pub mod nodes;
pub mod hashed;
pub mod full;
pub mod utils;
pub mod compat;

pub mod impact;
pub mod store;

#[cfg(test)]
mod tests;