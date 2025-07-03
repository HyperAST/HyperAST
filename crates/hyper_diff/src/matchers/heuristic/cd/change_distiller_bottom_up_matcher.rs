use crate::decompressed_tree_store::SimpleZsTree as ZsTree;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::matchers::heuristic::gt::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Decompressible, Mapper};
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::PrimInt;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, Tree, WithHashs};
use num_traits::{cast, one};
use std::fmt::Debug;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct ChangeDistillerBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 4,
    const SIM_THRESHOLD_NUM: u64 = 6,
    const SIM_THRESHOLD_DEN: u64 = 10,
    const SIM_THRESHOLD2_NUM: u64 = 4,
    const SIM_THRESHOLD2_DEN: u64 = 10,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
    Dsrc,
    Ddst,
    HAST: HyperAST,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize,   // = 1000,
    const SIM_THRESHOLD_NUM: u64,  // = 6,
    const SIM_THRESHOLD_DEN: u64,  // = 10,
    const SIM_THRESHOLD2_NUM: u64, // = 4,
    const SIM_THRESHOLD2_DEN: u64, // = 10,
> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for ChangeDistillerBottomUpMatcher<
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
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
        self.internal
    }
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
    const SIZE_THRESHOLD: usize,   // = 1000,
    const SIM_THRESHOLD_NUM: u64,  // = 6,
    const SIM_THRESHOLD_DEN: u64,  // = 10,
    const SIM_THRESHOLD2_NUM: u64, // = 4,
    const SIM_THRESHOLD2_DEN: u64, // = 10,
