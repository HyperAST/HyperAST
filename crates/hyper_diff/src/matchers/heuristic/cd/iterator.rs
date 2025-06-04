//! # Custom Post-Order Iterator for Coarse-Grained (Statement-Level) Traversal
//!
//! This module provides a configurable post-order iterator for traversing ASTs
//! in a way that mimics the coarse-grained, statement-level analysis of the original
//! ChangeDistiller algorithm, while operating over a fine-grained HyperAST.
//!
//! ## Motivation
//!
//! The original ChangeDistiller algorithm analyzes code at the statement level, treating
//! statements as atomic units and ignoring their internal structure. In contrast, HyperAST
//! preserves the complete fine-grained AST, including all syntactic elements. To bridge
//! these approaches, this iterator allows certain nodes (e.g., statement nodes) to be
//! treated as logical leaves, regardless of their actual children, enabling coarse-grained
//! analysis atop a fine-grained tree.
//!
//! ## Features
//! - **Configurable Leaf Detection:** A predicate function determines whether a node is
//!   considered a logical leaf (e.g., a statement node), allowing flexible adaptation to
//!   different analysis granularities.
//! - **Hierarchical Traversal:** Performs post-order traversal, but stops descent at
//!   logical leaves as defined by the predicate.
//! - **Flexible Iteration Modes:** Configurable to yield all nodes, only logical leaves,
//!   or only inner nodes, supporting both leaves and bottom-up matching phases efficiently.
//!
//! For more details and diagrams, see the associated paper section on coarse-grained
//! statement-level implementation.

use hyperast::types::{HyperAST, HyperType, TypeStore, WithHashs};

use crate::decompressed_tree_store::LazyDecompressedTreeStore;

/// Configuration for the custom post-order iterator.
///
/// Controls whether the iterator yields logical leaves, inner nodes, or both.
/// This enables flexible traversal strategies for different matching phases.

#[derive(Debug, Clone, Copy)]
pub struct CustomIteratorConfig {
    /// Whether to yield nodes that match the leaf predicate
    pub yield_leaves: bool,
    /// Whether to yield nodes that don't match the leaf predicate (inner nodes)
    pub yield_inner: bool,
}

impl Default for CustomIteratorConfig {
    fn default() -> Self {
        Self {
            yield_leaves: true,
            yield_inner: true,
        }
    }
}

/// Generic iterator for traversing nodes in post-order with custom leaf predicate
pub struct CustomPostOrderIterator<'a, D, HAST, IdS, IdD, F> {
    arena: &'a mut D,
    stores: HAST,
    current: Option<IdD>,
    to_traverse: Vec<IdD>,
    red: bool,
    config: CustomIteratorConfig,
    is_leaf_fn: F,
    _phantom: std::marker::PhantomData<IdS>,
}

