//! Matchers associate nodes in pairs of tree.
//!
//! Originally, the matching was a phase in a tree-diff algorithm,
//! where interpreting the matchings would allow to produce a set of actions to transform a given tree into another.
//! In this context, the objective is to minimise the transformation cost, e.g., the number and types of actions.
//!
//! Later the notion of matchings was extended,
//! leading to many different matching approaches.
//! Certain matching approaches also consider more semantic interpretations.
//! Moreover, matchers can also be composed.

pub mod heuristic;
pub mod mapping_store;
pub mod optimal;
pub mod similarity_metrics;

#[cfg(test)]
mod tests;

use std::ops::{Deref, DerefMut};

use hyperast::types::{DecompressedFrom, HyperAST, HyperASTShared};

use crate::matchers::mapping_store::MappingStore;

pub struct Decompressible<HAST, D> {
    /// the HyperAST which is being decompressed
    pub hyperast: HAST,
    /// the structure containing the (partially) decompressed subtree
    pub decomp: D,
}

impl<HAST, D: std::fmt::Debug> std::fmt::Debug for Decompressible<HAST, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decompressible")
            // .field("hyperast", &self.hyperast)
            .field("decomp", &self.decomp)
            .finish()
    }
}

impl<HAST: HyperAST + Copy, D: DecompressedFrom<HAST>> DecompressedFrom<HAST>
    for Decompressible<HAST, D>
{
    type Out = Decompressible<HAST, D::Out>;

    fn decompress(hyperast: HAST, id: &HAST::IdN) -> Self::Out {
        Decompressible {
            hyperast,
            decomp: D::decompress(hyperast, id),
        }
    }
}

impl<HAST, D> std::ops::Deref for Decompressible<HAST, D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.decomp
    }
}

impl<HAST, D> Decompressible<HAST, D> {
    pub(crate) fn map<DD>(self, f: impl Fn(D) -> DD) -> Decompressible<HAST, DD> {
        Decompressible {
            hyperast: self.hyperast,
            decomp: f(self.decomp)
        }
    }
}

// impl<HAST, D> std::ops::Deref for Decompressible<HAST, &mut D> {
//     type Target = D;
//     fn deref(&self) -> &Self::Target {
//         self.decomp
//     }
// }

impl<HAST, D> std::ops::DerefMut for Decompressible<HAST, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decomp
    }
}

impl<HAST: Copy, D> Decompressible<HAST, D> {
    pub fn as_ref(&self) -> Decompressible<HAST, &D> {
        Decompressible {
            hyperast: self.hyperast,
            decomp: &self.decomp,
        }
    }
    pub fn as_mut(&mut self) -> Decompressible<HAST, &mut D> {
        Decompressible {
            hyperast: self.hyperast,
            decomp: &mut self.decomp,
        }
    }
}

pub struct Mapper<HAST, Dsrc, Ddst, M> {
    /// the hyperAST to whom mappings are coming
    pub hyperast: HAST,
    /// the decompressed subtrees coming from hyperAST and their mappings
    pub mapping: Mapping<Dsrc, Ddst, M>,
}

impl<HAST: Copy, Dsrc, Ddst, M> Mapper<HAST, Dsrc, Ddst, M> {
    fn split<'a>(
        &'a mut self,
    ) -> Mapping<Decompressible<HAST, &'a mut Dsrc>, Decompressible<HAST, &'a mut Ddst>, &'a mut M>
    {
        let hyperast = self.hyperast;
        let mapping = &mut self.mapping;
        Mapping {
            src_arena: Decompressible {
                hyperast,
                decomp: &mut mapping.src_arena,
            },
            dst_arena: Decompressible {
                hyperast,
                decomp: &mut mapping.dst_arena,
            },
            mappings: &mut mapping.mappings,
        }
    }
}
// NOTE this is temporary, waiting for the refactoring of helpers
// the refactoring is simple, do a spliting borrow, before accessing content
// TODO remove these deref impls
impl<HAST, Dsrc, Ddst, M> Deref for Mapper<HAST, Dsrc, Ddst, M> {
    type Target = Mapping<Dsrc, Ddst, M>;

    fn deref(&self) -> &Self::Target {
        &self.mapping
    }
}

impl<HAST, Dsrc, Ddst, M> DerefMut for Mapper<HAST, Dsrc, Ddst, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapping
    }
}

