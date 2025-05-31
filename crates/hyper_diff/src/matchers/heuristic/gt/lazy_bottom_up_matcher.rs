use crate::decompressed_tree_store::{
    DecompressedTreeStore, DecompressedWithParent, LazyDecompressed, LazyDecompressedTreeStore,
    Shallow,
};
use crate::matchers::mapping_store::MonoMappingStore;
use hyperast::{types::WithStats, PrimInt};
use num_traits::ToPrimitive;
pub struct BottomUpMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<
        Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
            + DecompressedWithParent<HAST, Dsrc::IdD>
            + LazyDecompressedTreeStore<HAST, M::Src>,
        Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
            + DecompressedWithParent<HAST, Ddst::IdD>
            + LazyDecompressedTreeStore<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
    > BottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
{
    pub(super) fn get_dst_candidates(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(src) {
            if self.mappings.is_src(&c) {
                let m = self.mappings.get_dst_unchecked(&c);
                let m = self.dst_arena.decompress_to(&m);
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

    pub(super) fn get_dst_candidates_lazily(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let src_arena = &self.src_arena;
        let dst_arena = &mut self.dst_arena;
        let mappings = &self.mappings;
        let mut seeds = vec![];
        let s = &src_arena.original(src);
        for c in src_arena.descendants(src) {
            if mappings.is_src(&c) {
                let m = mappings.get_dst_unchecked(&c);
                let m = dst_arena.decompress_to(&m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;dst_arena.len()];
        let t = self.stores.resolve_type(s);
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
                if self.stores.resolve_type(p) == t
                    && !(mappings.is_dst(parent.shallow()) || parent.shallow() == &dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }

    /// Returns true if *all* descendants in src are unmapped
    pub(super) fn are_srcs_unmapped(&self, src: &Dsrc::IdD) -> bool {
        self.src_arena
            .descendants(src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }

    /// Returns true if *all* descendants in dst are unmapped
    pub(super) fn are_dsts_unmapped(&self, dst: &Ddst::IdD) -> bool {
        self.dst_arena
            .descendants(dst)
            .iter()
            .all(|x| !self.mappings.is_dst(x))
    }

    /// Returns true if *any* descendants in src are unmapped
    pub(super) fn has_unmapped_src_children(&self, src: &Dsrc::IdD) -> bool {
        self.src_arena
            .descendants(src)
            .iter()
            .any(|x| !self.mappings.is_src(x))
    }
    
    /// Returns true if *any* descendants in dst are unmapped
    pub(super) fn has_unmapped_dst_children(&self, dst: &Ddst::IdD) -> bool {
        self.dst_arena
            .descendants(dst)
            .iter()
            .any(|x| !self.mappings.is_dst(x))
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &Dsrc::IdD, dst: &Ddst::IdD) {
        self.mappings.link(*src.shallow(), *dst.shallow());
        self.src_arena
            .descendants(src)
            .iter()
            .zip(self.dst_arena.descendants(dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }
}

use hyperast::types::HyperAST;

impl<
        HAST: HyperAST + Copy,
        Dsrc: LazyDecompressed<M::Src>,
        Ddst: LazyDecompressed<M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
{
    pub(super) fn get_dst_candidates_lazily(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let src_arena = &self.mapping.src_arena;
        let dst_arena = &mut self.mapping.dst_arena;
        let mappings = &self.mapping.mappings;
        let mut seeds = vec![];
        let s = &src_arena.original(src);
        for c in src_arena.descendants(src) {
            if mappings.is_src(&c) {
                let m = mappings.get_dst_unchecked(&c);
                let m = dst_arena.decompress_to(&m);
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