/// Custom post-order iterator for AST traversal with logical leaf detection.
///
/// This iterator traverses the tree in post-order, but uses a user-provided predicate
/// to determine which nodes should be treated as logical leaves (e.g., statement nodes).
/// When a node matches the predicate, its children are not traversed, mimicking
/// coarse-grained statement-level analysis while preserving the underlying fine-grained structure.
///
/// The iterator can be configured to yield all nodes, only logical leaves, or only inner nodes.
impl<'a, D, HAST, IdD, F> CustomPostOrderIterator<'a, D, HAST, IdD, D::IdD, F>
where
    D: LazyDecompressedTreeStore<HAST, IdD>,
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    <HAST::TS as TypeStore>::Ty: HyperType,
    F: Fn(&mut D, HAST, &D::IdD) -> bool,
{
    /// Create a new custom iterator
    pub fn new(
        arena: &'a mut D,
        stores: HAST,
        root: D::IdD,
        config: CustomIteratorConfig,
        is_leaf_fn: F,
    ) -> Self {
        Self {
            arena,
            stores,
            current: Some(root),
            to_traverse: vec![],
            red: false,
            config,
            is_leaf_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, D, HAST, IdD, F> Iterator for CustomPostOrderIterator<'a, D, HAST, IdD, D::IdD, F>
where
    D: LazyDecompressedTreeStore<HAST, IdD>,
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    <HAST::TS as TypeStore>::Ty: HyperType,
    F: Fn(&mut D, HAST, &D::IdD) -> bool,
{
    type Item = D::IdD;

    /// Creates a new custom post-order iterator.
    ///
    /// - `arena`: The decompressed tree store to traverse.
    /// - `stores`: The HyperAST stores.
    /// - `root`: The root node to start traversal from.
    /// - `config`: Iterator configuration (which nodes to yield).
    /// - `is_leaf_fn`: Predicate function to determine logical leaves.
    ///
    /// The iterator will traverse the tree in post-order, treating nodes for which
    /// `is_leaf_fn` returns true as logical leaves (i.e., their children are not traversed).
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.red {
                let Some(sib) = self.to_traverse.pop() else {
                    return None;
                };
                self.current = Some(sib);
                self.red = false;
            }

            let Some(current) = self.current.take() else {
                return None;
            };

            // Check if this matches our custom leaf predicate
            // Pass arena, stores, and node to the predicate function
            let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &current);

            if is_custom_leaf {
                self.red = true;
                if self.config.yield_leaves {
                    return Some(current);
                } else {
                    continue; // Skip leaf nodes
                }
            }

            // Rest of the implementation stays the same...
            let mut children = self.arena.decompress_children(&current);
            if children.is_empty() {
                self.red = true;
                if self.config.yield_inner {
                    return Some(current);
                } else {
                    continue;
                }
            }

            let result = current;
            children.reverse();
            self.current = children.pop();
            self.to_traverse.extend(children);

            if self.config.yield_inner {
                return Some(result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::matchers::Decompressible;
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;

    use hyperast::types::{DecompressedFrom, LabelStore, Labeled, NodeStore};

    #[test]
    fn test_iterator_default_config() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "a"; [
                    tree!(1, "b"; [
                        tree!(2, "d"),
                        tree!(2, "e"),
                    ]),
                    tree!(1, "c"),
            ]),
            tree!(
                0, "a"; [
                    tree!(1, "c"),
                    tree!(1, "b"; [
                        tree!(2, "e"),
                        tree!(2, "d"),
                    ]),
            ]),
        ));
        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        // Test with default config (yield both leaves and inner nodes)
        let config = CustomIteratorConfig::default();

        // Use actual leaves predicate - nodes with no children
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Print labels
        println!(
            "Labels: {:?}",
            nodes
                .iter()
                .map(|node| {
                    let n = stores.node_store.resolve(node);
                    let l_id = n.get_label_unchecked();
                    stores.label_store.resolve(l_id)
                })
                .collect::<Vec<_>>()
        );

        // Should visit all nodes in post-order
        assert_eq!(nodes.len(), 3);

        println!("Nodes: {:?}", nodes);

        // The root should be the last element in post-order traversal
        let last_node = nodes.last().unwrap();
        assert_eq!(*last_node, root);
    }

    // #[test]
    // fn test_iterator_only_leaves() {
    //     let (stores, src, _dst) =
    //         vpair_to_stores(crate::tests::examples::example_leaf_label_swap());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     // Test with config to yield only leaves
    //     let config = CustomIteratorConfig {
    //         yield_leaves: true,
    //         yield_inner: false,
    //     };

    //     // Use actual leaves predicate - nodes with no children
    //     let is_leaf_fn =
    //         |arena: &mut _, _stores, node: &_| arena.decompress_children(node).is_empty();

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should only visit leaf nodes
    //     for node in &nodes {
    //         let children = src_arena.decompress_children(node);
    //         assert!(children.is_empty(), "Only leaf nodes should be yielded");
    //     }

    //     // Should have 2 leaf nodes for the example tree (children "b" and "c")
    //     assert_eq!(nodes.len(), 2);
    // }

    // #[test]
    // fn test_iterator_only_inner_nodes() {
    //     let (stores, src, _dst) =
    //         vpair_to_stores(crate::tests::examples::example_leaf_label_swap());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     // Test with config to yield only inner nodes
    //     let config = CustomIteratorConfig {
    //         yield_leaves: false,
    //         yield_inner: true,
    //     };

    //     // Use actual leaves predicate - nodes with no children
    //     let is_leaf_fn =
    //         |arena: &mut _, _stores, node: &_| arena.decompress_children(node).is_empty();

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should only visit inner nodes (nodes with children)
    //     for node in &nodes {
    //         let children = src_arena.decompress_children(node);
    //         assert!(!children.is_empty(), "Only inner nodes should be yielded");
    //     }

    //     // Should have 1 inner node for the example tree (root "a")
    //     assert_eq!(nodes.len(), 1);
    //     assert_eq!(nodes[0], root);
    // }

    // #[test]
    // fn test_iterator_custom_leaf_predicate() {
    //     let (stores, src, _dst) =
    //         vpair_to_stores(crate::tests::examples::example_leaf_label_swap());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     // Test with custom leaf predicate based on label
    //     let config = CustomIteratorConfig::default();

    //     // Custom predicate: consider nodes with label "b" as leaves
    //     let is_leaf_fn = |arena: &mut _, stores, node: &_| {
    //         let original = arena.original(node);
    //         let node_ref = stores.node_store().resolve(&original);
    //         if let Some(label_id) = node_ref.try_get_label() {
    //             let label = stores.label_store().resolve(&label_id);
    //             label == "b"
    //         } else {
    //             false
    //         }
    //     };

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should visit all nodes, but the one with label "b" should be treated as a leaf
    //     assert!(!nodes.is_empty());

    //     // Verify that nodes with label "b" are treated as leaves by the predicate
    //     for node in &nodes {
    //         let original = src_arena.original(node);
    //         let node_ref = stores.node_store().resolve(&original);
    //         if let Some(label_id) = node_ref.try_get_label() {
    //             let label = stores.label_store().resolve(&label_id);
    //             if label == "b" {
    //                 // This node should have been treated as a leaf by our predicate
    //                 // We can't directly test the predicate behavior here, but we know it was called
    //                 break;
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_iterator_single_node_tree() {
    //     let (stores, src, _dst) = vpair_to_stores(crate::tests::examples::example_single());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     let config = CustomIteratorConfig::default();

    //     // Use actual leaves predicate
    //     let is_leaf_fn =
    //         |arena: &mut _, _stores, node: &_| arena.decompress_children(node).is_empty();

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should visit exactly one node (the root, which is also a leaf)
    //     assert_eq!(nodes.len(), 1);
    //     assert_eq!(nodes[0], root);
    // }

    // #[test]
    // fn test_iterator_deeper_tree() {
    //     let (stores, src, _dst) = vpair_to_stores(crate::tests::examples::example_gt_slides());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     let config = CustomIteratorConfig::default();

    //     // Use actual leaves predicate
    //     let is_leaf_fn =
    //         |arena: &mut _, _stores, node: &_| arena.decompress_children(node).is_empty();

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should visit multiple nodes in post-order
    //     assert!(nodes.len() > 1);

    //     // The root should be the last element in post-order traversal
    //     let last_node = nodes.last().unwrap();
    //     assert_eq!(*last_node, root);
    // }

    // #[test]
    // fn test_iterator_with_type_based_predicate() {
    //     let (stores, src, _dst) = vpair_to_stores(crate::tests::examples::example_leaf_swap());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     let config = CustomIteratorConfig::default();

    //     // Custom predicate: consider nodes of type 4 as leaves
    //     let is_leaf_fn = |arena: &mut _, stores, node: &_| {
    //         let original = arena.original(node);
    //         let node_type = stores.resolve_type(&original);
    //         node_type == 4
    //     };

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should visit all nodes
    //     assert!(!nodes.is_empty());

    //     // Verify that at least one node of type 4 exists and was handled
    //     let has_type_4 = nodes.iter().any(|node| {
    //         let original = src_arena.original(node);
    //         let node_type = stores.resolve_type(&original);
    //         node_type == 4
    //     });
    //     assert!(has_type_4, "Should have at least one node of type 4");
    // }

    // #[test]
    // fn test_iterator_yield_nothing() {
    //     let (stores, src, _dst) =
    //         vpair_to_stores(crate::tests::examples::example_leaf_label_swap());
    //     let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);

    //     let root = src_arena.root();

    //     // Test with config to yield nothing
    //     let config = CustomIteratorConfig {
    //         yield_leaves: false,
    //         yield_inner: false,
    //     };

    //     // Use actual leaves predicate
    //     let is_leaf_fn =
    //         |arena: &mut _, _stores, node: &_| arena.decompress_children(node).is_empty();

    //     let iterator =
    //         CustomPostOrderIterator::new(&mut src_arena, stores, root, config, is_leaf_fn);

    //     let nodes: Vec<_> = iterator.collect();

    //     // Should visit no nodes
    //     assert!(nodes.is_empty());
    // }
}
