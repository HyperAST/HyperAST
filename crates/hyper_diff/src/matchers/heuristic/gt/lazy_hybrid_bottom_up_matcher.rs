use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, PostOrder,
    PostOrderIterable,
};
use crate::decompressed_tree_store::{
    LazyDecompressed, LazyDecompressedTreeStore, LazyPOBorrowSlice, Shallow,
    ShallowDecompressedTreeStore,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::PrimInt;
use hyperast::types::Childrn;
use hyperast::types::Labeled;
use hyperast::types::{HashKind, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use hyperast::types::{TypeStore, WithChildren};
use num_traits::cast;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct LazyHybridBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    MZs: MonoMappingStore = M,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
    _phantom: PhantomData<*const MZs>,
}

impl<
    'a,
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyPOBorrowSlice<HAST, Dsrc::IdD, M::Src>
        + ShallowDecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>
        + LazyDecompressed<M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyPOBorrowSlice<HAST, Ddst::IdD, M::Dst>
        + ShallowDecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>
        + LazyDecompressed<M::Dst>,
    MZs: MonoMappingStore<Src = Dsrc::IdD, Dst = <Ddst as LazyDecompressed<M::Dst>>::IdD> + Default,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    const SIZE_THRESHOLD: usize,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
    LazyHybridBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        MZs,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: mapping,
            _phantom: Default::default(),
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher);
        crate::matchers::Mapper {
            hyperast: matcher.internal.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.internal.mapping.src_arena,
                dst_arena: matcher.internal.mapping.dst_arena,
                mappings: matcher.internal.mapping.mappings,
            },
        }
    }

    pub fn execute<'b>(&mut self) {
        for t in self.internal.src_arena.iter_df_post::<false>() {
            let a = self.internal.src_arena.decompress_to(&t);
            if !self.internal.mappings.is_src(&t) && self.src_has_children(a) {
                let candidates = self.internal.get_dst_candidates_lazily(&a);
                let mut best = None;
                let mut max_sim = -1f64;
                for candidate in candidates {
                    let t_descendents = self.internal.src_arena.descendants_range(&a);
                    let candidate_descendents =
                        self.internal.dst_arena.descendants_range(&candidate);
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &t_descendents,
                        &candidate_descendents,
                        &self.internal.mappings,
                    )
                    .chawathe();
                    let threshold = 1f64
                        / (1f64
                            + ((self.internal.dst_arena.descendants_count(&candidate)
                                + self.internal.src_arena.descendants_count(&a))
                                as f64)
                                .ln());
                    if sim > max_sim && sim >= threshold {
                        max_sim = sim;
                        best = Some(candidate);
                    }
                }
                if let Some(best) = best {
                    self.last_chance_match_hybrid(a, best);
                    self.internal.mappings.link(*a.shallow(), *best.shallow());
                }
            } else if self.internal.mappings.is_src(&t)
                && self.internal.has_unmapped_src_children_lazy(&a)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&t) {
                    let dst = self.internal.dst_arena.decompress_to(&dst);
                    if self.internal.has_unmapped_dst_children_lazy(&dst) {
                        self.last_chance_match_hybrid(a, dst);
                    }
                }
            }
        }

        self.internal.mapping.mappings.link(
            self.internal.mapping.src_arena.root(),
            self.internal.mapping.dst_arena.root(),
        );
        self.last_chance_match_hybrid(
            self.internal.src_arena.starter(),
            self.internal.dst_arena.starter(),
        );
    }

    /// Hybrid recovery algorithm (finds mappings between src and dst descendants)
    /// Uses ZS (optimal) if the number of descendents is below SIZE_THRESHOLD
    /// Uses simple recovery otherwise
    fn last_chance_match_hybrid(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        if self.internal.src_arena.descendants_count(&src) < SIZE_THRESHOLD
            && self.internal.dst_arena.descendants_count(&dst) < SIZE_THRESHOLD
        {
            self.last_chance_match_zs(src, dst);
        } else {
            self.last_chance_match_histogram(src, dst);
        }
    }

    /// Simple recovery algorithm (finds mappings between src and dst descendants)
    fn last_chance_match_histogram(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);
        let src_is_root = self.internal.src_arena.parent(&src).is_none();
        let dst_is_root = self.internal.dst_arena.parent(&dst).is_none();
        if src_is_root && dst_is_root {
            self.histogram_matching(src, dst);
        } else if !(src_is_root || dst_is_root) {
            if self.internal.hyperast.resolve_type(
                &self
                    .internal
                    .src_arena
                    .original(&self.internal.src_arena.parent(&src).unwrap()),
            ) == self.internal.hyperast.resolve_type(
                &self
                    .internal
                    .dst_arena
                    .original(&self.internal.dst_arena.parent(&dst).unwrap()),
            ) {
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
        let src_children = &mut Vec::new();
        for c in self.internal.src_arena.decompress_children(&src) {
            if !self.internal.mappings.is_src(c.shallow()) {
                src_children.push(c);
            }
        }
        let dst_children = &mut Vec::new();
        for c in self.internal.dst_arena.decompress_children(&dst) {
            if !self.internal.mappings.is_dst(c.shallow()) {
                dst_children.push(c);
            }
        }

        let lcs =
            longest_common_subsequence::<_, _, usize, _>(src_children, dst_children, |src, dst| {
                cmp(self, *src, *dst)
            });
        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.internal.are_srcs_unmapped_lazy(&t1)
                && self.internal.are_dsts_unmapped_lazy(&t2)
            {
                self.internal.add_mapping_recursively_lazy(&t1, &t2);
            }
        }
    }

    /// Matches all pairs of nodes whose types appear only once in src and dst (step 3 of simple recovery)
    fn histogram_matching(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        let mut src_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Dsrc::IdD>> =
            HashMap::new();
        for c in self.internal.src_arena.decompress_children(&src) {
            if self.internal.mappings.is_src(c.shallow()) {
                continue;
            }
            let t = &(self.internal.hyperast).resolve_type(&self.internal.src_arena.original(&c));
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Ddst::IdD>> =
            HashMap::new();
        for c in self.internal.dst_arena.decompress_children(&dst) {
            if self.internal.mappings.is_dst(c.shallow()) {
                continue;
            }
            let t = &(self.internal.hyperast).resolve_type(&self.internal.dst_arena.original(&c));
            if !dst_histogram.contains_key(t) {
                dst_histogram.insert(*t, vec![]);
            }
            dst_histogram.get_mut(t).unwrap().push(c);
        }
        for t in src_histogram.keys() {
            if dst_histogram.contains_key(t)
                && src_histogram[t].len() == 1
                && dst_histogram[t].len() == 1
            {
                let t1 = src_histogram[t][0];
                let t2 = dst_histogram[t][0];
                self.internal.mappings.link(*t1.shallow(), *t2.shallow());
                self.last_chance_match_histogram(t1, t2);
            }
        }
    }

    /// Checks if src and dst are (structurally) isomorphic
    fn isomorphic<const STRUCTURAL: bool>(&self, src: Dsrc::IdD, dst: Ddst::IdD) -> bool {
        let src = self.internal.src_arena.original(&src);
        let dst = self.internal.dst_arena.original(&dst);

        self.isomorphic_aux::<true, STRUCTURAL>(&src, &dst)
    }

    fn isomorphic_aux<const USE_HASH: bool, const STRUCTURAL: bool>(
        &self,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        if src == dst {
            return true;
        }

        let _src = self.internal.hyperast.node_store().resolve(src);
        let _dst = self.internal.hyperast.node_store().resolve(dst);
        if USE_HASH {
            let src_hash = WithHashs::hash(&_src, &HashKind::label());
            let dst_hash = WithHashs::hash(&_dst, &HashKind::label());
            if src_hash != dst_hash {
                return false;
            }
        }

        let src_type = self.internal.hyperast.resolve_type(&src);
        let dst_type = self.internal.hyperast.resolve_type(&dst);
        if src_type != dst_type {
            return false;
        }

        if !STRUCTURAL {
            let src_label = _src.try_get_label();
            let dst_label = _dst.try_get_label();
            if src_label != dst_label {
                return false;
            }
        }

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

    /// Optimal ZS recovery algorithm (finds mappings between src and dst descendants)
    fn last_chance_match_zs(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        let stores = self.internal.hyperast;
        let mapping = &mut self.internal.mapping;
        let src_arena = &mut mapping.src_arena;
        let dst_arena = &mut mapping.dst_arena;
        let src_s = src_arena.descendants_count(&src);
        let dst_s = dst_arena.descendants_count(&dst);
        if !(src_s < SIZE_THRESHOLD || dst_s < SIZE_THRESHOLD) {
            return;
        }
        let src_offset;
        let dst_offset;
        let zs_mappings: MZs = {
            let src_arena = src_arena.slice_po(&src);
            src_offset = src - src_arena.root();
            let dst_arena = dst_arena.slice_po(&dst);
            dst_offset = dst - dst_arena.root();
            ZsMatcher::match_with(stores, src_arena, dst_arena)
        };
        use num_traits::ToPrimitive;
        assert_eq!(
            mapping.src_arena.first_descendant(&src).to_usize(),
            src_offset.to_usize()
        );
        let mappings = &mut mapping.mappings;
        for (i, t) in zs_mappings.iter() {
            //remapping
            let src: Dsrc::IdD = src_offset.clone() + cast(i).unwrap();
            let dst: Ddst::IdD = dst_offset.clone() + cast(t).unwrap();
            // use it
            if !mappings.is_src(src.shallow()) && !mappings.is_dst(dst.shallow()) {
                let tsrc = stores.resolve_type(&mapping.src_arena.original(&src));
                let tdst = stores.resolve_type(&mapping.dst_arena.original(&dst));
                if tsrc == tdst {
                    mappings.link(*src.shallow(), *dst.shallow());
                }
            }
        }
    }

    fn src_has_children(&mut self, src: Dsrc::IdD) -> bool {
        self.internal
            .hyperast
            .node_store()
            .resolve(&self.internal.src_arena.original(&src))
            .has_children()
    }
}
