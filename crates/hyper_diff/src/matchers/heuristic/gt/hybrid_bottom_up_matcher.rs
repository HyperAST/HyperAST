use super::bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::{ShallowDecompressedTreeStore, SimpleZsTree as ZsTree};
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::matchers::Decompressible;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::position::tags::TopDownNoSpace;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use num_traits::{cast, one};
use std::fmt::Debug;
use std::time::Instant;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct HybridBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
    max_size: usize,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
    Dsrc,
    Ddst,
    HAST: HyperAST,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64, // = 1,
    const SIM_THRESHOLD_DEN: u64, // = 2,
> Into<BottomUpMatcher<Dsrc, Ddst, HAST, M>>
for HybridBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M,
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
    Dsrc: DecompressedTreeStore<HAST, M::Src>
    + DecompressedWithParent<HAST, M::Src>
    + PostOrder<HAST, M::Src>
    + PostOrderIterable<HAST, M::Src>
    + DecompressedFrom<HAST, Out = Dsrc>
    + ContiguousDescendants<HAST, M::Src>
    + POBorrowSlice<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst>
    + DecompressedWithParent<HAST, M::Dst>
    + PostOrder<HAST, M::Dst>
    + PostOrderIterable<HAST, M::Dst>
    + DecompressedFrom<HAST, Out = Ddst>
    + ContiguousDescendants<HAST, M::Dst>
    + POBorrowSlice<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
> HybridBottomUpMatcher<Dsrc, Ddst, HAST, M, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
        M::Src: PrimInt,
        M::Dst: PrimInt,
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
            max_size
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

    pub fn matchh(store: HAST, src: &'a HAST::IdN, dst: &'a HAST::IdN, mappings: M, max_size: usize) -> Self {
        let mut matcher = Self::new(
            store,
            Dsrc::decompress(store, src),
            Ddst::decompress(store, dst),
            mappings,
            max_size
        );
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute(&mut matcher);
        matcher
    }

    pub fn execute<'b>(&mut self) {
        for t in self.internal.src_arena.iter_df_post::<true>() {
            // let path = self.internal.src_arena.path::<usize>(&self.internal.src_arena.root(), &t);
            // dbg!(path);
            if self.internal.src_arena.parent(&t).is_none() {
                self.internal.mappings.link(
                    self.internal.src_arena.root(),
                    self.internal.dst_arena.root(),
                );
                self.last_chance_match_hybrid(
                    &self.internal.src_arena.root(),
                    &self.internal.dst_arena.root(),
                );
                break;
            } else if !(self.internal.mappings.is_src(&t) || !self.src_has_children(t)) {
                let candidates = self.internal.get_dst_candidates(&t);
                let mut best = None;
                let mut max_sim = -1f64;
                for candidate in candidates {
                    let t_descendents = &self.internal.src_arena.descendants(&t);
                    let candidate_descendents = &self.internal.dst_arena.descendants(&candidate);
                    let sim = similarity_metrics::chawathe_similarity(
                        t_descendents,
                        candidate_descendents,
                        &self.internal.mappings,
                    );
                    let threshold = 1f64 / (1f64 + ((candidate_descendents.len() + t_descendents.len()) as f64).ln());
                    // SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                    if sim > max_sim && sim >= threshold {
                        max_sim = sim;
                        best = Some(candidate);
                    }
                }
                if let Some(best) = best {
                    self.last_chance_match_hybrid(&t, &best);
                    self.internal.mappings.link(t, best);
                }
            } else if self.internal.mappings.is_src(&t) && self.internal.has_unmapped_src_children(&t) {
                if let Some(dst) = self.internal.mappings.get_dst(&t) {
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.last_chance_match_hybrid(&t, &dst);
                    }
                }
            }
        }
    }

    fn last_chance_match_hybrid(&mut self, src: &M::Src, dst: &M::Dst) {
        if self.internal.src_arena.descendants_count(&src) < self.max_size
            && self.internal.dst_arena.descendants_count(&dst) < self.max_size
        {
            self.last_chance_match_optimal(src, dst);
        } else {
            self.internal.last_chance_match_histogram(src, dst);
        }
    }

    fn last_chance_match_optimal(&mut self, src: &M::Src, dst: &M::Dst) {
        let src_arena = self.internal.src_arena.slice_po(&src);
        let dst_arena = self.internal.dst_arena.slice_po(&dst);

        let src_offset: M::Src = *src - src_arena.root();
        let dst_offset: M::Dst = self.internal.dst_arena.first_descendant(&dst);

        let mappings: M = ZsMatcher::match_with(self.internal.stores, src_arena, dst_arena);

        for (i, t) in mappings.iter() {
            //remapping
            let src: M::Src = src_offset + cast(i).unwrap();
            let dst: M::Dst = dst_offset + cast(t).unwrap();
            // use it
            if !self.internal.mappings.is_src(&src) && !self.internal.mappings.is_dst(&dst) {
                let tsrc = self
                    .internal
                    .stores
                    .resolve_type(&self.internal.src_arena.original(&src));
                let tdst = self
                    .internal
                    .stores
                    .resolve_type(&self.internal.dst_arena.original(&dst));
                if tsrc == tdst {
                    self.internal.mappings.link(src, dst);
                }
            }
        }
    }

    fn src_has_children(&mut self, src: M::Src) -> bool {
        use num_traits::ToPrimitive;
        let r = self
            .internal
            .stores
            .node_store()
            .resolve(&self.internal.src_arena.original(&src))
            .has_children();
        assert_eq!(
            r,
            self.internal.src_arena.lld(&src) < src,
            "{:?} {:?}",
            self.internal.src_arena.lld(&src),
            src.to_usize()
        );
        r
    }
}
