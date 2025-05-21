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

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct SimpleMarriageBottomUpMatcher<
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
    for SimpleMarriageBottomUpMatcher<
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
    SimpleMarriageBottomUpMatcher<
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
        assert_eq!(
            // TODO move it inside the arena ...
            self.internal.src_arena.root(),
            cast::<_, M::Src>(self.internal.src_arena.len()).unwrap() - one()
        );
        assert!(self.internal.src_arena.len() > 0);
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in self.internal.src_arena.iter_df_post::<true>() {
            if self.internal.src_arena.parent(&a).is_none() {
                // TODO remove and flip const param of iter_df_post
                break;
            } else if !(self.internal.mappings.is_src(&a) || !self.src_has_children(a)) {
                if let Some(best_dst) = self.best_dst_candidate(&a) {
                    if self.best_src_candidate(&best_dst) == Some(a) {
                        //println!("chosen link: {:?} -> {:?}", &a_orig, &best_dst_orig);
                        self.internal.last_chance_match_histogram(&a, &best_dst);
                        self.internal.mappings.link(a, best_dst);
                    }
                }
            } else if self.internal.mappings.is_src(&a)
                && self.has_unmapped_src_children(&a)
                && self.has_unmapped_dst_children(
                    &self
                        .internal
                        .mappings
                        .get_dst(&a)
                        .expect("No dst found for src"),
                )
            {
                if let Some(dst) = self.internal.mappings.get_dst(&a) {
                    self.internal.last_chance_match_histogram(&a, &dst);
                }
            }
        }
        // for root
        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        self.internal.last_chance_match_histogram(
            &self.internal.src_arena.root(),
            &self.internal.dst_arena.root(),
        );
    }

    fn has_unmapped_src_children(&self, src: &M::Src) -> bool {
        for a in self.internal.src_arena.descendants(src) {
            if !self.internal.mappings.is_src(&a) {
                return true;
            }
        }
        return false;
    }

    fn has_unmapped_dst_children(&self, dst: &M::Dst) -> bool {
        for a in self.internal.dst_arena.descendants(dst) {
            if !self.internal.mappings.is_dst(&a) {
                return true;
            }
        }
        return false;
    }

    fn best_dst_candidate(&self, src: &M::Src) -> Option<M::Dst> {
        let candidates = self.internal.get_dst_candidates(src);
        let mut best = None;
        let mut max: f64 = -1.;
        for cand in candidates {
            let sim = similarity_metrics::SimilarityMeasure::range(
                &self.internal.src_arena.descendants_range(src),
                &self.internal.dst_arena.descendants_range(&cand),
                &self.internal.mappings,
            )
            .chawathe();
            if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                max = sim;
                best = Some(cand);
            }
        }
        best
    }

    fn best_src_candidate(&self, dst: &M::Dst) -> Option<M::Src> {
        let candidates = self.internal.get_src_candidates(dst);
        let mut best = None;
        let mut max: f64 = -1.;
        for cand in candidates {
            let sim = similarity_metrics::SimilarityMeasure::range(
                &self.internal.src_arena.descendants_range(&cand),
                &self.internal.dst_arena.descendants_range(dst),
                &self.internal.mappings,
            )
            .chawathe();
            if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                max = sim;
                best = Some(cand);
            }
        }
        best
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
