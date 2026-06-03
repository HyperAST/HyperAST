use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, PostOrder,
    PostOrderIterable,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::similarity_metrics;
use hyperast::PrimInt;
use hyperast::types::{HyperAST, NodeId, NodeStore, Tree, WithHashs};
use std::collections::HashMap;
use std::fmt::Debug;

pub struct XYBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
    Dsrc: DecompressedTreeStore<HAST, M::Src>
        + DecompressedWithParent<HAST, M::Src>
        + PostOrder<HAST, M::Src>
        + PostOrderIterable<HAST, M::Src>
        + ContiguousDescendants<HAST, M::Src>, // descendants_range
    Ddst: DecompressedTreeStore<HAST, M::Dst>
        + DecompressedWithParent<HAST, M::Dst>
        + PostOrder<HAST, M::Dst>
        + PostOrderIterable<HAST, M::Dst>
        + ContiguousDescendants<HAST, M::Dst>, // descendants_range
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
> XYBottomUpMatcher<Dsrc, Ddst, HAST, M, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
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

    pub fn execute(&mut self) {
        for a in self.internal.src_arena.iter_df_post::<false>() {
            if !(self.internal.mappings.is_src(&a) || !self.src_has_children(a)) {
                let candidates = self.internal.get_dst_candidates(&a);
                let mut best = None;
                let mut max: f64 = -1.;
                for cand in candidates {
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &self.internal.src_arena.descendants_range(&a),
                        &self.internal.dst_arena.descendants_range(&cand),
                        &self.internal.mappings,
                    )
                    .jaccard();
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = Some(cand);
                    }
                }

                if let Some(best) = best {
                    self.last_chance_match(a, best);
                    self.internal.mappings.link(a, best);
                }
            }
        }
        // for root
        self.internal.mapping.mappings.link(
            self.internal.mapping.src_arena.root(),
            self.internal.mapping.dst_arena.root(),
        );
        self.last_chance_match(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
    }

    fn src_has_children(&mut self, src: M::Src) -> bool {
        use num_traits::ToPrimitive;
        let r = self
            .internal
            .hyperast
            .node_store()
            .resolve(&self.internal.src_arena.original(&src))
            .has_children();
        debug_assert_eq!(
            r,
            self.internal.src_arena.lld(&src) < src,
            "{:?} {:?}",
            self.internal.src_arena.lld(&src),
            src.to_usize()
        );
        r
    }
    fn last_chance_match(&mut self, src: M::Src, dst: M::Dst) {
        let mut src_types: HashMap<_, Vec<M::Src>> = HashMap::new();
        let mut dst_types: HashMap<_, Vec<M::Dst>> = HashMap::new();

        for src_child in self.internal.src_arena.children(&src) {
            let original = self.internal.src_arena.original(&src_child);
            let src_type = self.internal.hyperast.resolve_type(&original);
            src_types.entry(src_type).or_default().push(src_child);
        }

        for dst_child in self.internal.dst_arena.children(&dst) {
            let original = self.internal.dst_arena.original(&dst_child);
            let dst_type = self.internal.hyperast.resolve_type(&original);
            dst_types.entry(dst_type).or_default().push(dst_child);
        }

        for (src_type, src_list) in src_types.iter() {
            // TODO same thing use an Option instead of a Vec
            if src_list.len() == 1 {
                if let Some(dst_list) = dst_types.get(src_type) {
                    if dst_list.len() == 1 {
                        if !self.internal.mappings.is_src(&src_list[0])
                            && !self.internal.mappings.is_dst(&dst_list[0])
                        {
                            self.internal.mappings.link(src_list[0], dst_list[0]);
                        }
                    }
                }
            }
        }
    }
}
