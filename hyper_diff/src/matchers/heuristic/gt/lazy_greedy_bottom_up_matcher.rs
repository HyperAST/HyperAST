//! makes greedy_bottom_up_matcher lazy
//! - [ ] first make post order iterator lazy
//!
use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, PrimInt};

use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, PostOrderKeyRoots,
    Shallow, ShallowDecompressedTreeStore,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyper_ast::types::{
    DecompressedSubtree, HyperAST, LabelStore, NodeStore, SlicedLabel, Tree, WithHashs, WithStats,
};

use super::lazy_bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::SimpleZsTree as ZsTree;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct GreedyBottomUpMatcher<
    'a,
    Dsrc,
    Ddst,
    T: 'a + Tree + WithHashs,
    S,
    LS: LabelStore<SlicedLabel, I = T::Label>,
    M: MonoMappingStore,
    MZs: MonoMappingStore = M,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    label_store: &'a LS,
    internal: BottomUpMatcher<'a, Dsrc, Ddst, T, S, M>,
    _phantom: PhantomData<*const MZs>,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
        'a,
        Dsrc,
        Ddst,
        T: 'a + Tree + WithHashs,
        S,
        LS: LabelStore<SlicedLabel, I = T::Label>,
        M: MonoMappingStore,
        MZs: MonoMappingStore<Src = M::Src, Dst = M::Dst>,
        const SIZE_THRESHOLD: usize,
        const SIM_THRESHOLD_NUM: u64,
        const SIM_THRESHOLD_DEN: u64,
    > Into<BottomUpMatcher<'a, Dsrc, Ddst, T, S, M>>
    for GreedyBottomUpMatcher<
        'a,
        Dsrc,
        Ddst,
        T,
        S,
        LS,
        M,
        MZs,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    fn into(self) -> BottomUpMatcher<'a, Dsrc, Ddst, T, S, M> {
        self.internal
    }
}
impl<
        'a,
        Dsrc,
        Ddst,
        T: 'a + Tree + WithHashs,
        S,
        LS: LabelStore<SlicedLabel, I = T::Label>,
        M: MonoMappingStore,
        const SIZE_THRESHOLD: usize,  // = 1000,
        const SIM_THRESHOLD_NUM: u64, // = 1,
        const SIM_THRESHOLD_DEN: u64, // = 2,
    >
    GreedyBottomUpMatcher<
        'a,
        Dsrc,
        Ddst,
        T,
        S,
        LS,
        M,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    pub fn new(
        node_store: &'a S,
        label_store: &'a LS,
        src_arena: Dsrc,
        dst_arena: Ddst,
        mappings: M,
    ) -> Self {
        Self {
            label_store,
            internal: BottomUpMatcher {
                node_store,
                src_arena,
                dst_arena,
                mappings,
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
        }
    }
}

/// TODO PostOrder might not be necessary
impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<'a, T, Dsrc::IdD, M::Src>
            + DecompressedWithParent<'a, T, Dsrc::IdD>
            + PostOrder<'a, T, Dsrc::IdD, M::Src>
            + PostOrderIterable<'a, T, Dsrc::IdD, M::Src>
            + DecompressedSubtree<'a, T>
            + ContiguousDescendants<'a, T, Dsrc::IdD, M::Src>
            + LazyPOBorrowSlice<'a, T, Dsrc::IdD, M::Src>
            + ShallowDecompressedTreeStore<'a, T, Dsrc::IdD, M::Src>
            + LazyDecompressedTreeStore<'a, T, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<'a, T, Ddst::IdD, M::Dst>
            + DecompressedWithParent<'a, T, Ddst::IdD>
            + PostOrder<'a, T, Ddst::IdD, M::Dst>
            + PostOrderIterable<'a, T, Ddst::IdD, M::Dst>
            + DecompressedSubtree<'a, T>
            + ContiguousDescendants<'a, T, Ddst::IdD, M::Dst>
            + LazyPOBorrowSlice<'a, T, Ddst::IdD, M::Dst>
            + ShallowDecompressedTreeStore<'a, T, Ddst::IdD, M::Dst>
            + LazyDecompressedTreeStore<'a, T, M::Dst>,
        T: 'a + Tree + WithHashs + WithStats,
        S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        LS: 'a + LabelStore<SlicedLabel, I = T::Label>,
        M: MonoMappingStore,
        MZs: MonoMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>,
        const SIZE_THRESHOLD: usize,
        const SIM_THRESHOLD_NUM: u64,
        const SIM_THRESHOLD_DEN: u64,
    >
    GreedyBottomUpMatcher<
        'a,
        Dsrc,
        Ddst,
        T,
        S,
        LS,
        M,
        MZs,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
