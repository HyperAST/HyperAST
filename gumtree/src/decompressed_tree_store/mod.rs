/// different decompressed tree layouts optimized for different traversals and exposing different features.
/// Here decomressed means that nodes are not shared ie. they only have one parent.
use num_traits::PrimInt;

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{NodeStore, Stored, WithChildren, WithStats};

// pub mod breath_first;
pub mod bfs_wrapper;
pub mod breath_first;
pub mod complete_post_order;
pub mod pre_order_wrapper;
pub mod simple_post_order;
pub use breath_first::BreathFirst;
pub mod simple_zs_tree;
pub use complete_post_order::CompletePostOrder;
pub use simple_zs_tree::SimpleZsTree;

pub trait Initializable<'a, T: Stored> {
    fn new<S>(store: &'a S, root: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

/// TODO remove this trait when the specialization feature improves
/// NOTE compared to Initializable this trait only adds WithStats bound on T.
/// the WithStats bound helps a lot with lazy decompressions
pub trait InitializableWithStats<'a, T: Stored + WithStats>: Initializable<'a, T> {
    fn considering_stats<S>(store: &'a S, root: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait ShallowDecompressedTreeStore<'a, T: WithChildren, IdD> {
    fn len(&self) -> usize;
    fn original(&self, id: &IdD) -> T::TreeId;
    fn leaf_count(&self) -> IdD;
    fn root(&self) -> IdD;
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<Idx>;
    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        //'a: 'b,
        S: 'b + NodeStore<T::TreeId, R<'b> = T>;
    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        // 'a: 'b,
        S: NodeStore<T::TreeId, R<'b> = T>;
}

pub trait DecompressedTreeStore<'a, T: WithChildren, IdD>:
    ShallowDecompressedTreeStore<'a, T, IdD>
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
    fn first_descendant(&self, i: &IdD) -> IdD;
}
pub trait ContiguousDescendants<'a, T: WithChildren, IdD>:
    DecompressedTreeStore<'a, T, IdD>
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD>;
}

pub trait DecompressedWithParent<'a, T: WithChildren, IdD: Clone> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    type PIt<'b>: 'b + Iterator<Item = IdD>
    where
        Self: 'b;
    fn parents(&self, id: IdD) -> Self::PIt<'_>;
    fn position_in_parent(
        &self,
        c: &IdD,
    ) -> Option<T::ChildIdx>;
    // fn position_in_parent<'b, S>(
    //     &self,
    //     store: &'b S,
    //     c: &IdD,
    // ) -> <S::R<'b> as WithChildren>::ChildIdx
    // where
    //     S: NodeStore<T::TreeId, R<'b> = T>;
}

pub trait PosInParent<'a, T: WithChildren, IdD> {
    fn position_in_parent_with_store<'b, S>(
        &self,
        store: &'b S,
        c: &IdD,
    ) -> <S::R<'b> as WithChildren>::ChildIdx
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
        <S::R<'b> as WithChildren>::ChildIdx: PrimInt;
}

pub trait DecompressedWithSiblings<'a, IdC, IdD> {
    fn siblings_count(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<Idx, S>(&self, store: &S, c: &IdD) -> Idx
    where
        S: 'a + NodeStore<IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait BreathFirstIterable<'a, T: WithChildren, IdD>: DecompressedTreeStore<'a, T, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_bf(&'a self) -> Self::It;
}

pub trait PostOrderIterable<'a, T: WithChildren, IdD>: DecompressedTreeStore<'a, T, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_df_post(&self) -> Self::It;
}

pub trait BreathFirstContiguousSiblings<'a, T: WithChildren, IdD>:
    DecompressedTreeStore<'a, T, IdD>
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<'a, T: 'a + WithChildren, IdD>: PostOrderIterable<'a, T, IdD> {
    fn lld(&self, i: &IdD) -> IdD;
    fn tree(&self, id: &IdD) -> T::TreeId;
}

pub trait PostOrderKeyRoots<'a, T: 'a + WithChildren, IdD: PrimInt>: PostOrder<'a, T, IdD> {
    fn kr(&self, x: IdD) -> IdD;
}

pub struct Iter<IdD> {
    current: IdD,
    len: IdD,
}

impl<IdD: PrimInt> Iterator for Iter<IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        use num_traits::one;
        if self.current == self.len {
            None
        } else {
            let r = self.current;
            self.current = r + one();
            Some(r)
        }
    }
}

pub trait MapDecompressed<
    'a,
    T: WithChildren + 'a,
    IdD: PrimInt,
    D: DecompressedTreeStore<'a, T, IdD>,
>: Sized
{
    /// Converts to this type from the input type.
    fn map_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait WrapDecompressed<
    'a,
    T: WithChildren + 'a,
    IdD: PrimInt,
    D: DecompressedTreeStore<'a, T, IdD>,
>: Sized
{
    /// Converts to this type from the input type.
    fn wrap_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}
