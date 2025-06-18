use crate::decompressed_tree_store::{
    DecompressedTreeStore, DecompressedWithParent, LazyDecompressed, LazyDecompressedTreeStore,
    Shallow,
};
use crate::matchers::mapping_store::{MappingStore, MonoMappingStore};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::types::WithStats;
use hyperast::{
    PrimInt,
    types::{
        Childrn, HashKind, HyperAST, Labeled, NodeId, NodeStore, Tree, TypeStore, WithChildren,
        WithHashs,
    },
};
use num_traits::ToPrimitive;
use std::collections::HashMap;

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
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
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
            while let Some(parent) = self.dst_arena.parent(&seed) {
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);

                let p = &self.dst_arena.original(&parent);
                if self.stores.resolve_type(p) == t
                    && !self.mappings.is_dst(parent.shallow())
                    && parent.shallow() != &self.dst_arena.root()
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
            while let Some(parent) = dst_arena.parent(&seed) {
                // If visited break, otherwise mark as visisted
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);

                let p = &dst_arena.original(&parent);
                let p_type = self.stores.resolve_type(p);
                if p_type == t
                    && !mappings.is_dst(parent.shallow()) 
                    && parent.shallow() != &dst_arena.root()
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

    /// Return true if src has *any* children
    pub(super) fn src_has_children(&mut self, src: Dsrc::IdD) -> bool {
        self.stores
            .node_store()
            .resolve(&self.src_arena.original(&src))
            .has_children()
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &Dsrc::IdD, dst: &Ddst::IdD) {
        self.mappings.link(*src.shallow(), *dst.shallow());
        self.src_arena
            .descendants(src)
            .iter()
            .zip(self.dst_arena.descendants(dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    pub fn last_chance_match_histogram(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);

        let src_is_root = self.src_arena.parent(&src).is_none();
        let dst_is_root = self.dst_arena.parent(&dst).is_none();
        if src_is_root && dst_is_root {
            self.histogram_matching(src, dst);
        } else if !(src_is_root || dst_is_root) {
            let src_type = self.stores.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(&src).unwrap()),
            );
            let dst_type = self.stores.resolve_type(
                &self
                    .dst_arena
                    .original(&self.dst_arena.parent(&dst).unwrap()),
            );
            if src_type == dst_type {
                self.histogram_matching(src, dst)
            }
        }
    }

    /// Matches all strictly isomorphic nodes in the descendants of src and dst (step 1 of simple recovery)
    fn lcs_equal_matching(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<false>(src, dst))
    }

    /// Matches all structurally isomorphic nodes in the descendants of src and dst (step 2 of simple recovery)
    fn lcs_structure_matching(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<true>(src, dst))
    }

    fn lcs_matching<F: Fn(&Self, Dsrc::IdD, Ddst::IdD) -> bool>(
        &mut self,
        src: Dsrc::IdD,
        dst: Ddst::IdD,
        cmp: F,
    ) {
        //!TODO is there a way to do it without decompressing children?
        let src_children: Vec<<Dsrc as LazyDecompressed<<M as MappingStore>::Src>>::IdD> = self
            .src_arena
            .decompress_children(&src)
            .into_iter()
            .filter(|child| !self.mappings.is_src(child.shallow()))
            .collect();

        let dst_children: Vec<<Ddst as LazyDecompressed<<M as MappingStore>::Dst>>::IdD> = self
            .dst_arena
            .decompress_children(&dst)
            .into_iter()
            .filter(|child| !self.mappings.is_dst(child.shallow()))
            .collect();

        let lcs = longest_common_subsequence::<_, _, usize, _>(
            &src_children,
            &dst_children,
            |src, dst| cmp(self, *src, *dst),
        );

        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.are_srcs_unmapped(&t1) && self.are_dsts_unmapped(&t2) {
                self.add_mapping_recursively(&t1, &t2);
            }
        }
    }

    /// Matches all pairs of nodes whose types appear only once in src and dst (step 3 of simple recovery)
    fn histogram_matching(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        let src_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Dsrc::IdD>> = self
            .src_arena
            .decompress_children(&src)
            .into_iter()
            .filter(|child| !self.mappings.is_src(&child.shallow()))
            .fold(HashMap::new(), |mut acc, child| {
                let child_type = self.stores.resolve_type(&self.src_arena.original(&child));
                acc.entry(child_type).or_insert_with(Vec::new).push(child);
                acc
            });

        let dst_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Ddst::IdD>> = self
            .dst_arena
            .decompress_children(&dst)
            .into_iter()
            .filter(|child| !self.mappings.is_dst(&child.shallow()))
            .fold(HashMap::new(), |mut acc, child| {
                let child_type = self.stores.resolve_type(&self.dst_arena.original(&child));
                acc.entry(child_type).or_insert_with(Vec::new).push(child);
                acc
            });

        for src_type in src_histogram.keys() {
            if dst_histogram.contains_key(src_type)
                && src_histogram[src_type].len() == 1
                && dst_histogram[src_type].len() == 1
            {
                let t1 = src_histogram[src_type][0];
                let t2 = dst_histogram[src_type][0];
                self.mappings
                    .link_if_both_unmapped(*t1.shallow(), *t2.shallow());
                self.last_chance_match_histogram(t1, t2);
            }
        }
    }

    /// Checks if src and dst are (structurally) isomorphic
    fn isomorphic<const STRUCTURAL: bool>(&self, src: Dsrc::IdD, dst: Ddst::IdD) -> bool {
        let src = self.src_arena.original(&src);
        let dst = self.dst_arena.original(&dst);

        self.isomorphic_aux::<true, STRUCTURAL>(&src, &dst)
    }

    fn isomorphic_aux<const USE_HASH: bool, const STRUCTURAL: bool>(
        &self,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        // trivial case, they are the same
        if src == dst {
            return true;
        }

        // Compare hashes, if the 'use_hash' flag is set. If the hashes arent equal we return false
        let _src = self.stores.node_store().resolve(src);
        let _dst = self.stores.node_store().resolve(dst);
        if USE_HASH {
            let src_hash = WithHashs::hash(&_src, &HashKind::label());
            let dst_hash = WithHashs::hash(&_dst, &HashKind::label());
            if src_hash != dst_hash {
                return false;
            }
        }

        // If the types aren't the same, we return false
        let src_type = self.stores.resolve_type(&src);
        let dst_type = self.stores.resolve_type(&dst);
        if src_type != dst_type {
            return false;
        }

        // If the structural flag is set, we compare labels and return false if they are not equal
        if !STRUCTURAL {
            let src_label = _src.try_get_label();
            let dst_label = _dst.try_get_label();
            if src_label != dst_label {
                return false;
            }
        }

        // If none of the previous comparisons were conclusive we will look at the children
        let src_children: Option<Vec<_>> = _src.children().map(|x| x.iter_children().collect());
        let dst_children: Option<Vec<_>> = _dst.children().map(|x| x.iter_children().collect());
        match (src_children, dst_children) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                if src_c.len() != dst_c.len() {
                    false
                } else {
                    for (src, dst) in src_c.iter().zip(dst_c.iter()) {
                        if !self.isomorphic_aux::<false, STRUCTURAL>(src, dst) {
                            return false;
                        }
                    }
                    true
                }
            }
            _ => false,
        }
    }
}

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