pub struct Mapping<Dsrc, Ddst, M> {
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<HAST, Dsrc, Ddst, M: MappingStore + Default> From<(HAST, (Dsrc, Ddst))>
    for Mapper<HAST, Dsrc, Ddst, M>
{
    fn from((hyperast, (src_arena, dst_arena)): (HAST, (Dsrc, Ddst))) -> Self {
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

impl<HAST, Dsrc, Ddst, M: MappingStore> Mapper<HAST, Dsrc, Ddst, M> {
    pub fn mappings(&self) -> &M {
        &self.mapping.mappings
    }
    pub fn map<Dsrc2, Ddst2, Fsrc: Fn(Dsrc) -> Dsrc2, Fdst: Fn(Ddst) -> Ddst2>(
        self,
        f_src: Fsrc,
        f_dst: Fdst,
    ) -> Mapper<HAST, Dsrc2, Ddst2, M> {
        Mapper {
            hyperast: self.hyperast,
            mapping: self.mapping.map(f_src, f_dst),
        }
    }
}

impl<Dsrc, Ddst, M: MappingStore> Mapping<Dsrc, Ddst, M> {
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

impl<HAST: HyperASTShared, Dsrc, Ddst, M> HyperASTShared for Mapper<HAST, Dsrc, Ddst, M> {
    type IdN = HAST::IdN;

    type Idx = HAST::Idx;

    type Label = HAST::Label;

    // type T<'t> = HAST::T<'t>;

    // type RT = HAST::RT;
}

// impl<TS, NS, LS>  for SimpleStores<TS, NS, LS>
// where
//     NS: crate::types::NStore,
//     NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
//     LS: crate::types::LStore,
//     <NS as crate::types::NStore>::IdN:
//         crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
//     for<'t> <NS as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N:
//         crate::types::Tree<
//             Label = <LS as crate::types::LStore>::I,
//             TreeId = <NS as crate::types::NStore>::IdN,
//             ChildIdx = <NS as crate::types::NStore>::Idx,
//         >,
// {
// }

impl<'a, HAST: HyperAST, Dsrc, Ddst, M> hyperast::types::NLending<'a, HAST::IdN>
    for Mapper<HAST, Dsrc, Ddst, M>
{
    type N = <HAST as hyperast::types::NLending<'a, HAST::IdN>>::N;
}

impl<'a, HAST: HyperAST, Dsrc, Ddst, M> hyperast::types::AstLending<'a>
    for Mapper<HAST, Dsrc, Ddst, M>
{
    type RT = <HAST as hyperast::types::AstLending<'a>>::RT;
}

impl<HAST: HyperAST, Dsrc, Ddst, M> HyperAST for Mapper<HAST, Dsrc, Ddst, M> {
    // type TM = HAST::TM;
    type NS = HAST::NS;

    fn node_store(&self) -> &Self::NS {
        self.hyperast.node_store()
    }

    type LS = HAST::LS;

    fn label_store(&self) -> &Self::LS {
        self.hyperast.label_store()
    }

    type TS = HAST::TS;
}

// use crate::decompressed_tree_store::Persistable;
// impl<Dsrc: Persistable, Ddst: Persistable, M> Mapping<Dsrc, Ddst, M> {
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

// impl<HAST, Dsrc: Persistable, Ddst: Persistable, M> Mapper<HAST, Dsrc, Ddst, M> {
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
// impl<M> Mapping<CompletePostOrder<T, u32>, CompletePostOrder<T, u32>, M> {
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
// impl<HAST: HyperAST, M>
//     Mapper<HAST, CompletePostOrder<HAST::IdN, u32>, CompletePostOrder<HAST::IdN, u32>, M>
// where
//     HAST::IdN: Eq,
// {
//     pub fn persist(
//         self,
//     ) -> Mapping<
//         CompletePostOrder<HAST::IdN, u32>,
//         CompletePostOrder<HAST::IdN, u32>,
//         M,
//     > {
//         let mapping = self.mapping;
//         Mapping {
//             src_arena: mapping.src_arena,// unsafe { std::mem::transmute(mapping.src_arena) },
//             dst_arena: mapping.dst_arena,// unsafe { std::mem::transmute(mapping.dst_arena) },
//             mappings: mapping.mappings,
//         }
//     }
//     /// used to enable easy caching of mappings
//     /// safety: be sure to unpersist on the same HyperAST
//     pub unsafe fn unpersist<'a>(
//         _hyperast: HAST,
//         p: &'a Mapping<
//             CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
//             CompletePostOrder<PersistedNode<HAST::IdN>, u32>,
//             M,
//         >,
//     ) -> &'a Mapping<CompletePostOrder<HAST::IdN, u32>, CompletePostOrder<HAST::IdN, u32>, M> {
//         unsafe { std::mem::transmute(p) }
//     }
// }

use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
// impl<HAST: HyperAST, M>
//     Mapper<HAST, LazyPostOrder<HAST::IdN, u32>, LazyPostOrder<HAST::IdN, u32>, M>
// where
//     HAST::IdN: Eq,
// {
//     pub fn persist(
//         self,
//     ) -> Mapping<
//         LazyPostOrder<PersistedNode<HAST::IdN>, u32>,
//         LazyPostOrder<PersistedNode<HAST::IdN>, u32>,
//         M,
//     > {
//         let mapping = self.mapping;
//         Mapping {
//             src_arena: unsafe { std::mem::transmute(mapping.src_arena) },
//             dst_arena: unsafe { std::mem::transmute(mapping.dst_arena) },
//             mappings: mapping.mappings,
//         }
//     }
//     /// used to enable easy caching of mappings
//     /// safety: be sure to unpersist on the same HyperAST
//     pub unsafe fn unpersist<'a>(
//         _hyperast: HAST,
//         p: &'a mut Mapping<
//             LazyPostOrder<PersistedNode<HAST::IdN>, u32>,
//             LazyPostOrder<PersistedNode<HAST::IdN>, u32>,
//             M,
//         >,
//     ) -> &'a mut Mapping<LazyPostOrder<HAST::IdN, u32>, LazyPostOrder<HAST::IdN, u32>, M> {
//         unsafe { std::mem::transmute(p) }
//     }
// }
