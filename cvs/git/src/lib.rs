#![feature(test)]
#![feature(extract_if)]
pub mod allrefs;
pub mod cpp;
pub mod git;
pub mod java;
pub mod make;
pub mod maven;

#[cfg(feature = "cpp")]
pub mod cpp_processor;
#[cfg(feature = "java")]
pub mod java_processor;
#[cfg(feature = "make")]
pub mod make_processor;
#[cfg(feature = "maven")]
pub mod maven_processor;
pub mod multi_preprocessed;
pub mod no_space;
/// for now only tested on maven repositories with a pom in root.
pub mod preprocessed;
pub mod processing;
mod utils;

#[cfg(test)]
pub mod tests;

use git::BasicGitObject;
use git2::Oid;
use hyper_ast::{store::defaults::LabelIdentifier, utils::Bytes};

mod type_store;

pub use type_store::MultiType;
pub use type_store::TStore;

pub type SimpleStores = hyper_ast::store::SimpleStores<TStore>;

// might also skip
pub(crate) const PROPAGATE_ERROR_ON_BAD_CST_NODE: bool = false;

pub(crate) const MAX_REFS: u32 = 10000; //4096;

pub(crate) type DefaultMetrics =
    hyper_ast::tree_gen::SubTreeMetrics<hyper_ast::hashed::SyntaxNodeHashs<u32>>;

pub struct Diffs();
pub struct Impacts();

#[derive(Clone)]
pub struct Commit {
    pub parents: Vec<git2::Oid>,
    processing_time: u128,
    memory_used: Bytes,
    pub ast_root: hyper_ast::store::nodes::DefaultNodeIdentifier,
    pub tree_oid: git2::Oid,
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

#[derive(Debug)]
pub(crate) enum ParseErr {
    NotUtf8(std::str::Utf8Error),
    IllFormed,
}

impl From<std::str::Utf8Error> for ParseErr {
    fn from(value: std::str::Utf8Error) -> Self {
        ParseErr::NotUtf8(value)
    }
}

#[cfg(feature = "cpp")]
fn ts_lang_cpp() -> Option<tree_sitter::Language> {
    Some(hyper_ast_gen_ts_cpp::language())
}
#[cfg(not(feature = "cpp"))]
fn ts_lang_cpp() -> Option<tree_sitter::Language> {
    None
}
#[cfg(feature = "java")]
fn ts_lang_java() -> Option<tree_sitter::Language> {
    Some(hyper_ast_gen_ts_java::language())
}
#[cfg(not(feature = "java"))]
fn ts_lang_java() -> Option<tree_sitter::Language> {
    None
}

pub fn resolve_language(language: &str) -> Option<tree_sitter::Language> {
    match language {
        "Java" | "java" => ts_lang_java(),
        "Cpp" | "cpp" => ts_lang_cpp(),
        _ => None
    }
}
