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

use crate::decompressed_tree_store::{DecompressedTreeStore, LazyDecompressedTreeStore};

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
    /// Whether to yield only the deepest logical leaves (no logical leaves as descendants)
    pub deepest_leaf: bool,
}

impl CustomIteratorConfig {
    pub fn shallow_leaves() -> Self {
        Self {
            yield_leaves: true,
            yield_inner: false,
            deepest_leaf: false,
        }
    }

    pub fn deep_leaves() -> Self {
        Self {
            yield_leaves: true,
            yield_inner: false,
            deepest_leaf: true,
        }
    }

    pub fn leaves(deep: bool) -> Self {
        Self {
            yield_leaves: true,
            yield_inner: false,
            deepest_leaf: deep,
        }
    }

    pub fn shallow_inner() -> Self {
        Self {
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: false,
        }
    }

    pub fn deep_inner() -> Self {
        Self {
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: true,
        }
    }

    pub fn inner(deep: bool) -> Self {
        Self {
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: deep,
        }
    }
}

impl Default for CustomIteratorConfig {
    fn default() -> Self {
        Self {
            yield_leaves: true,
            yield_inner: true,
            deepest_leaf: false,
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
    D::IdD: std::fmt::Debug + Clone,
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

                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);

                if is_custom_leaf {
                    // For logical leaves that had their children processed,
                    // we only yield if deepest_leaf is false
                    if self.config.yield_leaves && !self.config.deepest_leaf {
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

                if is_custom_leaf && self.config.deepest_leaf {
                    // When looking for deepest logical leaves, we need to check if this
                    // logical leaf contains other logical leaves
                    let children = self.arena.decompress_children(&node);

                    if children.is_empty() {
                        // No children - this is definitely a deepest logical leaf
                        if self.config.yield_leaves {
                            return Some(node);
                        } else {
                            continue;
                        }
                    } else {
                        // Has children - need to check if any are logical leaves
                        let mut has_logical_leaf_descendant = false;
                        let mut to_check = children.clone();

                        while let Some(child) = to_check.pop() {
                            if (self.is_leaf_fn)(self.arena, self.stores, &child) {
                                has_logical_leaf_descendant = true;
                                break;
                            }
                            let child_children = self.arena.decompress_children(&child);
                            to_check.extend(child_children);
                        }

                        if has_logical_leaf_descendant {
                            // This logical leaf contains other logical leaves, so traverse its children
                            self.stack.push((node, true));
                            for child in children.into_iter().rev() {
                                self.stack.push((child, false));
                            }
                            continue;
                        } else {
                            // No logical leaf descendants - this is a deepest logical leaf
                            if self.config.yield_leaves {
                                return Some(node);
                            } else {
                                continue;
                            }
                        }
                    }
                } else if is_custom_leaf {
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

/// Iterator for fully decompressed trees (no lazy decompression needed)
pub struct DecompressedCustomPostOrderIterator<'a, D, HAST, IdD, F> {
    arena: &'a D,
    stores: HAST,
    stack: Vec<(IdD, bool)>, // (node, children_processed)
    config: CustomIteratorConfig,
    is_leaf_fn: F,
}

/// Custom post-order iterator for fully decompressed AST traversal with logical leaf detection.
///
/// This iterator is similar to `CustomPostOrderIterator` but works with fully decompressed trees
/// (`DecompressedTreeStore`) instead of lazy ones (`LazyDecompressedTreeStore`). Since the tree
/// is already fully decompressed, it only needs immutable access to the arena.
impl<'a, D, HAST, IdD, F> DecompressedCustomPostOrderIterator<'a, D, HAST, IdD, F>
where
    D: DecompressedTreeStore<HAST, IdD>,
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    <HAST::TS as TypeStore>::Ty: HyperType,
    F: Fn(&D, HAST, &IdD) -> bool,
{
    /// Create a new iterator for a fully decompressed tree
    pub fn new(
        arena: &'a D,
        stores: HAST,
        root: IdD,
        config: CustomIteratorConfig,
        is_leaf_fn: F,
    ) -> Self {
        Self {
            arena,
            stores,
            stack: vec![(root, false)],
            config,
            is_leaf_fn,
        }
    }
}

impl<'a, D, HAST, IdD, F> Iterator for DecompressedCustomPostOrderIterator<'a, D, HAST, IdD, F>
where
    D: DecompressedTreeStore<HAST, IdD>,
    HAST: HyperAST + Copy,
    IdD: std::fmt::Debug + Clone,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    <HAST::TS as TypeStore>::Ty: HyperType,
    F: Fn(&D, HAST, &IdD) -> bool,
{
    type Item = IdD;

    /// Creates a new custom post-order iterator for fully decompressed trees.
    ///
    /// - `arena`: The decompressed tree store to traverse (immutable access).
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

                let is_custom_leaf = (self.is_leaf_fn)(self.arena, self.stores, &node);

                if is_custom_leaf {
                    // For logical leaves that had their children processed,
                    // we only yield if deepest_leaf is false
                    if self.config.yield_leaves && !self.config.deepest_leaf {
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

                if is_custom_leaf && self.config.deepest_leaf {
                    // When looking for deepest logical leaves, we need to check if this
                    // logical leaf contains other logical leaves
                    let children = self.arena.children(&node);

                    if children.is_empty() {
                        // No children - this is definitely a deepest logical leaf
                        if self.config.yield_leaves {
                            return Some(node);
                        } else {
                            continue;
                        }
                    } else {
                        // Has children - need to check if any are logical leaves
                        let mut has_logical_leaf_descendant = false;
                        let mut to_check = children.clone();

                        while let Some(child) = to_check.pop() {
                            if (self.is_leaf_fn)(self.arena, self.stores, &child) {
                                has_logical_leaf_descendant = true;
                                break;
                            }
                            let child_children = self.arena.children(&child);
                            to_check.extend(child_children);
                        }

                        if has_logical_leaf_descendant {
                            // This logical leaf contains other logical leaves, so traverse its children
                            self.stack.push((node, true));
                            for child in children.into_iter().rev() {
                                self.stack.push((child, false));
                            }
                            continue;
                        } else {
                            // No logical leaf descendants - this is a deepest logical leaf
                            if self.config.yield_leaves {
                                return Some(node);
                            } else {
                                continue;
                            }
                        }
                    }
                } else if is_custom_leaf {
                    // Custom leaf - don't process children, yield immediately if configured
                    if self.config.yield_leaves {
                        return Some(node);
                    } else {
                        continue;
                    }
                } else {
                    // Not a custom leaf - get children from the fully decompressed tree
                    let children = self.arena.children(&node);

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
    use crate::decompressed_tree_store::Shallow;
    use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
    use crate::decompressed_tree_store::{CompletePostOrder, lazy_post_order::LazyPostOrder};
    use crate::matchers::Decompressible;
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;
    use ahash::HashMap;
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
            $test_name_decomp:ident,
            tree = $tree:expr,
            config = $config:expr,
            is_leaf_fn = $is_leaf_fn:expr,
            is_leaf_fn_decomp = $is_leaf_fn_decomp:expr,
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

            #[test]
            fn $test_name_decomp() {
                let (stores, src, _dst) = vpair_to_stores(($tree, tree!(0, "")));

                let src_arena =
                    Decompressible::<_, CompletePostOrder<u16, u16>>::decompress(&stores, &src);

                let root = src_arena.root();

                let config = $config;

                let is_leaf_fn = $is_leaf_fn_decomp;

                let iterator = DecompressedCustomPostOrderIterator::new(
                    &src_arena, &stores, root, config, is_leaf_fn,
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

    fn decompressed_no_children(
        arena: &Decompressible<&Store, CompletePostOrder<u16, u16>>,
        _stores: &Store,
        node: &u16,
    ) -> bool {
        println!(
            "[DecompressedNoChildren] Node has {:?} children: {:?}",
            arena.descendants_count(node),
            arena.descendants(node)
        );
        arena.descendants(node).len() == 0
    }

    fn decompressed_label_statement(
        _arena: &Decompressible<&Store, CompletePostOrder<u16, u16>>,
        stores: &Store,
        node: &u16,
    ) -> bool {
        let node_ref = stores.node_store.resolve(&node);
        let label_id = node_ref.get_label_unchecked();
        let label = stores.label_store.resolve(&label_id);
        label.starts_with("statement")
    }

    // Example usage:
    iterator_test!(
        test_iterator_default_config,
        test_iterator_default_config_decomp,
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
        is_leaf_fn_decomp = decompressed_no_children,
        expected = &["l.l", "l.r", "l", "r", "root"]
    );

    iterator_test!(
        test_iterator_default_config_deepest,
        test_iterator_default_config_deepest_decomp,
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
            yield_inner: true,
            deepest_leaf: true
        },
        is_leaf_fn = no_children,
        is_leaf_fn_decomp = decompressed_no_children,
        expected = &["l.l", "l.r", "l", "r", "root"]
    );

    iterator_test!(
        test_iterator_leaves_only,
        test_iterator_leaves_only_decomp,
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
            yield_inner: false,
            deepest_leaf: false
        },
        is_leaf_fn = no_children,
        is_leaf_fn_decomp = decompressed_no_children,
        expected = &["l.l", "l.r", "r"]
    );

    iterator_test!(
        test_iterator_inner_only,
        test_iterator_inner_only_decomp,
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
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: false
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["r.r", "r", "root"]
    );

    iterator_test!(
        test_iterator_custom_leaves,
        test_iterator_custom_leaves_decomp,
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
            yield_inner: false,
            deepest_leaf: false
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["statement_l", "statement_r.l"]
    );
    iterator_test!(
        test_iterator_custom_leaves_deepest,
        test_iterator_custom_leaves_deepest_decomp,
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
            yield_inner: false,
            deepest_leaf: true
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["statement_l", "statement_r.l"]
    );

    iterator_test!(
        test_iterator_nested_statements_only_highest,
        test_iterator_nested_statements_only_highest_decomp,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"; [
                        tree!(0, "statement_l.l.l"; [
                            tree!(1, "l.l.l.l"),
                            tree!(1, "statement_l.l.l.r"),
                        ]),
                        tree!(1, "l.l.r"),
                    ]),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"; [
                        tree!(1, "r.l.l"),
                        tree!(1, "statement_r.l.r"),
                    ]),
                    tree!(1, "statement_r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false,
            deepest_leaf: false
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["statement_l", "statement_r.l", "statement_r.r"]
    );
    iterator_test!(
        test_iterator_nested_statements_inner,
        test_iterator_nested_statements_inner_decomp,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"; [
                        tree!(0, "statement_l.l.l"; [
                            tree!(1, "l.l.l.l"),
                            tree!(1, "statement_l.l.l.r"),
                        ]),
                        tree!(1, "l.l.r"),
                    ]),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"; [
                        tree!(1, "r.l.l"),
                        tree!(1, "statement_r.l.r"),
                    ]),
                    tree!(1, "statement_r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: false
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["r", "root"]
    );

    // Test with deepest_leaf = true (new behavior)
    iterator_test!(
        test_deepest_logical_leaves,
        test_deepest_logical_leaves_decomp,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"; [
                        tree!(0, "statement_l.l.l"; [
                            tree!(1, "l.l.l.l"),
                            tree!(1, "statement_l.l.l.r"),
                        ]),
                        tree!(1, "l.l.r"),
                    ]),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"; [
                        tree!(1, "r.l.l"),
                        tree!(1, "statement_r.l.r"),
                    ]),
                    tree!(1, "r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: true,
            yield_inner: false,
            deepest_leaf: true,
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &["statement_l.l.l.r", "statement_r.l.r"]
    );

    // Test with deepest_leaf = true (new behavior)
    iterator_test!(
        test_deepest_logical_leaves_inner,
        test_deepest_logical_leaves_inner_decomp,
        tree = tree!(
            0, "root"; [
                tree!(0, "statement_l"; [
                    tree!(0, "l.l"; [
                        tree!(0, "statement_l.l.l"; [
                            tree!(1, "l.l.l.l"),
                            tree!(1, "statement_l.l.l.r"),
                        ]),
                        tree!(1, "l.l.r"),
                    ]),
                    tree!(1, "l.r"),
                ]),
                tree!(0, "r"; [
                    tree!(0, "statement_r.l"; [
                        tree!(1, "r.l.l"),
                        tree!(1, "statement_r.l.r"),
                    ]),
                    tree!(1, "r.r"),
                ]),
            ]
        ),
        config = CustomIteratorConfig {
            yield_leaves: false,
            yield_inner: true,
            deepest_leaf: true,
        },
        is_leaf_fn = label_statement,
        is_leaf_fn_decomp = decompressed_label_statement,
        expected = &[
            "l.l.l.l",
            "statement_l.l.l",
            "l.l.r",
            "l.l",
            "statement_l",
            "r.l.l",
            "statement_r.l",
            "r.r",
            "r",
            "root"
        ]
    );

    #[test]
    fn test_custom_leaves_count() {
        let tree = tree!(
        0, "root"; [                            // 12
            tree!(0, "statement_l"; [           // 6
                tree!(0, "l.l"; [               // 4
                    tree!(0, "l.l.l"; [         // 2
                        tree!(1, "l.l.l.l"),    // 0
                        tree!(1, "l.l.l.r"),    // 1
                    ]),
                    tree!(1, "l.l.r"),          // 3
                ]),
                tree!(1, "l.r"),                // 5
            ]),
            tree!(0, "r"; [                     // 11
                tree!(0, "statement_r.l"; [     // 9
                    tree!(1, "r.l.l"),          // 7
                    tree!(1, "r.l.r"),          // 8
                ]),
                tree!(1, "r.r"),                // 10
            ]),
        ]
        );

        let (stores, src, _dst) = vpair_to_stores((tree, tree!(0, "")));

        let mut src_arena = Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(&stores, &src);
        let mut src_arena_mut = src_arena.as_mut();

        let root = src_arena_mut.root();

        let src_leaves = CustomPostOrderIterator::new(
            &mut src_arena_mut,
            &stores,
            root,
            CustomIteratorConfig {
                yield_leaves: true,
                yield_inner: false,
                deepest_leaf: false,
            },
            label_statement,
        )
        .collect::<Vec<_>>();

        let src_nodes = CustomPostOrderIterator::new(
            &mut src_arena_mut,
            &stores,
            root,
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
                deepest_leaf: false,
            },
            label_statement,
        )
        .collect::<Vec<_>>();

        let mut leaf_counts: HashMap<u16, usize> = HashMap::default();

        for src in &src_leaves {
            leaf_counts.insert(*src.shallow(), 1);
        }

        // Process nodes in post-order so children are processed before parents
        for src in &src_nodes {
            let leaf_count = src_arena_mut
                .children(&src)
                .iter()
                .map(|child| leaf_counts.get(child).copied().unwrap_or(0))
                .sum();

            leaf_counts.insert(*src.shallow(), leaf_count);
        }

        println!("Leaf counts: {:?}", leaf_counts);
        assert_eq!(*leaf_counts.get(&12).unwrap(), 2);
        assert_eq!(*leaf_counts.get(&11).unwrap(), 1);
    }
}
