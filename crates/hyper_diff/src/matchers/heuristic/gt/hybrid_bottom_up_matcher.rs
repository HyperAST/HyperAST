use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use num_traits::cast;
use std::fmt::Debug;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct HybridBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    MZs: MonoMappingStore = M,
    const SIZE_THRESHOLD: usize = 100,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
    _phantom: std::marker::PhantomData<*const MZs>,
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
    MZs: MonoMappingStore<Src = M::Src, Dst = M::Dst> + Default,
    const SIZE_THRESHOLD: usize,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
    HybridBottomUpMatcher<
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
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
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
        for t in self.internal.mapping.src_arena.iter_df_post::<true>() {
            // let path = self.internal.src_arena.path::<usize>(&self.internal.src_arena.root(), &t);
            // dbg!(path);
            if self.internal.mapping.src_arena.parent(&t).is_none() {
                self.internal.mapping.mappings.link(
                    self.internal.mapping.src_arena.root(),
                    self.internal.mapping.dst_arena.root(),
                );
                self.last_chance_match_hybrid(
                    &self.internal.mapping.src_arena.root(),
                    &self.internal.mapping.dst_arena.root(),
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
                    self.last_chance_match_hybrid(&t, &best);
                    self.internal.mappings.link(t, best);
                }
            } else if self.internal.mappings.is_src(&t)
                && self.internal.has_unmapped_src_children(&t)
            {
                if let Some(dst) = self.internal.mappings.get_dst(&t) {
                    if self.internal.has_unmapped_dst_children(&dst) {
                        self.last_chance_match_hybrid(&t, &dst);
                    }
                }
            }
        }
    }

    fn last_chance_match_hybrid(&mut self, src: &M::Src, dst: &M::Dst) {
        if self.internal.src_arena.descendants_count(&src) < SIZE_THRESHOLD
            && self.internal.dst_arena.descendants_count(&dst) < SIZE_THRESHOLD
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

        let mappings: MZs = ZsMatcher::match_with(self.internal.hyperast, src_arena, dst_arena);

        for (i, t) in mappings.iter() {
            //remapping
            let src: M::Src = src_offset + cast(i).unwrap();
            let dst: M::Dst = dst_offset + cast(t).unwrap();
            // use it
            if !self.internal.mappings.is_src(&src) && !self.internal.mappings.is_dst(&dst) {
                let tsrc = self
                    .internal
                    .hyperast
                    .resolve_type(&self.internal.src_arena.original(&src));
                let tdst = self
                    .internal
                    .hyperast
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
            .hyperast
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
