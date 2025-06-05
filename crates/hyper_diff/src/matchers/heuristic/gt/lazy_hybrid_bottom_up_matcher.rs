use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, PostOrderKeyRoots,
    Shallow, ShallowDecompressedTreeStore, SimpleZsTree as ZsTree,
};
use crate::matchers::{
    Decompressible, heuristic::gt::lazy_bottom_up_matcher::BottomUpMatcher,
    mapping_store::MonoMappingStore, optimal::zs::ZsMatcher, similarity_metrics,
};
use hyperast::{
    PrimInt,
    types::{DecompressedFrom, HyperAST, NodeId, WithHashs},
};
use num_traits::cast;
use std::{fmt::Debug, marker::PhantomData};

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
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
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
    const SIZE_THRESHOLD: usize,  // = 1000,
    const SIM_THRESHOLD_NUM: u64, // = 1,
    const SIM_THRESHOLD_DEN: u64, // = 2,
> Into<BottomUpMatcher<Dsrc, Ddst, HAST, M>>
    for LazyHybridBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        MZs,
        SIZE_THRESHOLD,
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
    pub fn new(stores: HAST, src_arena: Dsrc, dst_arena: Ddst, mappings: M) -> Self {
        Self {
            internal: BottomUpMatcher {
                stores,
                src_arena,
                dst_arena,
                mappings,
            },
            _phantom: Default::default(),
        }
    }

    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: BottomUpMatcher {
                stores: mapping.hyperast,
                src_arena: mapping.mapping.src_arena,
                dst_arena: mapping.mapping.dst_arena,
                mappings: mapping.mapping.mappings,
            },
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

    pub fn execute<'b>(&mut self) {
        for node in self.internal.src_arena.iter_df_post::<true>() {
            // let path = self.internal.src_arena.path::<usize>(&self.internal.src_arena.root(), &t);
            // dbg!(path);
            let decompressed_node = self.internal.src_arena.decompress_to(&node);
            if self.internal.src_arena.parent(&decompressed_node).is_none() {
                self.internal.mappings.link(
                    self.internal.src_arena.root(),
                    self.internal.dst_arena.root(),
                );
                self.last_chance_match_hybrid(
                    self.internal.src_arena.starter(),
                    self.internal.dst_arena.starter(),
                );
                break;
            } else if !(self.internal.mappings.is_src(&node)
                || !self.internal.src_has_children(decompressed_node))
            {
                let candidates = self.internal.get_dst_candidates_lazily(&decompressed_node);
                let mut best = None;
                let mut max_sim = -1f64;
                for candidate in candidates {
                    let t_descendents = &self.internal.src_arena.descendants(&decompressed_node);
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
                    self.last_chance_match_hybrid(decompressed_node, best);
                    self.internal
                        .mappings
                        .link(*decompressed_node.shallow(), *best.shallow());
                }
            } else if self.internal.mappings.is_src(&node)
                && self.internal.has_unmapped_src_children(&decompressed_node)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&node) {
                    let dst = self.internal.dst_arena.decompress_to(&dst);
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.last_chance_match_hybrid(decompressed_node, dst);
                    }
                }
            }
        }
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
            self.internal.last_chance_match_histogram(src, dst);
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
        if !(src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap()) {
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
}
