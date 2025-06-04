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

use std::fmt::Display;

use hyperast::types::{HyperAST, HyperType, NodeId, TypeStore, WithHashs};

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
    stack: Vec<(IdD, bool)>, // (node, children_processed)
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
            stack: vec![(root, false)],
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
    D::IdD: std::fmt::Debug,
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
        while let Some((node, children_processed)) = self.stack.pop() {
            println!("=== Iterator loop start ===");
            println!(
                "Processing node: {:?}, children_processed: {}, stack: {:?}",
                node, children_processed, self.stack
            );

            if children_processed {
                // This node's children have all been processed, so we can yield it
                println!(
                    "Children already processed, checking if should yield: {:?}",
                    node
                );

                // Check if this matches our custom leaf predicate
                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);
                println!("is_custom_leaf: {}", is_custom_leaf);

                if is_custom_leaf {
                    if self.config.yield_leaves {
                        println!("Yielding custom leaf node: {:?}", node);
                        return Some(node);
                    } else {
                        println!("Skipping custom leaf node (yield_leaves=false)");
                        continue;
                    }
                } else {
                    if self.config.yield_inner {
                        println!("Yielding inner node: {:?}", node);
                        return Some(node);
                    } else {
                        println!("Skipping inner node (yield_inner=false)");
                        continue;
                    }
                }
            } else {
                // This node's children haven't been processed yet
                println!("Children not yet processed for node: {:?}", node);

                // Check if this matches our custom leaf predicate
                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);
                println!("is_custom_leaf: {}", is_custom_leaf);

                if is_custom_leaf {
                    // Custom leaf - don't process children, yield immediately if configured
                    println!("Node is custom leaf, not processing children");
                    if self.config.yield_leaves {
                        println!("Yielding custom leaf node: {:?}", node);
                        return Some(node);
                    } else {
                        println!("Skipping custom leaf node (yield_leaves=false)");
                        continue;
                    }
                } else {
                    // Not a custom leaf - check if it has actual children
                    let children = self.arena.decompress_children(&node);
                    println!(
                        "Node has {} actual children: {:?}",
                        children.len(),
                        children
                    );

                    if children.is_empty() {
                        // No children - this is a regular leaf
                        println!("Node has no children (regular leaf)");
                        if self.config.yield_inner {
                            println!("Yielding regular leaf as inner node: {:?}", node);
                            return Some(node);
                        } else {
                            println!("Skipping regular leaf (yield_inner=false)");
                            continue;
                        }
                    } else {
                        // Has children - push back with children_processed=true, then push children
                        println!("Node has children, pushing back with children_processed=true");
                        self.stack.push((node, true));

                        // Push children in reverse order so they're processed left-to-right
                        println!("Pushing children to stack: {:?}", children);
                        for child in children.into_iter().rev() {
                            self.stack.push((child, false));
                        }
                        continue;
                    }
                }
            }
        }

        println!("Stack is empty, returning None");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::matchers::Decompressible;
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;

    use hyperast::{
        store::SimpleStores,
        test_utils::simple_tree::{LS, NS, TStore, Tree},
        types::{DecompressedFrom, LabelStore, Labeled, NodeStore},
    };

    type Store = SimpleStores<TStore, NS<Tree>, LS<u16>>;

    /// Helper function to extract labels from iterator results in order
    fn extract_labels<'a>(nodes: &'a [u16], stores: &'a Store) -> Vec<&'a str>
