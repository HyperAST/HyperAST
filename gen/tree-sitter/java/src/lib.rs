#![feature(min_specialization)]
#![feature(generic_associated_types)]
#![feature(let_chains)]
#![feature(backtrace)]
// #![feature(generic_const_exprs)]

pub mod compat;
pub mod legion_with_refs;

pub mod impact;
pub mod usage;

#[cfg(test)]
mod tests;

pub use hyper_ast::utils;