>
    ChangeDistillerBottomUpMatcher<
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
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mut matcher: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        matcher.mapping.mappings.topit(
            matcher.mapping.src_arena.len(),
            matcher.mapping.dst_arena.len(),
        );
        let mut matcher = Self { internal: matcher };
        Self::execute(&mut matcher);
        matcher.internal
    }

    pub fn iter_leaves<D, IdD: Eq>(arena: &D) -> impl Iterator<Item = IdD>
    where
        D: PostOrderIterable<HAST, IdD> + PostOrder<HAST, IdD>,
    {
        arena.iter_df_post::<true>().filter(|x|
            // is leaf. kind of an optimisation, it was easier like this anyway.
            arena.lld(x) == *x)
    }

    fn sim(internal: &Mapper<HAST, Dsrc, Ddst, M>, src: M::Src, dst: M::Dst) -> f64 {
        similarity_metrics::chawathe_similarity(
            &internal.src_arena.descendants(&src),
            &internal.dst_arena.descendants(&dst),
            &internal.mappings,
        )
    }

    pub fn execute<'b>(&mut self) {
        let internal = &self.internal;
        // List<Tree> dstTrees = TreeUtils.postOrder(dst);
        let mut dsts = internal.dst_arena.iter_df_post::<true>();

        // for (Tree currentSrcTree : src.postOrder()) {
        for src in internal.src_arena.iter_df_post::<true>() {
            let internal = &self.internal;
            // int numberOfLeaves = numberOfLeaves(currentSrcTree);
            dbg!(src);
            let aaa = internal.src_arena.lld(&src);
            dbg!(aaa);
            let number_of_leaves =
                crate::decompressed_tree_store::Iter::new(aaa, src) // TODO check if it is lld or lld - 1
                    .filter(|x| internal.src_arena.lld(dbg!(x)) == *x)
                    .count();
            // TODO use the properties of the post order traversal to compute the number of leafs incrementally
            // TODO use the derived data of hyper ast to count number_of_leaves

            // List<Tree> dstTrees = TreeUtils.postOrder(dst);
            let mut dsts = internal.dst_arena.iter_df_post::<true>();

            // for (Tree currentDstTree : dstTrees) {
            loop {
                let Some(dst) = dsts.next() else { break };
                dbg!(dst);
                let internal = &self.internal;
                let mapping = &internal.mapping;
                let mappings = &mapping.mappings;
                // mappings.isMappingAllowed(currentSrcTree, currentDstTree)
                if !mappings.is_src(&src) && !mappings.is_dst(&dst) {
                    dbg!();
                    let tsrc = mapping.src_arena.original(&src);
                    let tsrc = internal.hyperast.resolve_type(&tsrc);
                    let tdst = mapping.dst_arena.original(&dst);
                    let tdst = internal.hyperast.resolve_type(&tdst);
                    if tsrc == tdst {
                        // !(currentSrcTree.isLeaf() || currentDstTree.isLeaf())
                        dbg!();
                        if !(internal.src_arena.lld(&src) == src
                            || internal.dst_arena.lld(&dst) == dst)
                        {
                            // double similarity = SimilarityMetrics.chawatheSimilarity(currentSrcTree, currentDstTree, mappings);
                            let sim = Self::sim(internal, src, dst);
                            dbg!(src, dst, sim);

                            // numberOfLeaves > maxNumberOfLeaves && similarity >= structSimThreshold1
                            let cond1 = number_of_leaves > SIZE_THRESHOLD
                                && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                            // numberOfLeaves <= maxNumberOfLeaves && similarity >= structSimThreshold2
                            let cond2 = number_of_leaves <= SIZE_THRESHOLD
                                && sim >= SIM_THRESHOLD2_NUM as f64 / SIM_THRESHOLD2_DEN as f64;
                            if cond1 || cond2 {
                                dbg!();
                                self.internal.mapping.mappings.link(src, dst);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

// private static final double DEFAULT_STRUCT_SIM_THRESHOLD_1 = 0.6;
// protected double structSimThreshold1 = DEFAULT_STRUCT_SIM_THRESHOLD_1;

// private static final double DEFAULT_STRUCT_SIM_THRESHOLD_2 = 0.4;
// protected double structSimThreshold2 = DEFAULT_STRUCT_SIM_THRESHOLD_1;

// private static final int DEFAULT_MAX_NUMBER_OF_LEAVES = 4;
// protected int maxNumberOfLeaves = DEFAULT_MAX_NUMBER_OF_LEAVES;

// public ChangeDistillerBottomUpMatcher() {
// }

// @Override
// public void configure(GumtreeProperties properties) {
//     structSimThreshold1 = properties.tryConfigure(ConfigurationOptions.cd_structsim1,
//             DEFAULT_STRUCT_SIM_THRESHOLD_1);

//     structSimThreshold2 = properties.tryConfigure(ConfigurationOptions.cd_structsim2,
//             DEFAULT_STRUCT_SIM_THRESHOLD_2);

//     maxNumberOfLeaves = properties.tryConfigure(ConfigurationOptions.cd_maxleaves, DEFAULT_MAX_NUMBER_OF_LEAVES);

// }

// @Override
// public MappingStore match(Tree src, Tree dst, MappingStore mappings) {
//     List<Tree> dstTrees = TreeUtils.postOrder(dst);
//     for (Tree currentSrcTree : src.postOrder()) {
//         int numberOfLeaves = numberOfLeaves(currentSrcTree);
//         for (Tree currentDstTree : dstTrees) {
//             if (mappings.isMappingAllowed(currentSrcTree, currentDstTree)
//                     && !(currentSrcTree.isLeaf() || currentDstTree.isLeaf())) {
//                 double similarity = SimilarityMetrics.chawatheSimilarity(currentSrcTree, currentDstTree, mappings);
//                 if ((numberOfLeaves > maxNumberOfLeaves && similarity >= structSimThreshold1)
//                         || (numberOfLeaves <= maxNumberOfLeaves && similarity >= structSimThreshold2)) {
//                     mappings.addMapping(currentSrcTree, currentDstTree);
//                     break;
//                 }
//             }
//         }
//     }

//     return mappings;
// }

#[cfg(test)]
mod tests {
    use hyperast::test_utils::simple_tree::{SimpleTree, vpair_to_stores};
    use hyperast::types::{DecompressedFrom, HyperASTShared};

    use crate::{
        decompressed_tree_store::CompletePostOrder,
        matchers::{
            Decompressible,
            mapping_store::{MappingStore, MultiVecStore},
        },
    };

    use super::super::change_distiller_leaves_matcher::ChangeDistillerLeavesMatcher;

    use super::*;

    #[test]
    fn test_leaf_matcher() {
        use crate::tests::tree;
        let (stores, src, dst) = vpair_to_stores((
            tree!(0, "a"; [
                tree!(0, "b"),
                tree!(0, "c"),
            ]),
            tree!(0, "a"; [
                tree!(0, "c"),
                tree!(0, "b"),
            ]),
        ));

        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

        let src_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &src);
        let dst_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &dst);
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings: crate::matchers::mapping_store::VecStore::default(),
            },
        };
        //  MappingStore mappings = new ChangeDistillerLeavesMatcher().match(src, dst);
        let mapping =
            ChangeDistillerLeavesMatcher::<_, _, _, _>::match_it::<MultiVecStore<_>>(mapping);
        // assertEquals(2, mappings.size());
        assert_eq!(2, mapping.mapping.mappings.len());
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = mapping.mapping.src_arena.root();
        let src_cs = mapping.mapping.src_arena.children(&src);
        let dst = mapping.mapping.dst_arena.root();
        let dst_cs = mapping.mapping.dst_arena.children(&dst);
        dbg!(&mapping.mapping.mappings);
        dbg!(&src_cs);
        dbg!(&dst_cs);
        // assertTrue(mappings.has(src.getChild(0), dst.getChild(1)));
        assert!(mapping.mapping.mappings.has(&src_cs[0], &dst_cs[1]));
        // assertTrue(mappings.has(src.getChild(1), dst.getChild(0)));
        assert!(mapping.mapping.mappings.has(&src_cs[1], &dst_cs[0]));

        let mapping = ChangeDistillerBottomUpMatcher::<_, _, _, _, 1>::match_it(mapping);
        dbg!(&mapping.mapping.mappings);
        assert!(mapping.mapping.mappings.has(&src, &dst));
    }
}
