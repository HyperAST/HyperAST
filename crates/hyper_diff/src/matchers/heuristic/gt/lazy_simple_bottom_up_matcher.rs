use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
    ShallowDecompressedTreeStore,
};
use crate::matchers::{
    heuristic::gt::lazy_bottom_up_matcher::BottomUpMatcher, mapping_store::MonoMappingStore,
    similarity_metrics,
};
use hyperast::{
    PrimInt,
    types::{HyperAST, NodeId, WithHashs},
};
use std::{fmt::Debug, marker::PhantomData};

pub struct LazySimpleBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    MZs: MonoMappingStore = M,
    const SIMILARITY_THRESHOLD_NUM: u64 = 1,
    const SIMILARITY_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
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
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    MZs: MonoMappingStore<Src = Dsrc::IdD, Dst = <Ddst as LazyDecompressed<M::Dst>>::IdD> + Default,
    const SIMILARITY_THRESHOLD_NUM: u64, // 1
    const SIMILARITY_THRESHOLD_DEN: u64, // 2
>
    LazySimpleBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        MZs,
        SIMILARITY_THRESHOLD_NUM,
        SIMILARITY_THRESHOLD_DEN,
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

    pub fn execute(&mut self) {
        assert!(self.internal.src_arena.len() > 0);
        let similarity_threshold: f64 =
            SIMILARITY_THRESHOLD_NUM as f64 / SIMILARITY_THRESHOLD_DEN as f64;

        for node in self.internal.src_arena.iter_df_post::<true>() {
            let decompressed_node = self.internal.src_arena.decompress_to(&node);
            if self.internal.src_arena.parent(&decompressed_node).is_none() {
                self.internal.mappings.link(
                    self.internal.src_arena.root(), // <- this is node
                    self.internal.dst_arena.root(),
                );
                self.internal.last_chance_match_histogram(
                    self.internal.src_arena.starter(), // <- this is node
                    self.internal.dst_arena.starter(),
                );
                break;
            } else if !self.internal.mappings.is_src(&node)
                && self.internal.src_has_children(decompressed_node)
            {
                let candidates = self.internal.get_dst_candidates_lazily(&decompressed_node);
                let mut best_candidate = None;
                let mut max_similarity: f64 = -1.;

                for candidate in candidates {
                    let t_descendents = self
                        .internal
                        .src_arena
                        .descendants_range(&decompressed_node);
                    let candidate_descendents =
                        self.internal.dst_arena.descendants_range(&candidate);
                    let similarity = similarity_metrics::SimilarityMeasure::range(
                        &t_descendents,
                        &candidate_descendents,
                        &self.internal.mappings,
                    )
                    .chawathe();

                    if similarity > max_similarity && similarity >= similarity_threshold {
                        max_similarity = similarity;
                        best_candidate = Some(candidate);
                    }
                }

                if let Some(best_candidate) = best_candidate {
                    self.internal
                        .last_chance_match_histogram(decompressed_node, best_candidate);
                    self.internal
                        .mappings
                        .link(*decompressed_node.shallow(), *best_candidate.shallow());
                }
            } else if self.internal.mappings.is_src(&node)
                && self.internal.has_unmapped_src_children(&decompressed_node)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&node) {
                    let dst = self.internal.dst_arena.decompress_to(&dst);
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.internal
                            .last_chance_match_histogram(decompressed_node, dst);
                    }
                }
            }
        }
    }
}
