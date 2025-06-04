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
            if children_processed {
                // This node's children have all been processed, so we can yield it

                // Check if this matches our custom leaf predicate
                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);

                if is_custom_leaf {
                    if self.config.yield_leaves {
                        return Some(node);
                    } else {
                        continue;
                    }
                } else {
                    if self.config.yield_inner {
                        return Some(node);
                    } else {
                        continue;
                    }
                }
            } else {
                // This node's children haven't been processed yet

                // Check if this matches our custom leaf predicate
                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);

                if is_custom_leaf {
                    // Custom leaf - don't process children, yield immediately if configured
                    if self.config.yield_leaves {
                        return Some(node);
                    } else {
                        continue;
                    }
                } else {
                    // Not a custom leaf - check if it has actual children
                    let children = self.arena.decompress_children(&node);

                    if children.is_empty() {
                        // No children - this is a regular leaf
                        if self.config.yield_inner {
                            return Some(node);
                        } else {
                            continue;
                        }
                    } else {
                        // Has children - push back with children_processed=true, then push children
                        self.stack.push((node, true));

                        // Push children in reverse order so they're processed left-to-right
                        for child in children.into_iter().rev() {
                            self.stack.push((child, false));
                        }
                        continue;
                    }
                }
            }
        }

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
    fn extract_labels<'a>(nodes: &'a [u16], stores: &'a Store) -> Vec<&'a str> {
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

    macro_rules! iterator_test {
        (
            $test_name:ident,
            tree = $tree:expr,
            config = $config:expr,
            is_leaf_fn = $is_leaf_fn:expr,
            expected = $expected:expr
        ) => {
            #[test]
            fn $test_name() {
                let (stores, src, _dst) = vpair_to_stores(($tree, tree!(0, "")));

                let mut src_arena =
                    Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
                let mut src_arena_mut = src_arena.as_mut();

                let root = src_arena_mut.root();

                let config = $config;

                let is_leaf_fn = $is_leaf_fn;

                let iterator = CustomPostOrderIterator::new(
                    &mut src_arena_mut,
                    &stores,
                    root,
                    config,
                    is_leaf_fn,
                );

                let nodes: Vec<_> = iterator.collect();

                assert_labels(&nodes, &stores, $expected);
            }
        };
    }

    fn no_children(
        arena: &mut Decompressible<&Store, &mut LazyPostOrder<u16, u16>>,
        _stores: &Store,
        node: &u16,
    ) -> bool {
        arena.decompress_children(node).is_empty()
    }

    fn always_false(
        _arena: &mut Decompressible<&Store, &mut LazyPostOrder<u16, u16>>,
        _stores: &Store,
        _node: &u16,
    ) -> bool {
        false
    }

    fn always_true(
        _arena: &mut Decompressible<&Store, &mut LazyPostOrder<u16, u16>>,
        _stores: &Store,
        _node: &u16,
    ) -> bool {
        true
    }

    fn label_statement(
        arena: &mut Decompressible<&Store, &mut LazyPostOrder<u16, u16>>,
        stores: &Store,
        node: &u16,
    ) -> bool {
        let original = arena.original(node);
        let node_ref = stores.node_store.resolve(&original);
        let label_id = node_ref.get_label_unchecked();
        let label = stores.label_store.resolve(&label_id);
        label.starts_with("statement")
    }

    // Example usage:
    iterator_test!(
        test_iterator_default_config,
        tree = tree!(
            0, "root"; [
                tree!(0, "l"; [
                    tree!(1, "l.l"),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"),
            ]
        ),
        config = CustomIteratorConfig::default(),
        is_leaf_fn = no_children,
        expected = &["l.l", "l.r", "l", "r", "root"]
    );

    iterator_test!(
        test_iterator_leaves_only,
        tree = tree!(
            0, "root"; [
                tree!(0, "l"; [
                    tree!(1, "l.l"),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false
        },
        is_leaf_fn = no_children,
        expected = &["l.l", "l.r", "r"]
    );

    iterator_test!(
        test_iterator_custom_leaves,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"),
                    tree!(0, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"),
                    tree!(0, "r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false
        },
        is_leaf_fn = label_statement,
        expected = &["statement_l", "statement_r.l"]
    );

    iterator_test!(
        test_iterator_nested_statements_only_highest,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"; [
                        tree!(0, "l.l.l"; [
                            tree!(1, "l.l.l.l"),
                            tree!(1, "l.l.l.r"),
                        ]),
                        tree!(1, "l.l.r"),
                    ]),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"; [
                        tree!(1, "r.l.l"),
                        tree!(1, "r.l.r"),
                    ]),
                    tree!(1, "r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false
        },
        is_leaf_fn = label_statement,
        expected = &["statement_l", "statement_r.l"]
    );
}
