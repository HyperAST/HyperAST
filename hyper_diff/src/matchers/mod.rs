pub mod heuristic;
pub mod mapping_store;
pub mod matcher;
pub mod optimal;
pub mod similarity_metrics;

#[cfg(test)]
mod tests;

use std::ops::{Deref, DerefMut};

use hyper_ast::types::HyperAST;

use crate::matchers::mapping_store::MappingStore;

pub struct Mapper<'store, HAST, Dsrc, Ddst, M> {
    /// the hyperAST to whom mappings are coming
    pub hyperast: &'store HAST,
    /// the decompressed subtrees coming from hyperAST and their mappings
    pub mapping: Mapping<Dsrc, Ddst, M>,
}
// NOTE this is temporary, waiting for the refactoring of helpers
// the refactoring is simple, do a spliting borrow, before accessing content
// TODO remove these deref impls
impl<'store, HAST, Dsrc, Ddst, M> Deref for Mapper<'store, HAST, Dsrc, Ddst, M> {
    type Target = Mapping<Dsrc, Ddst, M>;

    fn deref(&self) -> &Self::Target {
        &self.mapping
    }
}

impl<'store, HAST, Dsrc, Ddst, M> DerefMut for Mapper<'store, HAST, Dsrc, Ddst, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapping
    }
}

pub struct Mapping<Dsrc, Ddst, M> {
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<'store, HAST, Dsrc, Ddst, M: MappingStore> From<(&'store HAST, (Dsrc, Ddst))>
    for Mapper<'store, HAST, Dsrc, Ddst, M>
{
    fn from((hyperast, (src_arena, dst_arena)): (&'store HAST, (Dsrc, Ddst))) -> Self {
        let mappings = M::default();
        Self {
            hyperast,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        }
    }
}

impl<'a, HAST, Dsrc, Ddst, M: MappingStore> Mapper<'a, HAST, Dsrc, Ddst, M> {
    pub fn mappings(&self) -> &M {
        &self.mapping.mappings
    }
    pub fn map<Dsrc2, Ddst2, Fsrc: Fn(Dsrc) -> Dsrc2, Fdst: Fn(Ddst) -> Ddst2>(
        self,
        f_src: Fsrc,
        f_dst: Fdst,
    ) -> Mapper<'a, HAST, Dsrc2, Ddst2, M> {
        Mapper {
            hyperast: self.hyperast,
            mapping: self.mapping.map(f_src, f_dst),
        }
    }
}

impl<'a, Dsrc, Ddst, M: MappingStore> Mapping<Dsrc, Ddst, M> {
    pub fn map<Dsrc2, Ddst2, Fsrc: Fn(Dsrc) -> Dsrc2, Fdst: Fn(Ddst) -> Ddst2>(
        self,
        f_src: Fsrc,
        f_dst: Fdst,
    ) -> Mapping<Dsrc2, Ddst2, M> {
        Mapping {
            src_arena: f_src(self.src_arena),
            dst_arena: f_dst(self.dst_arena),
            mappings: self.mappings,
        }
    }
}

impl<'store, HAST: HyperAST<'store>, Dsrc, Ddst, M> HyperAST<'store>
    for Mapper<'store, HAST, Dsrc, Ddst, M>
{
    type IdN = HAST::IdN;

    type Label = HAST::Label;

    type T = HAST::T;

    type NS = HAST::NS;

    fn node_store(&self) -> &Self::NS {
        self.hyperast.node_store()
    }

    type LS = HAST::LS;

    fn label_store(&self) -> &Self::LS {
        self.hyperast.label_store()
    }
}
