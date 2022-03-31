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


use maven::MD;
extern crate test;

// use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
// use rusted_gumtree_gen_ts_xml::xml_tree_gen::{self, XmlTreeGen};

pub type SimpleStores = hyper_ast::store::SimpleStores;

// might also skip
pub(crate) const FAIL_ON_BAD_CST_NODE: bool = false;

pub(crate) const MAX_REFS: usize = 10000; //4096;

pub struct Diffs();
pub struct Impacts();

pub struct Commit {
    meta_data: MD,
    parents: Vec<git2::Oid>,
    pub ast_root: hyper_ast::store::nodes::DefaultNodeIdentifier,
}
