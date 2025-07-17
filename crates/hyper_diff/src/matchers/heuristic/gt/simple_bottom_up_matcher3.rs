use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, PostOrder,
    PostOrderIterable,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Mapper, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{HyperAST, NodeId, WithHashs};
use std::fmt::Debug;

pub struct SimpleBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIMILARITY_THRESHOLD_NUM: u64 = 1,
    const SIMILARITY_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
    'a,
    Dsrc: DecompressedTreeStore<HAST, M::Src>
        + DecompressedWithParent<HAST, M::Src>
        + PostOrder<HAST, M::Src>
        + PostOrderIterable<HAST, M::Src>
        + ContiguousDescendants<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst>
        + DecompressedWithParent<HAST, M::Dst>
        + PostOrder<HAST, M::Dst>
        + PostOrderIterable<HAST, M::Dst>
        + ContiguousDescendants<HAST, M::Dst>,
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
        let mut matcher = Self { internal: mapping };
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
        assert!(self.internal.src_arena.len() > 0);
        let similarity_threshold: f64 =
            SIMILARITY_THRESHOLD_NUM as f64 / SIMILARITY_THRESHOLD_DEN as f64;

        for node in self.internal.src_arena.iter_df_post::<false>() {
            if !self.internal.mappings.is_src(&node) && self.internal.src_has_children(node) {
                let candidates = self.internal.get_dst_candidates(&node);
                let mut best = None;
                let mut max_similarity: f64 = -1.;

                // Can be used to calculate an appropriate threshold. In Gumtree this is done when no threshold is provided.
                // let tree_size = self.internal.src_arena.descendants_count(&tree);

                for candidate in candidates {
                    // In gumtree implementation they check if Simliarity_Threshold is set, otherwise they compute a fitting value
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
            } else if self.internal.mappings.is_src(&node)
                && self.internal.has_unmapped_src_children(&node)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&node) {
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.internal.last_chance_match_histogram(&node, &dst);
                    }
                }
            }
        }

        self.internal.mapping.mappings.link(
            self.internal.mapping.src_arena.root(),
            self.internal.mapping.dst_arena.root(),
        );
        self.internal.last_chance_match_histogram(
            &self.internal.src_arena.root(),
            &self.internal.dst_arena.root(),
        );
    }
}
