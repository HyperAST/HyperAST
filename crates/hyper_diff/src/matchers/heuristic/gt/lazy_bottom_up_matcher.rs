use std::{fmt::Debug, marker::PhantomData};

use num_traits::{PrimInt, ToPrimitive};

use crate::{
    decompressed_tree_store::{
        DecompressedTreeStore, DecompressedWithParent, LazyDecompressed, LazyDecompressedTreeStore,
        Shallow,
    },
    matchers::mapping_store::MonoMappingStore,
};
use hyperast::types::{Tree, WithStats};

pub struct BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M> {
    pub(super) stores: &'a HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub(super) _phantom: PhantomData<*const T>,
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<HAST::TM, Dsrc::IdD, M::Src>
            + DecompressedWithParent<HAST::TM, Dsrc::IdD>
            + LazyDecompressedTreeStore<HAST::TM, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<HAST::TM, Ddst::IdD, M::Dst>
            + DecompressedWithParent<HAST::TM, Ddst::IdD>
            + LazyDecompressedTreeStore<HAST::TM, M::Dst>,
        // T: Tree + WithStats,
        HAST: HyperAST,
        M: MonoMappingStore,
    > BottomUpMatcher<'a, Dsrc, Ddst, HAST::TM, HAST, M>
where
    // T::Type: Copy + Eq + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
    Dsrc::IdD: PrimInt + std::ops::SubAssign + Debug,
    Ddst::IdD: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn get_dst_candidates(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(self.stores.node_store(), src) {
            if self.mappings.is_src(&c) {
                let m = self.mappings.get_dst_unchecked(&c);
                let m = self.dst_arena.decompress_to(self.stores.node_store(), &m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;self.dst_arena.len()];
        let t = self.stores.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = self.dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &self.dst_arena.original(&parent);
                if self.stores.resolve_type(p) == t
                    && !(self.mappings.is_dst(parent.shallow())
                        || parent.shallow() == &self.dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}

use hyperast::types::HyperAST;

impl<
        'a,
        HAST: HyperAST,
        Dsrc: LazyDecompressed<M::Src>,
        Ddst: LazyDecompressed<M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
    // <HAST::T as Typed>::Type: Eq + Copy + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
    Dsrc::IdD: PrimInt + std::ops::SubAssign + Debug,
    Ddst::IdD: PrimInt + std::ops::SubAssign + Debug,
    Dsrc: DecompressedTreeStore<HAST::TM, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST::TM, Dsrc::IdD>
        + LazyDecompressedTreeStore<HAST::TM, M::Src>,
    Ddst: DecompressedTreeStore<HAST::TM, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST::TM, Ddst::IdD>
        + LazyDecompressedTreeStore<HAST::TM, M::Dst>,
{
    pub(super) fn get_dst_candidates_lazily(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let node_store = self.hyperast.node_store();
        let src_arena = &self.mapping.src_arena;
        let dst_arena = &mut self.mapping.dst_arena;
        let mappings = &self.mapping.mappings;
        let mut seeds = vec![];
        let s = &src_arena.original(src);
        for c in src_arena.descendants2(self.hyperast, src) {
            if mappings.is_src(&c) {
                let m = mappings.get_dst_unchecked(&c);
                let m = dst_arena.decompress_to(node_store, &m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;dst_arena.len()];
        let t = self.hyperast.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &dst_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !(mappings.is_dst(parent.shallow()) || parent.shallow() == &dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}
