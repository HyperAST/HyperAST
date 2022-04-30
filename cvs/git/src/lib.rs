#![feature(test)]
#![feature(drain_filter)]
pub mod allrefs;
pub mod git;
pub mod java;
pub mod maven;
/// for now only tested on maven repositories with a pom in root.
pub mod preprocessed;
#[cfg(test)]
pub mod tests;

use hyper_ast::utils::{MemoryUsage, Bytes};
use maven::MD;
extern crate test;

// use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
// use rusted_gumtree_gen_ts_xml::xml_tree_gen::{self, XmlTreeGen};

pub type SimpleStores = hyper_ast::store::SimpleStores;

// might also skip
pub(crate) const PROPAGATE_ERROR_ON_BAD_CST_NODE: bool = false;

pub(crate) const MAX_REFS: u32 = 10000; //4096;

pub struct Diffs();
pub struct Impacts();

#[derive(Clone)]
pub struct Commit {
    meta_data: MD,
    parents: Vec<git2::Oid>,
    processing_time:u128,
    memory_used:Bytes,
    pub ast_root: hyper_ast::store::nodes::DefaultNodeIdentifier,
}

impl Commit {
    pub fn processing_time(&self) -> u128 {
        self.processing_time
    }
    pub fn memory_used(&self) -> Bytes {
        self.memory_used
    }
}