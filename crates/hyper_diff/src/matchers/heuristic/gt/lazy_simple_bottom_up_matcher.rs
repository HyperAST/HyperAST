use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
    LazyDecompressedTreeStore, PostOrder, PostOrderIterable, Shallow, ShallowDecompressedTreeStore,
};
use crate::matchers::Mapper;
use crate::matchers::{mapping_store::MonoMappingStore, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{HyperAST, NodeId, WithHashs};
use std::fmt::Debug;

pub struct LazySimpleBottomUpMatcher<
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
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + ShallowDecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>
        + LazyDecompressed<M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + ShallowDecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>
        + LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    const SIMILARITY_THRESHOLD_NUM: u64, // 1
    const SIMILARITY_THRESHOLD_DEN: u64, // 2
> LazySimpleBottomUpMatcher<Dsrc, Ddst, HAST, M, SIMILARITY_THRESHOLD_NUM, SIMILARITY_THRESHOLD_DEN>
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

    pub fn execute(&mut self) {
        assert!(self.internal.src_arena.len() > 0);
        for node in self.internal.src_arena.iter_df_post::<false>() {
            let decomp = self.internal.src_arena.decompress_to(&node);
            if !self.internal.mappings.is_src(&node) && self.internal.src_has_children_lazy(decomp)
            {
                let candidates = self.internal.get_dst_candidates_lazily(&decomp);
                let mut best_candidate = None;
                let mut max_similarity: f64 = -1.;

                for candidate in candidates {
                    let t_descendents = (self.internal.src_arena).descendants_range(&decomp);
                    let candidate_descendents =
                        self.internal.dst_arena.descendants_range(&candidate);
                    let similarity = similarity_metrics::SimilarityMeasure::range(
                        &t_descendents,
                        &candidate_descendents,
                        &self.internal.mappings,
                    )
                    .chawathe();

                    if similarity
                        >= SIMILARITY_THRESHOLD_NUM as f64 / SIMILARITY_THRESHOLD_DEN as f64
                        && similarity > max_similarity
                    {
                        max_similarity = similarity;
                        best_candidate = Some(candidate);
                    }
                }

                if let Some(best_candidate) = best_candidate {
                    self.internal
                        .last_chance_match_histogram_lazy(decomp, best_candidate);
                    self.internal
                        .mappings
                        .link(*decomp.shallow(), *best_candidate.shallow());
                }
            } else if self.internal.mappings.is_src(&node)
                && self.internal.has_unmapped_src_children_lazy(&decomp)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&node) {
                    let dst = self.internal.dst_arena.decompress_to(&dst);
                    if self.internal.has_unmapped_dst_children_lazy(&dst) {
                        self.internal.last_chance_match_histogram_lazy(decomp, dst);
                    }
                }
            }
        }

        self.internal.mapping.mappings.link(
            self.internal.mapping.src_arena.root(),
            self.internal.mapping.dst_arena.root(),
        );
        self.internal.last_chance_match_histogram_lazy(
            self.internal.src_arena.starter(),
            self.internal.dst_arena.starter(),
        );
    }
}
