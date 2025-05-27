use hyperast::types::{HyperAST, HyperType, TypeStore, WithHashs};

use crate::decompressed_tree_store::LazyDecompressedTreeStore;

/// Configuration for the generic iterator
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
