use super::bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::similarity_metrics;
use hyperast::PrimInt;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, WithHashs};
use std::fmt::Debug;

pub struct SimpleBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIMILARITY_THRESHOLD_NUM: u64 = 1,
    const SIMILARITY_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
}

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
    const SIMILARITY_THRESHOLD_NUM: u64, // 1
    const SIMILARITY_THRESHOLD_DEN: u64, // 2
> SimpleBottomUpMatcher<Dsrc, Ddst, HAST, M, SIMILARITY_THRESHOLD_NUM, SIMILARITY_THRESHOLD_DEN>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
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
        assert!(self.internal.src_arena.len() > 0);
        let similarity_threshold: f64 =
            SIMILARITY_THRESHOLD_NUM as f64 / SIMILARITY_THRESHOLD_DEN as f64;

        for node in self.internal.src_arena.iter_df_post::<true>() {
            // Check if 'node' is the root (thus has no parents)
            if self.internal.src_arena.parent(&node).is_none() {
                self.internal.mappings.link(
                    self.internal.src_arena.root(), // <- this is node
                    self.internal.dst_arena.root(),
                );
                self.internal.last_chance_match_histogram(
                    &self.internal.src_arena.root(), // <- this is node
                    &self.internal.dst_arena.root(),
                );
                break;
            } else if !(self.internal.mappings.is_src(&node)
                || !self.internal.src_has_children(node))
            {
                let candidates = self.internal.get_dst_candidates(&node);
                let mut best = None;
                let mut max_similarity: f64 = -1.;

                // Can be used to calculate an appropriate threshold. In Gumtree this is done when no threshold is provided.
                // let tree_size = self.internal.src_arena.descendants_count(&tree);

                for candidate in candidates {
                    // In gumtree implementation they check if Simliarity_THreshold is set, otherwise they compute a fitting value
                    // But here we assume threshold is always set.
                    let similarity = similarity_metrics::chawathe_similarity(
                        &self.internal.src_arena.descendants(&node),
                        &self.internal.dst_arena.descendants(&candidate),
                        &self.internal.mappings,
                    );

                    if similarity > max_similarity && similarity >= similarity_threshold {
                        max_similarity = similarity;
                        best = Some(candidate);
                    }
                }

                if let Some(best_candidate) = best {
                    self.internal
                        .last_chance_match_histogram(&node, &best_candidate);
                    self.internal.mappings.link(node, best_candidate);
                }
            } else if self.internal.mappings.is_src(&node) && self.internal.are_srcs_unmapped(&node)
            // Check if there are unmapped children in src or dst
            {
                if let Some(dst) = self.internal.mappings.get_dst(&node) {
                    if self.internal.are_dsts_unmapped(&dst) {
                        self.internal.last_chance_match_histogram(&node, &dst);
                    }
                }
            }
        }
    }
}
