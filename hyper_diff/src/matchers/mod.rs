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

// use crate::decompressed_tree_store::Persistable;
// impl<'a, Dsrc: Persistable, Ddst: Persistable, M> Mapping<Dsrc, Ddst, M> {
//     pub fn persist(
//         self,
//     ) -> Mapping<<Dsrc as Persistable>::Persisted, <Ddst as Persistable>::Persisted, M> {
//         Mapping {
//             src_arena: self.src_arena.persist(),
//             dst_arena: self.dst_arena.persist(),
//             mappings: self.mappings,
//         }
//     }
// }

// impl<'store, HAST, Dsrc: Persistable, Ddst: Persistable, M> Mapper<'store, HAST, Dsrc, Ddst, M> {
//     unsafe fn unpersist(
//         hyperast: &'store HAST,
//         p: Mapping<<Dsrc as Persistable>::Persisted, <Ddst as Persistable>::Persisted, M>,
//     ) -> Self {
//         Self {
//             hyperast,
//             mapping: Mapping {
//                 src_arena: unsafe { <Dsrc as Persistable>::unpersist(p.src_arena) },
//                 dst_arena: unsafe { <Ddst as Persistable>::unpersist(p.dst_arena) },
//                 mappings: p.mappings,
//             },
//         }
//     }
// }
// impl<'a, M> Mapping<CompletePostOrder<T, u32>, CompletePostOrder<T, u32>, M> {
//     pub fn persist(
//         self,
//     ) -> Mapping<CompletePostOrder<T, u32>, <Ddst as >::Persisted, M> {
//         Mapping {
//             src_arena: self.src_arena.persist(),
//             dst_arena: self.dst_arena.persist(),
//             mappings: self.mappings,
//         }
//     }
// }
use crate::decompressed_tree_store::{CompletePostOrder, PersistedNode};
impl<'store, HAST: HyperAST<'store>, M>
    Mapper<'store, HAST, CompletePostOrder<HAST::T, u32>, CompletePostOrder<HAST::T, u32>, M>
where
    HAST::IdN: Eq,
{
    pub fn persist(
        self,
    ) -> Mapping<
        CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
        CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
        M,
    > {
        let mapping = self.mapping;
        Mapping {
            src_arena: unsafe { std::mem::transmute(mapping.src_arena) },
            dst_arena: unsafe { std::mem::transmute(mapping.dst_arena) },
            mappings: mapping.mappings,
        }
    }
    /// used to enable easy caching of mappings
    /// safety: be sure to unpersist on the same HyperAST
    pub unsafe fn unpersist<'a>(
        _hyperast: &'store HAST,
        p: &'a Mapping<
            CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
            CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
            M,
        >,
    ) -> &'a Mapping<CompletePostOrder<HAST::T, u32>, CompletePostOrder<HAST::T, u32>, M> {
        unsafe { std::mem::transmute(p)}
    }
}
