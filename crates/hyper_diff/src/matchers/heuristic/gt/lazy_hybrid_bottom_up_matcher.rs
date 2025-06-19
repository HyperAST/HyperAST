use super::lazy_bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::decompressed_tree_store::{
    LazyDecompressed, LazyDecompressedTreeStore, LazyPOBorrowSlice, Shallow,
    ShallowDecompressedTreeStore, SimpleZsTree as ZsTree,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Decompressible, Mapper};
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::PrimInt;
use hyperast::position::tags::TopDownNoSpace;
use hyperast::types::Childrn;
use hyperast::types::Labeled;
use hyperast::types::{DecompressedFrom, HashKind, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use hyperast::types::{TypeStore, WithChildren};
use num_traits::{cast, one};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Instant;

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
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
    max_size: usize,
    _phantom: PhantomData<*const MZs>,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
    Dsrc,
    Ddst,
    HAST: HyperAST,
    M: MonoMappingStore,
    MZs: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64, // = 1,
    const SIM_THRESHOLD_DEN: u64, // = 2,
> Into<BottomUpMatcher<Dsrc, Ddst, HAST, M>>
    for LazyHybridBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        MZs,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    fn into(self) -> BottomUpMatcher<Dsrc, Ddst, HAST, M> {
        self.internal
    }
}