// where
    //     IdD: Clone,
    {
        nodes
            .iter()
            .map(|node| {
                let n = stores.node_store.resolve(node);
                let l_id = n.get_label_unchecked();
                stores.label_store.resolve(l_id)
            })
            .collect()
    }

    /// Helper function to assert the order of labels from iterator results
    fn assert_labels(nodes: &[u16], stores: &Store, expected: &[&str]) {
        let actual_labels = extract_labels(nodes, stores);
        assert_eq!(
            actual_labels, expected,
            "Expected labels {:?}, but got {:?}",
            expected, actual_labels
        );
    }

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
            tree!(0, "a"),
        ));

        // Tree in post order: d, e, b, c, a
        //

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

        println!("Nodes: {:?}", nodes);

        // Print labels for debugging
        let labels = extract_labels(&nodes, &stores);
        println!("Labels: {:?}", labels);

        // Should visit all nodes in post-order: d, e, b, c, a
        assert_eq!(nodes.len(), 5);
        assert_labels(&nodes, &stores, &["d", "e", "b", "c", "a"]);

        // The root should be the last element in post-order traversal
        let last_node = nodes.last().unwrap();
        assert_eq!(*last_node, root);
    }

    #[test]
    fn test_iterator_only_leaves() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "a"; [
                    tree!(1, "b"; [
                        tree!(2, "d"),
                        tree!(2, "e"),
                    ]),
                    tree!(1, "c"),
            ]),
            tree!(0, "a"),
        ));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        // Test with config to yield only leaves
        let config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false,
        };

        // Use actual leaves predicate - nodes with no children
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should only visit leaf nodes: d, e, c
        assert_eq!(nodes.len(), 3);
        assert_labels(&nodes, &stores, &["d", "e", "c"]);

        // Verify all returned nodes are actually leaves
        for node in &nodes {
            let children = src_arena_mut.decompress_children(node);
            assert!(children.is_empty(), "Only leaf nodes should be yielded");
        }
    }

    #[test]
    fn test_iterator_only_inner_nodes() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "a"; [
                    tree!(1, "b"; [
                        tree!(2, "d"),
                        tree!(2, "e"),
                    ]),
                    tree!(1, "c"),
            ]),
            tree!(0, "a"),
        ));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        // Test with config to yield only inner nodes
        let config = CustomIteratorConfig {
            yield_leaves: false,
            yield_inner: true,
        };

        // Use actual leaves predicate - nodes with no children
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should only visit inner nodes: b, a (post-order)
        assert_eq!(nodes.len(), 2);
        assert_labels(&nodes, &stores, &["b", "a"]);

        // Verify all returned nodes have children
        for node in &nodes {
            let children = src_arena_mut.decompress_children(node);
            assert!(!children.is_empty(), "Only inner nodes should be yielded");
        }
    }

    #[test]
    fn test_iterator_custom_leaf_predicate() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "a"; [
                    tree!(1, "b"; [
                        tree!(2, "d"),
                        tree!(2, "e"),
                    ]),
                    tree!(1, "c"),
            ]),
            tree!(0, "a"),
        ));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        let config = CustomIteratorConfig::default();

        // Custom predicate: consider nodes with label "b" as leaves (don't traverse their children)
        let is_leaf_fn = |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>,
                          stores: &Store,
                          node: &_| {
            let original = arena.original(node);
            let node_ref = stores.node_store.resolve(&original);
            let label_id = node_ref.get_label_unchecked();
            let label = stores.label_store.resolve(&label_id);
            label == "b"
        };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should visit: b (treated as leaf, so d,e are not visited), c, a
        assert_eq!(nodes.len(), 3);
        assert_labels(&nodes, &stores, &["b", "c", "a"]);
    }

    #[test]
    fn test_iterator_single_node_tree() {
        let (stores, src, _dst) = vpair_to_stores((tree!(0, "single"), tree!(0, "single")));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        let config = CustomIteratorConfig::default();

        // Use actual leaves predicate
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should visit exactly one node (the root, which is also a leaf)
        assert_eq!(nodes.len(), 1);
        assert_labels(&nodes, &stores, &["single"]);
        assert_eq!(nodes[0], root);
    }

    #[test]
    fn test_iterator_deeper_tree() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "root"; [
                    tree!(1, "left"; [
                        tree!(2, "left_left"),
                        tree!(2, "left_right"),
                    ]),
                    tree!(1, "middle"),
                    tree!(1, "right"; [
                        tree!(2, "right_left"),
                        tree!(2, "right_right"; [
                            tree!(3, "deep"),
                        ]),
                    ]),
            ]),
            tree!(0, "root"),
        ));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        let config = CustomIteratorConfig::default();

        // Use actual leaves predicate
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should visit nodes in post-order
        assert!(nodes.len() > 1);
        assert_labels(
            &nodes,
            &stores,
            &[
                "left_left",
                "left_right",
                "left",
                "middle",
                "right_left",
                "deep",
                "right_right",
                "right",
                "root",
            ],
        );

        // The root should be the last element in post-order traversal
        let last_node = nodes.last().unwrap();
        assert_eq!(*last_node, root);
    }

    #[test]
    fn test_iterator_yield_nothing() {
        let (stores, src, _dst) = vpair_to_stores((
            tree!(
                0, "a"; [
                    tree!(1, "b"; [
                        tree!(2, "d"),
                        tree!(2, "e"),
                    ]),
                    tree!(1, "c"),
            ]),
            tree!(0, "a"),
        ));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        // Test with config to yield nothing
        let config = CustomIteratorConfig {
            yield_leaves: false,
            yield_inner: false,
        };

        // Use actual leaves predicate
        let is_leaf_fn =
            |arena: &mut Decompressible<_, &mut LazyPostOrder<u16, u16>>, _stores, node: &_| {
                arena.decompress_children(node).is_empty()
            };

        let iterator =
            CustomPostOrderIterator::new(&mut src_arena_mut, &stores, root, config, is_leaf_fn);

        let nodes: Vec<_> = iterator.collect();

        // Should visit no nodes
        assert!(nodes.is_empty());
    }

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
