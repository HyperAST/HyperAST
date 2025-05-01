use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice,
        PostOrder, PostOrderIterable,
    },
    matchers::{mapping_store::MonoMappingStore, similarity_metrics},
};
use hyperast::types::{
    self, DecompressedFrom, HashKind, HyperAST, NodeId, NodeStore, Tree, TypeStore, WithHashs,
};
use hyperast::PrimInt;
use num_traits::{ToPrimitive, Zero};
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

pub struct BottomUpMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<
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
        M: MonoMappingStore,
    > BottomUpMatcher<Dsrc, Ddst, HAST, M>
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
            stores: mapping.hyperast,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            mappings: mapping.mapping.mappings,
        };
        // matcher
        // .mappings
        // .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        Self::execute(&mut matcher);
        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.src_arena,
                dst_arena: matcher.dst_arena,
                mappings: matcher.mappings,
            },
        }
    }

    pub fn execute(&mut self) {
        // List<Tree> dstTrees = TreeUtils.postOrder(dst);
        // for (Tree currentSrcTree : src.postOrder()) {
        //     int numberOfLeaves = numberOfLeaves(currentSrcTree);
        //     for (Tree currentDstTree : dstTrees) {
        //         if (mappings.isMappingAllowed(currentSrcTree, currentDstTree)
        //                 && !(currentSrcTree.isLeaf() || currentDstTree.isLeaf())) {
        //             double similarity = SimilarityMetrics.chawatheSimilarity(currentSrcTree, currentDstTree, mappings);
        //             if ((numberOfLeaves > maxNumberOfLeaves && similarity >= structSimThreshold1)
        //                     || (numberOfLeaves <= maxNumberOfLeaves && similarity >= structSimThreshold2)) {
        //                 mappings.addMapping(currentSrcTree, currentDstTree);
        //                 break;
        //             }
        //         }
        //     }
        // }

        let mut dst_trees = self.dst_arena.iter_df_post::<true>();
        let max_number_of_leaves = 1000; // TODO: make configurable
        let struct_sim_threshold1 = 0.5;
        let struct_sim_threshold2 = 0.6;

        log::debug!(
            "Starting bottom-up matching with thresholds: {}, {}",
            struct_sim_threshold1,
            struct_sim_threshold2
        );

        for s in self.src_arena.iter_df_post::<true>() {
            let src_tree = s;
            let dst_tree = dst_trees.next().unwrap();
            let number_of_leaves = self.number_of_leaves_src(&src_tree);

            log::debug!("Examining source tree with {} leaves", number_of_leaves);

            let mapping_allowed = self.is_mapping_allowed(&src_tree, &dst_tree);
            let src_is_leaf = self.src_arena.children(&src_tree).is_empty();
            let dst_is_leaf = self.dst_arena.children(&dst_tree).is_empty();

            log::debug!(
                "Mapping allowed: {}, source tree is leaf: {}, destination tree is leaf: {}",
                mapping_allowed,
                src_is_leaf,
                dst_is_leaf
            );

            if mapping_allowed && !(src_is_leaf || dst_is_leaf) {
                let similarity = similarity_metrics::chawathe_similarity(
                    &self.src_arena.descendants(&src_tree),
                    &self.dst_arena.descendants(&dst_tree),
                    &self.mappings,
                );

                log::debug!("Mapping allowed, similarity: {}", similarity);

                if (number_of_leaves > max_number_of_leaves && similarity >= struct_sim_threshold1)
                    || (number_of_leaves <= max_number_of_leaves
                        && similarity >= struct_sim_threshold2)
                {
                    log::debug!("Adding mapping for trees with similarity {}", similarity);
                    self.mappings.link(src_tree, dst_tree);
                    break;
                }
            }
        }
        log::debug!("Completed mapping process");
        ()
    }

    fn number_of_leaves_src(&self, src_tree: &M::Src) -> usize {
        let children = &self.src_arena.children(src_tree);
        if children.is_empty() {
            1 // If no children, this is a leaf
        } else {
            children
                .iter()
                .map(|child| self.number_of_leaves_src(child))
                .sum()
        }
    }

    /// This function checks if a mapping between two nodes is allowed.
    /// It returns true if the nodes are of the same type, and are both unmapped.
    fn is_mapping_allowed(&self, src_tree: &M::Src, dst_tree: &M::Dst) -> bool {
        let src_linked = self.mappings.get_src(dst_tree).is_some();
        let dst_linked = self.mappings.get_dst(src_tree).is_some();

        log::debug!("Checking mapping between {:?} and {:?}", src_tree, dst_tree);
        log::debug!("Source linked: {}", src_linked);
        log::debug!("Destination linked: {}", dst_linked);

        if src_linked || dst_linked {
            return false;
        }

        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        let src_type = self.stores.resolve_type(&original_src);
        let dst_type = self.stores.resolve_type(&original_dst);

        log::debug!("Source type: {:?}", src_type);
        log::debug!("Destination type: {:?}", dst_type);

        src_type == dst_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
    use crate::{
        decompressed_tree_store::CompletePostOrder,
        matchers::{mapping_store::DefaultMappingStore, Decompressible, Mapper},
        tests::examples::example_simple,
        tree::simple_tree::vpair_to_stores,
    };

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn test_single_node_match() {
        init();
        // Create two identical single-node trees
        let (stores, src, dst) = vpair_to_stores(example_simple());

        log::info!("Initialized logging");

        // Create the mapping structure
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &src,
                ),
                dst_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &dst,
                ),
                mappings: DefaultMappingStore::default(),
            },
        };

        // Run the bottom-up matcher
        let result = BottomUpMatcher::match_it(mapping);

        // Verify that the root nodes are mapped to each other
        let mapped_root = result
            .mapping
            .mappings
            .get_dst(&result.mapping.src_arena.root());
        assert!(mapped_root.is_some());
        assert_eq!(mapped_root.unwrap(), result.mapping.dst_arena.root());
    }
}
