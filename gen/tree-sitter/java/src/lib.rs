#![feature(min_specialization)]
#![feature(generic_associated_types)]
#![feature(let_chains)]
#![feature(backtrace)]
#![feature(generic_const_exprs)]

pub mod compat;
pub mod full;
pub mod hashed;
pub mod java_tree_gen;
pub mod java_tree_gen2;
pub mod java_tree_gen_full_compress;
pub mod java_tree_gen_full_compress_ecs;
pub mod java_tree_gen_full_compress_legion;
pub mod java_tree_gen_full_compress_legion_ref;
pub mod java_tree_gen_full_compress_ref_md;
pub mod java_tree_gen_no_compress;
pub mod java_tree_gen_no_compress_arena;
pub mod nodes;
pub mod spaces;
pub mod tree_gen;
pub mod utils;
pub mod vec_map_store;

pub mod impact;
pub mod store;
pub mod filter;
pub mod usage;

#[cfg(test)]
mod tests;
