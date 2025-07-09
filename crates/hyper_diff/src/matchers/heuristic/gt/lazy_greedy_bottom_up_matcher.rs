//! makes greedy_bottom_up_matcher lazy
//! - [ ] first make post order iterator lazy
//!
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
    ShallowDecompressedTreeStore,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{
    DecompressedSubtree, HyperAST, NodeId, NodeStore, Tree, WithHashs, WithStats,
};
use num_traits::{cast, one};
use std::{fmt::Debug, marker::PhantomData};

pub struct GreedyBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
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
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + DecompressedSubtree<HAST>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyPOBorrowSlice<HAST, Dsrc::IdD, M::Src>
        + ShallowDecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + DecompressedSubtree<HAST>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyPOBorrowSlice<HAST, Ddst::IdD, M::Dst>
        + ShallowDecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    MZs: MonoMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default,
    const SIZE_THRESHOLD: usize,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
    GreedyBottomUpMatcher<
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
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        M: Default,
    {
        let mut matcher = Self {
            internal: mapping,
            _phantom: PhantomData,
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

    pub fn execute<'b>(&mut self)
    where
        M: Default,
    {
        assert_eq!(
            // TODO move it inside the arena ...
            self.internal.src_arena.root(),
            cast::<_, M::Src>(self.internal.src_arena.len()).unwrap() - one()
        );
        assert!(self.internal.src_arena.len() > 0);
        // println!("mappings={}", self.internal.mappings.len());
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in self.internal.src_arena.iter_df_post::<false>() {
            // if self.internal.src_arena.parent(&a).is_none() {
            //     break;
            // }
            if self.internal.mappings.is_src(&a) {
                continue;
            }
            let a = self.internal.src_arena.decompress_to(&a);
            if self.src_has_children(a) {
                let candidates = self.internal.get_dst_candidates_lazily(&a);
                let mut best = None;
                let mut max: f64 = -1.;
                for cand in candidates {
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &self.internal.src_arena.descendants_range(&a),
                        &self.internal.dst_arena.descendants_range(&cand),
                        &self.internal.mappings,
                    )
                    .dice();
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = Some(cand);
                    }
                }

                if let Some(best) = best {
                    self.last_chance_match_zs(a, best);
                    self.internal.mappings.link(*a.shallow(), *best.shallow());
                }
            }
        }
        // for root
        self.internal.mapping.mappings.link(
            self.internal.mapping.src_arena.root(),
            self.internal.mapping.dst_arena.root(),
        );
        self.last_chance_match_zs(
            self.internal.src_arena.starter(),
            self.internal.dst_arena.starter(),
        );
        // println!("nodes:{}", c);
        // println!("nodes:{}", c2);
    }

    fn src_has_children(&mut self, src: Dsrc::IdD) -> bool {
        let o = self.internal.src_arena.original(&src);
        let r = self
            .internal
            .hyperast
            .node_store()
            .resolve(&o)
            .has_children();
        use num_traits::ToPrimitive;
        debug_assert_eq!(
            r,
            self.internal.src_arena.lld(&src) < *src.shallow(),
            "{:?} {:?}",
            self.internal.src_arena.lld(&src),
            src.to_usize()
        );
        r
    }

    pub(crate) fn last_chance_match_zs(&mut self, src: Dsrc::IdD, dst: Ddst::IdD)
    where
        M: Default,
    {
        // WIP https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
        let src_s = self.internal.src_arena.descendants_count(&src);
        let dst_s = self.internal.dst_arena.descendants_count(&dst);
        if !(src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap()) {
            return;
        }
        let stores = self.internal.hyperast;
        let src_offset;
        let dst_offset;
        let mappings: MZs = {
            let src_arena = self.internal.mapping.src_arena.slice_po(&src);
            src_offset = src - src_arena.root();
            let dst_arena = self.internal.mapping.dst_arena.slice_po(&dst);
            dst_offset = dst - dst_arena.root();
            ZsMatcher::match_with(stores, src_arena, dst_arena)
        };
        use num_traits::ToPrimitive;
        assert_eq!(
            self.internal.src_arena.first_descendant(&src).to_usize(),
            src_offset.to_usize()
        );
        for (i, t) in mappings.iter() {
            //remapping
            let src: Dsrc::IdD = src_offset + cast(i).unwrap();
            let dst: Ddst::IdD = dst_offset + cast(t).unwrap();
            // use it
            if !self.internal.mappings.is_src(src.shallow())
                && !self.internal.mappings.is_dst(dst.shallow())
            {
                let tsrc = self
                    .internal
                    .hyperast
                    .resolve_type(&self.internal.src_arena.original(&src));
                let tdst = self
                    .internal
                    .hyperast
                    .resolve_type(&self.internal.dst_arena.original(&dst));
                if tsrc == tdst {
                    self.internal.mappings.link(*src.shallow(), *dst.shallow());
                }
            }
        }
    }
}
