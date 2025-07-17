use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedWithParent, LazyDecompressed, LazyDecompressedTreeStore,
    PostOrder, PostOrderIterable, Shallow,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::similarity_metrics;
use hyperast::PrimInt;
use hyperast::store::nodes::compo;
use hyperast::types::{HyperAST, NodeId, Tree, WithHashs, WithMetaData, WithStats};
use num_traits::ToPrimitive as _;
use std::fmt::Debug;

use super::leaf_count;

pub struct BottomUpMatcher<
    Dsrc,
    Ddst,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 4,
    const SIM_THRESHOLD_NUM: u64 = 6,
    const SIM_THRESHOLD_DEN: u64 = 10,
    const SIM_THRESHOLD2_NUM: u64 = 4,
    const SIM_THRESHOLD2_DEN: u64 = 10,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST,
    M,
    const SIZE_THRESHOLD: usize,   // = 1000,
    const SIM_THRESHOLD_NUM: u64,  // = 6,
    const SIM_THRESHOLD_DEN: u64,  // = 10,
    const SIM_THRESHOLD2_NUM: u64, // = 4,
    const SIM_THRESHOLD2_DEN: u64, // = 10,
>
    BottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
        SIM_THRESHOLD2_NUM,
        SIM_THRESHOLD2_DEN,
    >
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: Tree + WithHashs + WithStats,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
    HAST::IdN: Clone + Eq + Debug,
    Dsrc::IdD: PrimInt,
    Ddst::IdD: PrimInt,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
            WithMetaData<compo::MemberImportCount>,
    {
        let mut matcher = Self { internal: mapping };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal, leaf_count);
        matcher.internal
    }

    pub fn execute0(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        leaf_count: fn(HAST, HAST::IdN) -> usize,
    ) {
        let hyperast = internal.hyperast;
        for src in internal.src_arena.iter_df_post::<true>() {
            if internal.mappings.is_src(&src) {
                continue;
            }
            for dst in internal.dst_arena.iter_df_post::<true>() {
                if internal.mappings.is_dst(&dst) {
                    continue;
                }
                let mappings = &mut internal.mapping.mappings;
                let src_arena = &mut internal.mapping.src_arena;
                let dst_arena = &mut internal.mapping.dst_arena;
                let src = src_arena.decompress_to(&src);
                let dst = dst_arena.decompress_to(&dst);
                let leaves = leaf_count(hyperast, src_arena.original(&src));
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    pub fn execute(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        leaf_count: fn(HAST, HAST::IdN) -> usize,
    ) {
        let hyperast = internal.hyperast;
        for src in internal.src_arena.iter_df_post::<true>() {
            if internal.mappings.is_src(&src) {
                continue;
            }
            let mut dst_iter = PostIter::new(hyperast, &mut internal.mapping.dst_arena);
            while let Some(dst) = dst_iter.next_mappable(|dst|
                // we assume the whole subtree is already mapped
                internal.mapping.mappings.is_dst(dst.shallow()))
            {
                if internal.mapping.mappings.is_dst(dst.shallow()) {
                    continue;
                }
                let mappings = &mut internal.mapping.mappings;
                let src_arena = &mut internal.mapping.src_arena;
                let dst_arena = &*dst_iter.arena;
                let src = src_arena.decompress_to(&src);
                let leaves = leaf_count(hyperast, src_arena.original(&src));
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    pub fn execute1(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        leaf_count: fn(HAST, HAST::IdN) -> usize,
    ) {
        let hyperast = internal.hyperast;
        let mut src_iter = PostIter::new(hyperast, &mut internal.mapping.src_arena);
        while let Some(src) = src_iter.next_mappable(|src|
            // we assume the whole subtree is already mapped
            internal.mapping.mappings.is_src(src.shallow()))
        {
            if internal.mapping.mappings.is_src(src.shallow()) {
                continue;
            }
            let leaves = leaf_count(hyperast, src_iter.arena.original(&src));

            let mut dst_iter = PostIter::new(hyperast, &mut internal.mapping.dst_arena);
            while let Some(dst) = dst_iter.next_mappable(|dst|
                // we assume the whole subtree is already mapped
                internal.mapping.mappings.is_dst(dst.shallow()))
            {
                if internal.mapping.mappings.is_dst(dst.shallow()) {
                    continue;
                }
                let hyperast = internal.hyperast;
                let mappings = &mut internal.mapping.mappings;
                let src_arena = &*src_iter.arena;
                let dst_arena = &*dst_iter.arena;
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    fn inner(
        hyperast: HAST,
        mappings: &mut M,
        src_arena: &Dsrc,
        dst_arena: &Ddst,
        src: Dsrc::IdD,
        dst: Ddst::IdD,
        number_of_leaves: usize,
    ) -> bool
    where
        HAST: HyperAST + Copy,
    {
        let osrc = src_arena.original(&src);
        let tsrc = hyperast.resolve_type(&osrc);
        let odst = dst_arena.original(&dst);
        let tdst = hyperast.resolve_type(&odst);
        if tsrc == tdst {
            if !(src_arena.lld(&src) == src.to_shallow() || dst_arena.lld(&dst) == dst.to_shallow())
            {
                let sim = similarity_metrics::SimilarityMeasure::range(
                    &src_arena.descendants_range(&src),
                    &dst_arena.descendants_range(&dst),
                    &*mappings,
                )
                .chawathe();
                let cond1 = number_of_leaves > SIZE_THRESHOLD
                    && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                let cond2 = number_of_leaves <= SIZE_THRESHOLD
                    && sim >= SIM_THRESHOLD2_NUM as f64 / SIM_THRESHOLD2_DEN as f64;
                if cond1 || cond2 {
                    mappings.link(src.to_shallow(), dst.to_shallow());
                    return true;
                }
            }
        }
        false
    }
}

pub(super) struct PostIter<'a, HAST, D, IdS, IdD> {
    #[allow(unused)]
    stores: HAST,
    pub(super) arena: &'a mut D,
    to_traverse: Vec<IdD>,
    sibs: Vec<u16>,
    idd: IdD,
    down: bool,
    _phantom: std::marker::PhantomData<IdS>,
}

impl<'a, HAST, D, IdS> PostIter<'a, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    pub fn new(stores: HAST, arena: &'a mut D) -> Self {
        Self {
            stores,
            idd: arena.starter(),
            arena,
            to_traverse: Vec::new(),
            sibs: Vec::new(),
            down: true,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<HAST, D, IdS> Iterator for PostIter<'_, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    type Item = D::IdD;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_mappable(|_| false)
    }
}

impl<HAST, D, IdS> PostIter<'_, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    pub fn next_mappable(&mut self, skip: impl Fn(D::IdD) -> bool) -> Option<D::IdD> {
        loop {
            if self.down {
                if skip(self.idd) {
                    self.down = false;
                    continue;
                }
                let mut cs = self.arena.decompress_children(&self.idd);
                cs.reverse();
                let Some(idd) = cs.pop() else {
                    self.down = false;
                    return Some(self.idd);
                };
                self.to_traverse.push(self.idd);
                self.sibs.push(cs.len().to_u16().unwrap());
                self.idd = idd;
                self.to_traverse.extend(cs);
            } else {
                let Some(sib) = self.to_traverse.pop() else {
                    return None;
                };
                let sibs = self.sibs.last_mut().unwrap();
                if sibs == &0 {
                    self.sibs.pop();
                    return Some(sib);
                }
                *sibs -= 1;
                self.down = true;
                self.idd = sib;
            }
        }
    }
}
