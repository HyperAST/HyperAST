use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice,
        PostOrder, PostOrderIterable,
    },
    matchers::{
        Mapper, Mapping,
        heuristic::cd::iterator::{CustomIteratorConfig, DecompressedCustomPostOrderIterator},
        mapping_store::{MappingStore, MonoMappingStore},
        similarity_metrics,
    },
};
use hyperast::types::{DecompressedFrom, HyperAST, HyperType, NodeId, WithHashs};
use hyperast::{PrimInt, types::LabelStore};
use std::fmt::Debug;

use super::BottomUpMatcherConfig;

pub struct BottomUpMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub config: BottomUpMatcherConfig,
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
    pub fn with_config(
        mapping: Mapper<HAST, Dsrc, Ddst, M>,
        config: BottomUpMatcherConfig,
    ) -> Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            stores: mapping.hyperast,
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

    pub fn execute(&mut self) {
        for s in self.get_src_nodes() {
            let src_tree = s;
            let number_of_leaves = self.number_of_leaves_src(&src_tree);

            for dst_tree in self.get_dst_nodes() {
                let mapping_allowed = self.is_mapping_allowed(&src_tree, &dst_tree);
                let src_is_leaf = self.src_arena.children(&src_tree).is_empty();
                let dst_is_leaf = self.dst_arena.children(&dst_tree).is_empty();

                if mapping_allowed && !(src_is_leaf || dst_is_leaf) {
                    let similarity = similarity_metrics::chawathe_similarity(
                        &self.src_arena.descendants(&src_tree),
                        &self.dst_arena.descendants(&dst_tree),
                        &self.mappings,
                    );

                    if (number_of_leaves > self.config.max_leaves
                        && similarity >= self.config.sim_threshold_large_trees)
                        || (number_of_leaves <= self.config.max_leaves
                            && similarity >= self.config.sim_threshold_small_trees)
                    {
                        self.mappings.link(src_tree, dst_tree);
                        break;
                    }
                }
            }
        }
        ()
    }

    fn get_src_nodes(&self) -> Vec<<M as MappingStore>::Src> {
        let iter = DecompressedCustomPostOrderIterator::new(
            &self.src_arena,
            self.stores,
            self.src_arena.root(),
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &Dsrc, stores: HAST, node: &<M as MappingStore>::Src| -> bool {
                if arena.children(node).is_empty() {
                    return true;
                }
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                node_type.is_statement()
            },
        );
        iter.collect::<Vec<_>>()
    }

    fn get_dst_nodes(&self) -> Vec<<M as MappingStore>::Dst> {
        let iter = DecompressedCustomPostOrderIterator::new(
            &self.dst_arena,
            self.stores,
            self.dst_arena.root(),
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &Ddst, stores: HAST, node: &<M as MappingStore>::Dst| -> bool {
                if arena.children(node).is_empty() {
                    return true;
                }
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                node_type.is_statement()
            },
        );
        iter.collect::<Vec<_>>()
    }

    fn number_of_leaves_src(&self, src_tree: &M::Src) -> usize {
        self.src_arena
            .descendants(src_tree)
            .iter()
            .filter(|node| {
                if self.src_arena.children(node).is_empty() {
                    return true;
                }
                let original = self.src_arena.original(node);
                let node_type = self.stores.resolve_type(&original);
                node_type.is_statement()
            })
            .count()
    }

    /// This function checks if a mapping between two nodes is allowed.
    /// It returns true if the nodes are of the same type, and are both unmapped.
    fn is_mapping_allowed(&self, src_tree: &M::Src, dst_tree: &M::Dst) -> bool {
        if self.mappings.is_src(src_tree) || self.mappings.is_dst(dst_tree) {
            return false;
        }

        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        let src_type = self.stores.resolve_type(&original_src);
        let dst_type = self.stores.resolve_type(&original_dst);

        src_type == dst_type
    }
}

impl<HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for BottomUpMatcher<Dsrc, Ddst, HAST, M>
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
        Mapper {
            hyperast: self.stores,
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
    use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;
    use crate::{decompressed_tree_store::CompletePostOrder, tests::examples::example_simple};

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

    #[test]
    fn test_bottom_up_matcher() {
        // Setup test trees with similar structure and minimal differences
        // Source tree: a -> [e -> [f], b -> [c, d]]
        // Dest tree:   a -> [e -> [g], b -> [c, d]]
        // The only difference is 'f' vs 'g' under node 'e'
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

        // Create stores for the test trees
        let (stores, src, dst) = vpair_to_stores((src, dst));

        // Decompress the trees for testing
        let src_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &src);
        let dst_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &dst);

        // Initialize the mapping store
        let mut mappings = DefaultMappingStore::default();
        mappings.topit(src_arena.len(), dst_arena.len());

        // Get node references for pre-mapping
        let src_root = src_arena.root();
        let dst_root = dst_arena.root();

        // Get 'c' and 'd' nodes by path from root
        let src_node_c = src_arena.child(&src_root, &[1, 0]);
        let src_node_d = src_arena.child(&src_root, &[1, 1]);
        let dst_node_c = dst_arena.child(&dst_root, &[1, 0]);
        let dst_node_d = dst_arena.child(&dst_root, &[1, 1]);

        // Pre-map the 'c' and 'd' nodes
        mappings.link(src_node_c, dst_node_c);
        mappings.link(src_node_d, dst_node_d);

        // Create the mapper with initial configuration
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };

        // Run the bottom-up matcher
        let result = BottomUpMatcher::match_it(mapping);

        // Verify at least 4 mappings were created (2 pre-mapped + at least 2 more)
        let mapping_count = result.mappings.len();
        assert!(
            mapping_count == 4,
            "Expected exactly 4 mappings, got {}",
            mapping_count
        );

        // Get references to important nodes in the result
        let src_root = result.mapping.src_arena.root();
        let dst_root = result.mapping.dst_arena.root();

        // Verify root nodes are mapped
        assert!(
            result.mapping.mappings.has(&src_root, &dst_root),
            "Root nodes should be mapped"
        );

        // Get children of root nodes
        let src_children = result.mapping.src_arena.children(&src_root);
        let dst_children = result.mapping.dst_arena.children(&dst_root);

        // Verif 'b' nodes are mapped

        assert!(
            result
                .mapping
                .mappings
                .has(&src_children[1], &dst_children[1]),
            "The 'b' nodes should be mapped"
        );

        // Get children of 'b' node
        let src_b_children = result.mapping.src_arena.children(&src_children[1]);
        let dst_b_children = result.mapping.dst_arena.children(&dst_children[1]);

        // Verify children of 'b' ('c' and 'd') are mapped correctly
        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[0], &dst_b_children[0]),
            "The 'c' nodes should be mapped"
        );
        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[1], &dst_b_children[1]),
            "The 'd' nodes should be mapped"
        );
    }
}
