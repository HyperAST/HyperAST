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

trait Processor<Acc: Accumulator, Oid = self::Oid, O = BasicGitObject> {
    fn process(&mut self) -> Acc::Unlabeled {
        loop {
            if let Some(current_dir) = self.stack().last_mut().expect("never empty").1.pop() {
                self.pre(current_dir)
            } else if let Some((oid, _, acc)) = self.stack().pop() {
                if let Some(x) = self.post(oid, acc) {
                    return x;
                }
            } else {
                unreachable!("never empty")
            }
        }
    }
    fn stack(&mut self) -> &mut Vec<(Oid, Vec<O>, Acc)>;
    fn pre(&mut self, current_dir: O);
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
        _ => None,
    }
}

/// Identifying elements and fundamental derived metrics used to accelerate deduplication.
/// For example, hashing subtrees accelerates the deduplication process,
/// but it requires to hash children and it can be done by accumulating hashes iteratively per child (see [`hyper_ast::hashed::inner_node_hash`]).
pub struct BasicDirAcc<Id, L, M> {
    pub name: String,
    pub children: Vec<Id>,
    pub children_names: Vec<L>,
    pub metrics: M,
}

impl<Id, L, M: Default> BasicDirAcc<Id, L, M> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            children_names: Default::default(),
            children: Default::default(),
            metrics: Default::default(),
        }
    }
}

impl<Id, L, U: hyper_ast::hashed::NodeHashs>
    BasicDirAcc<Id, L, hyper_ast::tree_gen::SubTreeMetrics<U>>
{
    pub fn push(&mut self, name: L, id: Id, metrics: hyper_ast::tree_gen::SubTreeMetrics<U>) {
        self.children.push(id);
        self.children_names.push(name);
        self.metrics.acc(metrics);
    }
}

impl<Id, L, M> BasicDirAcc<Id, L, M> {
    pub fn map_metrics<N>(self, f: impl Fn(M) -> N) -> BasicDirAcc<Id, L, N> {
        BasicDirAcc {
            name: self.name,
            children: self.children,
            children_names: self.children_names,
            metrics: f(self.metrics),
        }
    }
}

impl<Id, L, M> BasicDirAcc<Id, L, M> {
    pub fn persist<K>(
        self,
        dyn_builder: &mut impl hyper_ast::store::nodes::EntityBuilder,
        interned_kind: K,
        label_id: L,
    ) -> M
    where
        K: 'static + Sized + hyper_ast::types::TypeTrait,
        L: 'static + std::marker::Send + std::marker::Sync,
        Id: 'static + std::marker::Send + std::marker::Sync,
    {
        dyn_builder.add(interned_kind);
        dyn_builder.add(label_id);

        let children = self.children;
        let children_names = self.children_names;
        assert_eq!(children_names.len(), children.len());
        if !children.is_empty() {
            use hyper_ast::store::nodes::legion::compo::CS;
            dyn_builder.add(CS(children_names.into_boxed_slice()));
            dyn_builder.add(CS(children.into_boxed_slice()));
        }
        self.metrics
    }
}
