use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::{Mapper, Mapping, mapping_store::MonoMappingStore, similarity_metrics},
};
use hyperast::PrimInt;
use hyperast::types::{HyperAST, NodeId, WithHashs};
use std::fmt::Debug;

use super::BottomUpMatcherConfig;

pub struct LazyBottomUpMatcher<Dsrc, Ddst, HAST, M: MonoMappingStore> {
    pub hyperast: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub config: BottomUpMatcherConfig,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> LazyBottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug + NodeId<IdN = HAST::IdN>,
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
        + LazyDecompressedTreeStore<HAST, M::Dst>,
{
    pub fn with_config(
        mapping: Mapper<HAST, Dsrc, Ddst, M>,
        config: BottomUpMatcherConfig,
    ) -> Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            hyperast: mapping.hyperast,
            mappings: mapping.mapping.mappings,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            config,
        };
        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        matcher.execute();
        matcher.into()
    }

    pub fn match_it(mapping: Mapper<HAST, Dsrc, Ddst, M>) -> Mapper<HAST, Dsrc, Ddst, M> {
        Self::with_config(mapping, BottomUpMatcherConfig::default())
    }

    fn execute(&mut self) {
        for s in self.src_arena.iter_df_post::<true>() {
            let src = self.src_arena.decompress_to(&s);
            let number_of_leaves = self.count_leaves(&src);

            for d in self.dst_arena.iter_df_post::<true>() {
                let dst = self.dst_arena.decompress_to(&d);

                if self.is_mapping_allowed(&src, &dst) {
                    let src_is_leaf = self.src_arena.children(&src).is_empty();
                    let dst_is_leaf = self.dst_arena.children(&dst).is_empty();

                    if !(src_is_leaf || dst_is_leaf) {
                        let similarity = self.compute_similarity(&src, &dst);
                        let threshold = if number_of_leaves > self.config.max_leaves {
                            self.config.sim_threshold_large_trees
                        } else {
                            self.config.sim_threshold_small_trees
                        };

                        if similarity >= threshold {
                            self.mappings.link(*src.shallow(), *dst.shallow());
                            break;
                        }
                    }
                }
            }
        }
    }

    fn count_leaves(&mut self, src: &Dsrc::IdD) -> usize {
        self.src_arena
            .descendants(src)
            .iter()
            .filter(|t| {
                let id = self.src_arena.decompress_to(&t);
                self.src_arena.children(&id).is_empty()
            })
            .count()
    }

    fn is_mapping_allowed(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> bool {
        if self.mappings.is_src(src.shallow()) || self.mappings.is_dst(dst.shallow()) {
            return false;
        }

        let src_type = self.hyperast.resolve_type(&self.src_arena.original(src));
        let dst_type = self.hyperast.resolve_type(&self.dst_arena.original(dst));

        src_type == dst_type
    }

    fn compute_similarity(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> f64 {
        // Using the optimized range-based similarity computation
        similarity_metrics::SimilarityMeasure::range(
            &self.src_arena.descendants_range(src),
            &self.dst_arena.descendants_range(dst),
            &self.mappings,
        )
        .dice()
    }
}

impl<HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for LazyBottomUpMatcher<Dsrc, Ddst, HAST, M>
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
        Mapper {
            hyperast: self.hyperast,
            mapping: Mapping {
                src_arena: self.src_arena,
                dst_arena: self.dst_arena,
                mappings: self.mappings,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore};
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;

    use hyperast::types::DecompressedFrom;

    #[test]
    fn test_bottom_up_matcher() {
        // Setup simple source and destination trees with similar structure
        // Source tree: a -> [e -> [f], b -> [c, d]]
        // Dest tree:   a -> [e -> [g], b -> [c, d]]
        // Only difference is 'f' vs 'g'
        let src = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "f")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);
        let dst = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "g")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);

        // Create the necessary stores for the test trees
        let (stores, src, dst) = vpair_to_stores((src, dst));

        // Create lazy post-order representations for the matcher
        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        // Also create complete post-order representations for accessing nodes by path
        let src_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &src);
        let dst_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &dst);

        // let src_arena_decomp = Decompressible::<_, CompletePostOrder<_, u16>>::from(src_arena_decomp);
        // let dst_arena_decomp = Decompressible::<_, CompletePostOrder<_, u16>>::from(dst_arena_decomp);

        // Initialize the mapping store
        let mut mappings = DefaultMappingStore::default();
        mappings.topit(src_arena.len(), dst_arena.len());

        // Get references to nodes we want to pre-map (c and d nodes)
        let src_root = src_arena_decomp.root();
        let dst_root = dst_arena_decomp.root();
        let src_node_c = src_arena_decomp.child(&src_root, &[1, 0]);
        let src_node_d = src_arena_decomp.child(&src_root, &[1, 1]);
        let dst_node_c = dst_arena_decomp.child(&dst_root, &[1, 0]);
        let dst_node_d = dst_arena_decomp.child(&dst_root, &[1, 1]);

        // Establish initial mappings for the bottom-up matcher
        mappings.link(src_node_c, dst_node_c);
        mappings.link(src_node_d, dst_node_d);

        // Create the mapper with initial configuration
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings,
            },
        };

        // Run the lazy bottom-up matcher
        let result = LazyBottomUpMatcher::match_it(mapping);

        // Verify the number of mappings
        // We expect 4 mappings - the two pre-mapped nodes, the 'b' node, and the root
        assert_eq!(
            result.mappings.len(),
            4,
            "Expected exactly 3 mappings, got {}",
            result.mappings.len()
        );

        // Get references to the nodes in the result
        let src_root = result.mapping.src_arena.root();
        let dst_root = result.mapping.dst_arena.root();

        // Verify root nodes are mapped
        assert!(
            result.mapping.mappings.has(&src_root, &dst_root),
            "Root nodes should be mapped"
        );

        let src_children = result.mapping.src_arena.children(&src_root);
        let dst_children = result.mapping.dst_arena.children(&dst_root);

        // Verify the 'b' node is mapped (at index 1 in children array)
        assert!(
            result
                .mapping
                .mappings
                .has(&src_children[1].shallow(), &dst_children[1].shallow()),
            "The 'b' nodes should be mapped"
        );

        // Verify that the children of 'b' are correctly mapped
        let src_b_children = result.mapping.src_arena.children(&src_children[1]);
        let dst_b_children = result.mapping.dst_arena.children(&dst_children[1]);

        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[0].shallow(), &dst_b_children[0].shallow()),
            "The 'c' nodes should be mapped"
        );
        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[1].shallow(), &dst_b_children[1].shallow()),
            "The 'd' nodes should be mapped"
        );
    }
}
