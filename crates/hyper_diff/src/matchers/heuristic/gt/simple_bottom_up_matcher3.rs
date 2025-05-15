use super::bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::SimpleZsTree as ZsTree;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::Decompressible;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use hyperast::PrimInt;
use num_traits::{cast, one};
use std::fmt::Debug;
use hyperast::position::tags::TopDownNoSpace;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct SimpleBottomUpMatcher3<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
        Dsrc,
        Ddst,
        HAST: HyperAST,
        M: MonoMappingStore,
        const SIZE_THRESHOLD: usize,  // = 1000,
        const SIM_THRESHOLD_NUM: u64, // = 1,
        const SIM_THRESHOLD_DEN: u64, // = 2,
    > Into<BottomUpMatcher<Dsrc, Ddst, HAST, M>>
    for SimpleBottomUpMatcher3<
        Dsrc,
        Ddst,
        HAST,
        M,
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
        const SIZE_THRESHOLD: usize,
        const SIM_THRESHOLD_NUM: u64,
        const SIM_THRESHOLD_DEN: u64,
    >
    SimpleBottomUpMatcher3<
        Dsrc,
        Ddst,
        HAST,
        M,
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
    pub fn new(stores: HAST, src_arena: Dsrc, dst_arena: Ddst, mappings: M) -> Self {
        Self {
            internal: BottomUpMatcher {
                stores,
                src_arena,
                dst_arena,
                mappings,
            },
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

    pub fn matchh(store: HAST, src: &'a HAST::IdN, dst: &'a HAST::IdN, mappings: M) -> Self {
        let mut matcher = Self::new(
            store,
            Dsrc::decompress(store, src),
            Ddst::decompress(store, dst),
            mappings,
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
            let path = self.internal.src_arena.path::<usize>(&self.internal.src_arena.root(), &t);
            dbg!(path);
            // if self.internal.src_arena.parent(&t).is_none() {
            //     self.internal
            //         .mappings
            //         .link(t, self.internal.dst_arena.root());
            //     self.last_chance_match(&t, &self.internal.dst_arena.root());
            //     break;
            // }
            if !(self.internal.mappings.is_src(&t) || !self.src_has_children(t)) {
                let candidates = self.internal.get_dst_candidates(&t);
                let mut best = None;
                let mut max_sim = -1f64;
                for candidate in candidates {
                    let sim = similarity_metrics::chawathe_similarity(
                        &self.internal.src_arena.descendants(&t),
                        &self.internal.dst_arena.descendants(&candidate),
                        &self.internal.mappings,
                    );
                    let threshold = SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                    if sim > max_sim && sim >= threshold {
                        max_sim = sim;
                        best = Some(candidate);
                    }
                }
                if let Some(best) = best {
                    self.last_chance_match(&t, &best);
                    self.internal.mappings.link(t, best);
                }
            } else if self.internal.mappings.is_src(&t)
                && self.internal.are_srcs_unmapped(&t)
                && let Some(dst) = self.internal.mappings.get_dst(&t)
                && self.internal.are_dsts_unmapped(&dst)
            {
                self.last_chance_match(&t, &dst);
            }
        }

        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        self.last_chance_match(
            &self.internal.src_arena.root(),
            &self.internal.dst_arena.root(),
        );
    }

    fn last_chance_match(&mut self, src: &M::Src, dst: &M::Dst) {
        self.internal.last_chance_match_histogram(src, dst);
    }
    
    fn last_chance_match_hybrid(&mut self, src: &M::Src, dst: &M::Dst) {
        if self.internal.src_arena.descendants_count(&src) < SIZE_THRESHOLD
        && self.internal.dst_arena.descendants_count(&dst) < SIZE_THRESHOLD {
            todo!()
        } else {
            self.internal.last_chance_match_histogram(src, dst);
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
