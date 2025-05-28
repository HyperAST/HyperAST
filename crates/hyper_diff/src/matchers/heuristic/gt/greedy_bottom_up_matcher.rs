use crate::decompressed_tree_store::SimpleZsTree as ZsTree;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Decompressible, Mapper, Mapping};
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use num_traits::{cast, one};
use std::fmt::Debug;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct GreedyBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 1000,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    pub(crate) internal: Mapper<HAST, Dsrc, Ddst, M>,
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
> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for GreedyBottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
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
> GreedyBottomUpMatcher<Dsrc, Ddst, HAST, M, SIZE_THRESHOLD, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
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
            internal: Mapper {
                hyperast: stores,
                mapping: Mapping {
                    src_arena,
                    dst_arena,
                    mappings,
                },
            },
        }
    }

    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = mapping;
        matcher.mapping.mappings.topit(
            matcher.mapping.src_arena.len(),
            matcher.mapping.dst_arena.len(),
        );
        let mut matcher = Self { internal: matcher };
        Self::execute(&mut matcher.internal);
        matcher.internal
    }

    pub fn matchh(store: HAST, src: &'a HAST::IdN, dst: &'a HAST::IdN, mappings: M) -> Self {
        let mut matcher = Self::new(
            store,
            Dsrc::decompress(store, src),
            Ddst::decompress(store, dst),
            mappings,
        );
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal);
        matcher
    }

    pub fn execute<'b>(mapper: &mut Mapper<HAST, Dsrc, Ddst, M>) {
        assert_eq!(
            // TODO move it inside the arena ...
            mapper.src_arena.root(),
            cast::<_, M::Src>(mapper.src_arena.len()).unwrap() - one()
        );
        assert!(mapper.src_arena.len() > 0);
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in mapper.src_arena.iter_df_post::<true>() {
            if mapper.src_arena.parent(&a).is_none() {
                // TODO remove and flip const param of iter_df_post
                break;
            }
            if !(mapper.mappings.is_src(&a) || !Self::src_has_children(mapper, a)) {
                let candidates = mapper.get_dst_candidates(&a);
                let mut best = None;
                let mut max: f64 = -1.;
                for cand in candidates {
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &mapper.src_arena.descendants_range(&a),
                        &mapper.dst_arena.descendants_range(&cand),
                        &mapper.mappings,
                    )
                    .dice();
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = Some(cand);
                    }
                }

                if let Some(best) = best {
                    Self::last_chance_match_zs(mapper, a, best);
                    mapper.mappings.link(a, best);
                }
            }
        }
        // for root
        mapper.mapping.mappings.link(
            mapper.mapping.src_arena.root(),
            mapper.mapping.dst_arena.root(),
        );
        Self::last_chance_match_zs(mapper, mapper.src_arena.root(), mapper.dst_arena.root());
    }

    fn src_has_children(mapper: &mut Mapper<HAST, Dsrc, Ddst, M>, src: M::Src) -> bool {
        use num_traits::ToPrimitive;
        let r = mapper
            .hyperast
            .node_store()
            .resolve(&mapper.src_arena.original(&src))
            .has_children();
        assert_eq!(
            r,
            mapper.src_arena.lld(&src) < src,
            "{:?} {:?}",
            mapper.src_arena.lld(&src),
            src.to_usize()
        );
        r
    }

    pub(crate) fn last_chance_match_zs(
        mapper: &mut Mapper<HAST, Dsrc, Ddst, M>,
        src: M::Src,
        dst: M::Dst,
    ) {
        // WIP https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
        let src_s = mapper.src_arena.descendants_count(&src);
        let dst_s = mapper.dst_arena.descendants_count(&dst);
        if !(src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap()) {
            return;
        }
        let stores = mapper.hyperast;
        let src_offset;
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let mappings: M = if SLICE {
            let src_arena = mapper.mapping.src_arena.slice_po(&src);
            src_offset = src - src_arena.root();
            let dst_arena = mapper.mapping.dst_arena.slice_po(&dst);
            ZsMatcher::match_with(mapper.hyperast, src_arena, dst_arena)
        } else {
            let o_src = mapper.mapping.src_arena.original(&src);
            let o_dst = mapper.mapping.dst_arena.original(&dst);
            let src_arena = ZsTree::<HAST::IdN, M::Src>::decompress(stores, &o_src);
            let src_arena = Decompressible {
                hyperast: stores,
                decomp: src_arena,
            };
            src_offset = src - src_arena.root();
            if cfg!(debug_assertions) {
                let src_arena_z = mapper.src_arena.slice_po(&src);
                for i in src_arena.iter_df_post::<true>() {
                    assert_eq!(src_arena.tree(&i), src_arena_z.tree(&i));
                    assert_eq!(src_arena.lld(&i), src_arena_z.lld(&i));
                }
                use num_traits::ToPrimitive;
                let mut last = src_arena_z.root();
                for k in src_arena_z.iter_kr() {
                    assert!(src_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(src_arena.kr[src_arena.kr.len() - 1]);
                dbg!(last == src_arena_z.root());
            }
            let dst_arena = ZsTree::<HAST::IdN, M::Dst>::decompress(stores, &o_dst);
            let dst_arena = Decompressible {
                hyperast: stores,
                decomp: dst_arena,
            };
            if cfg!(debug_assertions) {
                let dst_arena_z = mapper.dst_arena.slice_po(&dst);
                for i in dst_arena.iter_df_post::<true>() {
                    assert_eq!(dst_arena.tree(&i), dst_arena_z.tree(&i));
                    assert_eq!(dst_arena.lld(&i), dst_arena_z.lld(&i));
                }
                use num_traits::ToPrimitive;
                let mut last = dst_arena_z.root();
                for k in dst_arena_z.iter_kr() {
                    assert!(dst_arena.kr[k.to_usize().unwrap()]);
                    last = k;
                }
                assert!(dst_arena.kr[dst_arena.kr.len() - 1]);
                dbg!(last == dst_arena_z.root());
            }
            ZsMatcher::match_with(mapper.hyperast, src_arena, dst_arena)
        };
        let dst_offset = mapper.dst_arena.first_descendant(&dst);
        assert_eq!(mapper.src_arena.first_descendant(&src), src_offset);
        for (i, t) in mappings.iter() {
            //remapping
            let src: M::Src = src_offset + cast(i).unwrap();
            let dst: M::Dst = dst_offset + cast(t).unwrap();
            // use it
            if !mapper.mappings.is_src(&src) && !mapper.mappings.is_dst(&dst) {
                let tsrc = mapper
                    .hyperast
                    .resolve_type(&mapper.src_arena.original(&src));
                let tdst = mapper
                    .hyperast
                    .resolve_type(&mapper.dst_arena.original(&dst));
                if tsrc == tdst {
                    mapper.mappings.link(src, dst);
                }
            }
        }
    }
}