where
    T::TreeId: 'a + Clone + Debug,
    T::Type: Debug,
    Dsrc::IdD: 'a + PrimInt + std::ops::SubAssign + Debug,
    Ddst::IdD: 'a + PrimInt + std::ops::SubAssign + Debug,
    M::Src: 'a + PrimInt + std::ops::SubAssign + Debug,
    M::Dst: 'a + PrimInt + std::ops::SubAssign + Debug,
{
    // pub fn matchh<'b>(
    //     compressed_node_store: &'a S,
    //     label_store: &'a LS,
    //     src: &'a T::TreeId,
    //     dst: &'a T::TreeId,
    //     mut mappings: M,
    // ) -> Self {
    //     let src_arena = Dsrc::new(compressed_node_store, src);
    //     let dst_arena = Ddst::new(compressed_node_store, dst);
    //     let src_len = ShallowDecompressedTreeStore::<'a, T, Dsrc::IdD, M::Src>::len(&src_arena);
    //     let dst_len = ShallowDecompressedTreeStore::<'a, T, Ddst::IdD, M::Dst>::len(&dst_arena);
    //     mappings.topit(src_len + 1, src_len + 1);
    //     let mut matcher = Self::new(
    //         compressed_node_store,
    //         label_store,
    //         src_arena,
    //         dst_arena,
    //         mappings,
    //     );
    //     Self::execute(&mut matcher);
    //     matcher
    // }

    pub fn match_it<HAST>(
        mapping: crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
    where
        HAST: HyperAST<'a, NS = S, LS = LS>,
    {
        let mut matcher = Self {
            internal: BottomUpMatcher {
                node_store: mapping.hyperast.node_store(),
                src_arena: mapping.mapping.src_arena,
                dst_arena: mapping.mapping.dst_arena,
                mappings: mapping.mapping.mappings,
                _phantom: PhantomData,
            },
            label_store: mapping.hyperast.label_store(),
            _phantom: PhantomData,
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
            let a = self
                .internal
                .src_arena
                .decompress_to(self.internal.node_store, &a);
            if self.src_has_children(a) {
                let candidates = self.internal.get_dst_candidates(&a);
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
        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
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
        let r = self.internal.node_store.resolve(&o).has_children();
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

    pub(crate) fn last_chance_match_zs(&mut self, src: Dsrc::IdD, dst: Ddst::IdD) {
        // WIP https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
        let src_s = self
            .internal
            .src_arena
            .descendants_count(self.internal.node_store, &src);
        let dst_s = self
            .internal
            .dst_arena
            .descendants_count(self.internal.node_store, &dst);
        if !(src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap()) {
            return;
        }
        let node_store = self.internal.node_store;
        let label_store = self.label_store;
        let src_offset;
        let dst_offset;
        let mappings: MZs = if SLICE {
            let src_arena = self.internal.src_arena.slice_po(node_store, &src);
            src_offset = src - src_arena.root();
            let dst_arena = self.internal.dst_arena.slice_po(node_store, &dst);
            dst_offset = dst - dst_arena.root();
            ZsMatcher::match_with(node_store, label_store, src_arena, dst_arena)
        } else {
            let o_src = self.internal.src_arena.original(&src);
            let o_dst = self.internal.dst_arena.original(&dst);
            let src_arena = ZsTree::<T, Dsrc::IdD>::decompress(node_store, &o_src);
            src_offset = src - src_arena.root();
            if cfg!(debug_assertions) {
                let src_arena_z = self.internal.src_arena.slice_po(node_store, &src);
                for i in src_arena.iter_df_post::<true>() {
                    assert_eq!(src_arena.tree(&i), src_arena_z.tree(&i));
                    assert_eq!(src_arena.lld(&i), src_arena_z.lld(&i));
                }
                let mut last = src_arena_z.root();
                for k in src_arena_z.iter_kr() {
                    assert!(src_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(src_arena.kr[src_arena.kr.len() - 1]);
                dbg!(last == src_arena_z.root());
            }
            let dst_arena = ZsTree::<T, Ddst::IdD>::decompress(node_store, &o_dst);
            dst_offset = dst - dst_arena.root();
            if cfg!(debug_assertions) {
                let dst_arena_z = self.internal.dst_arena.slice_po(node_store, &dst);
                for i in dst_arena.iter_df_post::<true>() {
                    assert_eq!(dst_arena.tree(&i), dst_arena_z.tree(&i));
                    assert_eq!(dst_arena.lld(&i), dst_arena_z.lld(&i));
                }
                let mut last = dst_arena_z.root();
                for k in dst_arena_z.iter_kr() {
                    assert!(dst_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(dst_arena.kr[dst_arena.kr.len() - 1]);
                dbg!(last == dst_arena_z.root());
            }
            ZsMatcher::match_with(node_store, label_store, src_arena, dst_arena)
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
                    .node_store
                    .resolve(&self.internal.src_arena.original(&src))
                    .get_type();
                let tdst = self
                    .internal
                    .node_store
                    // .resolve(&matcher.src_arena.tree(&t))
                    .resolve(&self.internal.dst_arena.original(&dst))
                    .get_type();
                if tsrc == tdst {
                    self.internal.mappings.link(*src.shallow(), *dst.shallow());
                }
            }
        }
    }
}
