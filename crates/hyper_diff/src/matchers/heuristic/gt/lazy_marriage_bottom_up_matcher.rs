use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
    ShallowDecompressedTreeStore,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{HyperAST, NodeId, NodeStore, Tree, WithHashs, WithStats};
use num_traits::{cast, one};
use std::{fmt::Debug, marker::PhantomData};

pub struct LazyMarriageBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    MZs: MonoMappingStore = M,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
    _phantom: PhantomData<*const MZs>,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST,
    M,
    MZs,
    const SIZE_THRESHOLD: usize,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
    LazyMarriageBottomUpMatcher<
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
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: Tree + WithHashs + WithStats,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
    HAST::IdN: Clone + Eq + Debug,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    MZs: MonoMappingStore<Src = Dsrc::IdD, Dst = <Ddst as LazyDecompressed<M::Dst>>::IdD> + Default,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyPOBorrowSlice<HAST, Dsrc::IdD, M::Src>
        + ShallowDecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyPOBorrowSlice<HAST, Ddst::IdD, M::Dst>
        + ShallowDecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>
        + LazyDecompressed<M::Dst>,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: mapping,
            _phantom: PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal);
        matcher.internal
    }

    pub fn execute(internal: &mut Mapper<HAST, Dsrc, Ddst, M>) {
        assert_eq!(
            // TODO move it inside the arena ...
            internal.src_arena.root(),
            cast::<_, M::Src>(internal.src_arena.len()).unwrap() - one()
        );
        assert!(internal.src_arena.len() > 0);
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in internal.src_arena.iter_df_post::<false>() {
            let is_mapped = internal.mappings.is_src(&a);
            let a = internal.mapping.src_arena.decompress_to(&a);
            if !(is_mapped || !Self::src_has_children(internal, a)) {
                if let Some(best_dst) = Self::best_dst_candidate_lazy(internal, &a) {
                    if Self::best_src_candidate_lazy(internal, &best_dst) == Some(a) {
                        Self::last_chance_match(internal, a, best_dst);
                        internal.mappings.link(*a.shallow(), *best_dst.shallow());
                    }
                }
            }
        }
        // for root
        internal.mapping.mappings.link(
            internal.mapping.src_arena.root(),
            internal.mapping.dst_arena.root(),
        );
        let src = internal.src_arena.starter();
        let dst = internal.dst_arena.starter();
        Self::last_chance_match(internal, src, dst);
    }

    fn src_has_children(internal: &Mapper<HAST, Dsrc, Ddst, M>, src: Dsrc::IdD) -> bool {
        let o = internal.src_arena.original(&src);
        let r = internal.hyperast.node_store().resolve(&o).has_children();

        // TODO put it back
        // debug_assert_eq!(
        //     r,
        //     internal.src_arena.lld(&src) < *src.shallow(),
        //     "{:?} {:?}",
        //     internal.src_arena.lld(&src),
        //     src.to_usize()
        // );
        r
    }

    fn best_dst_candidate_lazy(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        src: &Dsrc::IdD,
    ) -> Option<Ddst::IdD> {
        let candidates = internal.get_dst_candidates_lazily(src);
        let mut best = None;
        let mut max: f64 = -1.;
        for cand in candidates {
            let sim = similarity_metrics::SimilarityMeasure::range(
                &internal.src_arena.descendants_range(src),
                &internal.dst_arena.descendants_range(&cand),
                &internal.mappings,
            )
            .chawathe();
            if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                max = sim;
                best = Some(cand);
            }
        }
        best
    }

    fn best_src_candidate_lazy(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        dst: &Ddst::IdD,
    ) -> Option<Dsrc::IdD> {
        let candidates = internal.get_src_candidates_lazily(dst);
        let mut best = None;
        let mut max: f64 = -1.;
        for cand in candidates {
            let sim = similarity_metrics::SimilarityMeasure::range(
                &internal.src_arena.descendants_range(&cand),
                &internal.dst_arena.descendants_range(dst),
                &internal.mappings,
            )
            .chawathe();
            if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                max = sim;
                best = Some(cand);
            }
        }
        best
    }

    pub(crate) fn last_chance_match(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        src: Dsrc::IdD,
        dst: Ddst::IdD,
    ) {
        Self::last_chance_match_zs(internal, src, dst);
        //internal.last_chance_match_histogram(src, dst);
    }

    pub(crate) fn last_chance_match_zs(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        src: Dsrc::IdD,
        dst: Ddst::IdD,
    ) {
        let mapping = &mut internal.mapping;
        let src_arena = &mut mapping.src_arena;
        let dst_arena = &mut mapping.dst_arena;
        let src_s = src_arena.descendants_count(&src);
        let dst_s = dst_arena.descendants_count(&dst);
        if !(src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap()) {
            return;
        }
        let src_offset;
        let dst_offset;
        let zs_mappings: MZs = {
            let src_arena = src_arena.slice_po(&src);
            src_offset = src - src_arena.root();
            let dst_arena = dst_arena.slice_po(&dst);
            dst_offset = dst - dst_arena.root();
            ZsMatcher::match_with(internal.hyperast, src_arena, dst_arena)
        };
        use num_traits::ToPrimitive;
        assert_eq!(
            mapping.src_arena.first_descendant(&src).to_usize(),
            src_offset.to_usize()
        );
        let mappings = &mut mapping.mappings;
        for (i, t) in zs_mappings.iter() {
            //remapping
            let src: Dsrc::IdD = src_offset + cast(i).unwrap();
            let dst: Ddst::IdD = dst_offset + cast(t).unwrap();
            // use it
            if !mappings.is_src(src.shallow()) && !mappings.is_dst(dst.shallow()) {
                let tsrc = internal
                    .hyperast
                    .resolve_type(&mapping.src_arena.original(&src));
                let tdst = internal
                    .hyperast
                    .resolve_type(&mapping.dst_arena.original(&dst));
                if tsrc == tdst {
                    mappings.link(*src.shallow(), *dst.shallow());
                }
            }
        }
    }
}
