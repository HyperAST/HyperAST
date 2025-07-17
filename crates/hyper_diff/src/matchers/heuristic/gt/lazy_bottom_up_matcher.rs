use crate::decompressed_tree_store::{
    DecompressedTreeStore, DecompressedWithParent, LazyDecompressed, LazyDecompressedTreeStore,
    Shallow,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use hyperast::PrimInt;
use hyperast::compat::HashMap;
use hyperast::types::{HyperAST, NodeId, NodeStore as _, WithHashs};
use num_traits::ToPrimitive;

impl<
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src> + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst> + LazyDecompressedTreeStore<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> Mapper<HAST, Dsrc, Ddst, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    /// Returns true if *all* descendants in src are unmapped
    pub(super) fn are_srcs_unmapped_lazy(&self, src: &Dsrc::IdD) -> bool {
        self.src_arena
            .descendants(src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }

    /// Returns true if *all* descendants in dst are unmapped
    pub(super) fn are_dsts_unmapped_lazy(&self, dst: &Ddst::IdD) -> bool {
        self.dst_arena
            .descendants(dst)
            .iter()
            .all(|x| !self.mappings.is_dst(x))
    }

    /// Returns true if *any* descendants in src are unmapped
    pub(super) fn has_unmapped_src_children_lazy(&self, src: &Dsrc::IdD) -> bool {
        self.src_arena
            .descendants(src)
            .iter()
            .any(|x| !self.mappings.is_src(x))
    }

    /// Returns true if *any* descendants in dst are unmapped
    pub(super) fn has_unmapped_dst_children_lazy(&self, dst: &Ddst::IdD) -> bool {
        self.dst_arena
            .descendants(dst)
            .iter()
            .any(|x| !self.mappings.is_dst(x))
    }

    pub(super) fn src_has_children_lazy(&mut self, src: Dsrc::IdD) -> bool {
        use hyperast::types::Tree;
        self.hyperast
            .node_store()
            .resolve(&self.src_arena.original(&src))
            .has_children()
    }

    pub(crate) fn add_mapping_recursively_lazy(&mut self, src: &Dsrc::IdD, dst: &Ddst::IdD) {
        self.mappings
            .link(src.shallow().clone(), dst.shallow().clone());
        // WARN check if it works well
        let src = self.src_arena.descendants(src);
        let dst = self.dst_arena.descendants(dst);
        src.iter()
            .zip(dst.iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    /// Matches all strictly isomorphic nodes in the descendants of src and dst (step 1 of simple recovery)
    fn lcs_equal_matching_lazy(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_matching_lazy(src, dst, move |s, src, dst| {
            s.isomorphic_lazy::<false>(src, dst)
        })
    }

    /// Matches all structurally isomorphic nodes in the descendants of src and dst (step 2 of simple recovery)
    fn lcs_structure_matching_lazy(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_matching_lazy(src, dst, move |s, src, dst| {
            s.isomorphic_lazy::<true>(src, dst)
        })
    }

    fn lcs_matching_lazy(
        &mut self,
        src: Dsrc::IdD,
        dst: Ddst::IdD,
        cmp: impl Fn(&Self, Dsrc::IdD, Ddst::IdD) -> bool,
    ) {
        let src_children = self
            .src_arena
            .decompress_children(&src)
            .into_iter()
            .filter(|child| !self.mappings.is_src(child.shallow()))
            .collect::<Vec<_>>();

        let dst_children = self
            .dst_arena
            .decompress_children(&dst)
            .into_iter()
            .filter(|child| !self.mappings.is_dst(child.shallow()))
            .collect::<Vec<_>>();

        use crate::utils::sequence_algorithms::longest_common_subsequence;
        let lcs = longest_common_subsequence::<_, _, usize, _>(
            &src_children,
            &dst_children,
            |src, dst| cmp(self, *src, *dst),
        );

        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.are_srcs_unmapped_lazy(&t1) && self.are_dsts_unmapped_lazy(&t2) {
                self.add_mapping_recursively_lazy(&t1, &t2);
            }
        }
    }

    /// Checks if src and dst are (structurally) isomorphic
    fn isomorphic_lazy<const STRUCTURAL: bool>(&self, src: Dsrc::IdD, dst: Ddst::IdD) -> bool {
        let src = self.src_arena.original(&src);
        let dst = self.dst_arena.original(&dst);
        super::isomorphic::<_, true, STRUCTURAL>(self.hyperast, &src, &dst)
    }
}

impl<
    HAST: HyperAST + Copy,
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    M: MonoMappingStore,
> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
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
    pub(super) fn get_src_candidates_lazily(&mut self, dst: &Ddst::IdD) -> Vec<Dsrc::IdD> {
        let src_arena = &mut self.mapping.src_arena;
        let dst_arena = &self.mapping.dst_arena;
        let mappings = &self.mapping.mappings;
        let mut seeds = vec![];
        let s = &dst_arena.original(dst);
        for c in dst_arena.descendants(dst) {
            if mappings.is_dst(&c) {
                let m = mappings.get_src_unchecked(&c);
                let m = src_arena.decompress_to(&m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;src_arena.len()];
        let t = self.hyperast.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = src_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &src_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !(mappings.is_src(parent.shallow()) || parent.shallow() == &src_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }

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
            while let Some(parent) = dst_arena.parent(&seed) {
                // If visited break, otherwise mark as visisted
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);

                let p = &dst_arena.original(&parent);
                let p_type = self.hyperast.resolve_type(p);
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
}

impl<
    HAST: HyperAST + Copy,
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    M: MonoMappingStore,
> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
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
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
{
    pub fn last_chance_match_histogram_lazy(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_equal_matching_lazy(src, dst);
        self.lcs_structure_matching_lazy(src, dst);

        let src_is_root = self.src_arena.parent(&src).is_none();
        let dst_is_root = self.dst_arena.parent(&dst).is_none();
        if src_is_root && dst_is_root {
            self.histogram_matching_lazy(src, dst);
        } else if !(src_is_root || dst_is_root) {
            let src_type = self.hyperast.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(&src).unwrap()),
            );
            let dst_type = self.hyperast.resolve_type(
                &self
                    .dst_arena
                    .original(&self.dst_arena.parent(&dst).unwrap()),
            );
            if src_type == dst_type {
                self.histogram_matching_lazy(src, dst)
            }
        }
    }

    /// Matches all pairs of nodes whose types appear only once in src and dst (step 3 of simple recovery)
    fn histogram_matching_lazy(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        let src_histogram: HashMap<_, Vec<Dsrc::IdD>> = self
            .src_arena
            .decompress_children(&src)
            .into_iter()
            .filter(|child| !self.mappings.is_src(&child.shallow()))
            .fold(HashMap::new(), |mut acc, child| {
                let child_type = self.hyperast.resolve_type(&self.src_arena.original(&child));
                acc.entry(child_type).or_insert_with(Vec::new).push(child);
                acc
            });

        let dst_histogram: HashMap<_, Vec<Ddst::IdD>> = self
            .dst_arena
            .decompress_children(&dst)
            .into_iter()
            .filter(|child| !self.mappings.is_dst(&child.shallow()))
            .fold(HashMap::new(), |mut acc, child| {
                let child_type = self.hyperast.resolve_type(&self.dst_arena.original(&child));
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
                self.last_chance_match_histogram_lazy(t1, t2);
            }
        }
    }
}