/// TODO PostOrder might not be necessary
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
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
    LazyHybridBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        MZs,
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
    pub fn new(stores: HAST, src_arena: Dsrc, dst_arena: Ddst, mappings: M, max_size: usize) -> Self {
        Self {
            internal: BottomUpMatcher {
                stores,
                src_arena,
                dst_arena,
                mappings,
            },
            max_size,
            _phantom: Default::default(),
        }
    }

    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
        max_size: usize,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: BottomUpMatcher {
                stores: mapping.hyperast,
                src_arena: mapping.mapping.src_arena,
                dst_arena: mapping.mapping.dst_arena,
                mappings: mapping.mapping.mappings,
            },
            max_size,
            _phantom: Default::default(),
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute(&mut matcher);
        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.internal.src_arena,
                dst_arena: matcher.internal.dst_arena,
                mappings: matcher.internal.mappings,
            },
        }
    }

    // pub fn matchh(store: HAST, src: &'a HAST::IdN, dst: &'a HAST::IdN, mappings: M) -> Self {
    //     let mut matcher = Self::new(
    //         store,
    //         Dsrc::decompress(store, src),
    //         Ddst::decompress(store, dst),
    //         mappings,
    //     );
    //     matcher.internal.mappings.topit(
    //         matcher.internal.src_arena.len(),
    //         matcher.internal.dst_arena.len(),
    //     );
    //     Self::execute(&mut matcher);
    //     matcher
    // }

    pub fn execute<'b>(&mut self) {
        for t in self.internal.src_arena.iter_df_post::<false>() {
            // let path = self.internal.src_arena.path::<usize>(&self.internal.src_arena.root(), &t);
            // dbg!(path);
            let a = self.internal.src_arena.decompress_to(&t);
            if !self.internal.mappings.is_src(&t) && self.src_has_children(a) {
                let candidates = self.internal.get_dst_candidates_lazily(&a);
                let mut best = None;
                let mut max_sim = -1f64;
                for candidate in candidates {
                    let t_descendents = &self.internal.src_arena.descendants(&a);
                    let candidate_descendents = &self.internal.dst_arena.descendants(&candidate);
                    let sim = similarity_metrics::chawathe_similarity(
                        t_descendents,
                        candidate_descendents,
                        &self.internal.mappings,
                    );
                    let threshold = 1f64
                        / (1f64
                            + ((candidate_descendents.len() + t_descendents.len()) as f64).ln());
                    // SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                    if sim > max_sim && sim >= threshold {
                        max_sim = sim;
                        best = Some(candidate);
                    }
                }
                if let Some(best) = best {
                    let _ = self.internal.dst_arena.decompress_descendants(&best);
                    self.last_chance_match_hybrid(a, best);
                    self.internal.mappings.link(*a.shallow(), *best.shallow());
                }
            } else if self.internal.mappings.is_src(&t)
                && self.internal.has_unmapped_src_children(&a)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&t) {
                    let dst = self.internal.dst_arena.decompress_to(&dst);
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.last_chance_match_hybrid(a, dst);
                    }
                }
            }
        }

        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
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
        if self.internal.src_arena.descendants_count(&src) < self.max_size
            && self.internal.dst_arena.descendants_count(&dst) < self.max_size
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
            if self.internal.stores.resolve_type(
                &self
                    .internal
                    .src_arena
                    .original(&self.internal.src_arena.parent(&src).unwrap()),
            ) == self.internal.stores.resolve_type(
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
        for c in &self.internal.src_arena.children(&src) {
            if !self.internal.mappings.is_src(c) {
                src_children.push(self.internal.src_arena.decompress_to(c));
            }
        }
        let dst_children = &mut Vec::new();
        for c in &self.internal.dst_arena.children(&dst) {
            if !self.internal.mappings.is_dst(c) {
                dst_children.push(self.internal.dst_arena.decompress_to(c));
            }
        }

        let lcs =
            longest_common_subsequence::<_, _, usize, _>(src_children, dst_children, |src, dst| {
                cmp(self, *src, *dst)
            });
        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.internal.are_srcs_unmapped(&t1) && self.internal.are_dsts_unmapped(&t2) {
                self.internal.add_mapping_recursively(&t1, &t2);
            }
        }
    }

    /// Matches all pairs of nodes whose types appear only once in src and dst (step 3 of simple recovery)
    fn histogram_matching(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        let mut src_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Dsrc::IdD>> =
            HashMap::new();
        for c in self.internal.src_arena.children(&src) {
            if self.internal.mappings.is_src(&c) {
                continue;
            }
            let c = self.internal.src_arena.decompress_to(&c);
            let t = &self
                .internal
                .stores
                .resolve_type(&self.internal.src_arena.original(&c));
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<Ddst::IdD>> =
            HashMap::new();
        for c in self.internal.dst_arena.children(&dst) {
            if self.internal.mappings.is_dst(&c) {
                continue;
            }
            let c = self.internal.dst_arena.decompress_to(&c);
            let t = &self
                .internal
                .stores
                .resolve_type(&self.internal.dst_arena.original(&c));
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
    fn isomorphic<const structural: bool>(&self, src: Dsrc::IdD, dst: Ddst::IdD) -> bool {
        let src = self.internal.src_arena.original(&src);
        let dst = self.internal.dst_arena.original(&dst);

        self.isomorphic_aux::<true, structural>(&src, &dst)
    }
    fn isomorphic_aux<const use_hash: bool, const structural: bool>(
        &self,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        if src == dst {
            return true;
        }

        let _src = self.internal.stores.node_store().resolve(src);
        let _dst = self.internal.stores.node_store().resolve(dst);
        if use_hash {
            let src_hash = WithHashs::hash(&_src, &HashKind::label());
            let dst_hash = WithHashs::hash(&_dst, &HashKind::label());
            if src_hash != dst_hash {
                return false;
            }
        }

        let src_type = self.internal.stores.resolve_type(&src);
        let dst_type = self.internal.stores.resolve_type(&dst);
        if src_type != dst_type {
            return false;
        }

        if !structural {
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
                        if !self.isomorphic_aux::<false, structural>(src, dst) {
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
        let stores = self.internal.stores;
        let mapping = &mut self.internal;
        let src_arena = &mut mapping.src_arena;
        let dst_arena = &mut mapping.dst_arena;
        let src_s = src_arena.descendants_count(&src);
        let dst_s = dst_arena.descendants_count(&dst);
        if !(src_s < cast(self.max_size).unwrap() || dst_s < cast(self.max_size).unwrap()) {
            return;
        }
        let src_offset;
        let dst_offset;
        let zs_mappings: MZs = if SLICE {
            let src_arena = src_arena.slice_po(&src);
            src_offset = src - src_arena.root();
            let dst_arena = dst_arena.slice_po(&dst);
            dst_offset = dst - dst_arena.root();
            ZsMatcher::match_with(stores, src_arena, dst_arena)
        } else {
            let o_src = src_arena.original(&src);
            let o_dst = dst_arena.original(&dst);
            let src_arena = ZsTree::<HAST::IdN, Dsrc::IdD>::decompress(stores, &o_src);
            let src_arena = Decompressible {
                hyperast: stores,
                decomp: src_arena,
            };
            src_offset = src - src_arena.root();
            if cfg!(debug_assertions) {
                let src_arena_z = mapping.src_arena.slice_po(&src);
                for i in src_arena.iter_df_post::<true>() {
                    assert_eq!(src_arena.tree(&i), src_arena_z.tree(&i));
                    assert_eq!(src_arena.lld(&i), src_arena_z.lld(&i));
                }
                let mut last = src_arena_z.root();
                for k in src_arena_z.iter_kr() {
                    assert!(src_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(src_arena.kr[src_arena.kr.len() - 1]);
                dbg!(last == src_arena_z.root());
            }
            let dst_arena = ZsTree::<HAST::IdN, Ddst::IdD>::decompress(stores, &o_dst);
            let dst_arena = Decompressible {
                hyperast: stores,
                decomp: dst_arena,
            };
            dst_offset = dst - dst_arena.root();
            if cfg!(debug_assertions) {
                let dst_arena_z = mapping.dst_arena.slice_po(&dst);
                for i in dst_arena.iter_df_post::<true>() {
                    assert_eq!(dst_arena.tree(&i), dst_arena_z.tree(&i));
                    assert_eq!(dst_arena.lld(&i), dst_arena_z.lld(&i));
                }
                let mut last = dst_arena_z.root();
                for k in dst_arena_z.iter_kr() {
                    assert!(dst_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(dst_arena.kr[dst_arena.kr.len() - 1]);
                dbg!(last == dst_arena_z.root());
            }
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
            .stores
            .node_store()
            .resolve(&self.internal.src_arena.original(&src))
            .has_children()
    }
}
