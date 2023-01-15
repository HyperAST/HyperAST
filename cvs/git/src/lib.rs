#![feature(test)]
#![feature(drain_filter)]
pub mod allrefs;
pub mod git;
pub mod java;
pub mod maven;

pub mod java_processor;
pub mod maven_processor;
/// for now only tested on maven repositories with a pom in root.
pub mod preprocessed;
pub mod multi_preprocessed;
#[cfg(test)]
pub mod tests;

use git::BasicGitObject;
use git2::Oid;
use hyper_ast::{store::defaults::LabelIdentifier, utils::Bytes};
use maven::MD;
extern crate test;

// use hyper_ast_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
// use hyper_ast_gen_ts_xml::xml_tree_gen::{self, XmlTreeGen};

pub type SimpleStores = hyper_ast::store::SimpleStores;

// might also skip
pub(crate) const PROPAGATE_ERROR_ON_BAD_CST_NODE: bool = false;

pub(crate) const MAX_REFS: u32 = 10000; //4096;

pub struct Diffs();
pub struct Impacts();

#[derive(Clone)]
pub struct Commit {
    pub meta_data: MD,
    pub parents: Vec<git2::Oid>,
    processing_time: u128,
    memory_used: Bytes,
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

trait Accumulator: hyper_ast::tree_gen::Accumulator<Node = (LabelIdentifier, Self::Unlabeled)> {
    type Unlabeled;
    // fn push(&mut self, name: LabelIdentifier, full_node: Self::Node);
}

trait Processor<Acc: Accumulator> {
    fn process(&mut self) -> Acc::Unlabeled {
        loop {
            if let Some(current_dir) = self.stack().last_mut().expect("never empty").1.pop() {
                self.pre(current_dir)
            } else if let Some((oid, _, acc)) = self.stack().pop() {
                if let Some(x) = self.post(oid, acc) {
                    return x;
                }
            } else {
                panic!("never empty")
            }
        }
    }
    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, Acc)>;

    fn pre(&mut self, current_dir: BasicGitObject);
    fn post(&mut self, oid: Oid, acc: Acc) -> Option<Acc::Unlabeled>;
}
