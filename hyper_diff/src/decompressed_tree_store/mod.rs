//! different decompressed tree layouts optimized for different traversals and exposing different behavior.
//!
//! Here decomressed means that nodes are not shared ie. each node only has one parent.
//!
//! The most important layout is Post Order.
//! We need both post-order traversal and breadth-first.
use num_traits::PrimInt;

use hyper_ast::types::{NodeStore, Stored, WithChildren, WithStats};

// pub mod breath_first;
pub mod basic_post_order;
pub mod bfs_wrapper;
pub mod breadth_first;
pub mod complete_post_order;
pub mod lazy_post_order;
pub mod pre_order_wrapper;
pub mod simple_post_order;
pub use breadth_first::BreathFirst;
pub mod simple_zs_tree;
pub use complete_post_order::CompletePostOrder;
pub use simple_zs_tree::SimpleZsTree;

pub use hyper_ast::types::DecompressedSubtree;

// /// show that the decompression can be done
// /// - needed to initialize in matchers
// pub trait Initializable<'a, T: Stored> {
//     /// decompress the tree at [`root`] in [`store`]
//     fn decompress<S>(store: &'a S, root: &T::TreeId) -> Self
//     where
//         S: NodeStore<T::TreeId, R<'a> = T>;
// }

/// TODO remove this trait when the specialization feature improves
///
/// NOTE compared to Initializable this trait only adds WithStats bound on T.
///
/// the WithStats bound helps a lot with lazy decompressions
pub trait InitializableWithStats<'a, T: Stored + WithStats>: DecompressedSubtree<'a, T> {
    fn considering_stats<S>(store: &'a S, root: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

/// create a lazy decompresed tree store
///
/// You should also implement a way of finalysing the decompression
/// eg. fn finalyze(store: &'a S, lazy: Lazy) -> Self
/// - I do not think it can be easily made into a trait
pub trait LazyInitializable<'a, T: Stored + WithStats> {
    fn create<S>(store: &'a S, root: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait ShallowDecompressedTreeStore<'a, T: WithChildren, IdD, IdS = IdD> {
    fn len(&self) -> usize;
    fn original(&self, id: &IdD) -> T::TreeId;
    fn root(&self) -> IdS;
    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdS
    where
        //'a: 'b,
        S: 'b + NodeStore<T::TreeId, R<'b> = T>;
    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdS>
    where
        // 'a: 'b,
        S: NodeStore<T::TreeId, R<'b> = T>;
}
pub trait Shallow<T> {
    fn shallow(&self) -> &T;
    // fn direct(&self) -> T where T: Clone{
    //     self.shallow().clone()
    // }
}

impl Shallow<u64> for u64 {
    fn shallow(&self) -> &u64 {
        self
    }
}
impl Shallow<u32> for u32 {
    fn shallow(&self) -> &u32 {
        self
    }
}
impl Shallow<u16> for u16 {
    fn shallow(&self) -> &u16 {
        self
    }
}
impl Shallow<u8> for u8 {
    fn shallow(&self) -> &u8 {
        self
    }
}

pub trait LazyDecompressedTreeStore<'a, T: WithChildren + WithStats, IdS>:
    DecompressedTreeStore<'a, T, Self::IdD, IdS>
{
    type IdD: Shallow<IdS>;
    #[must_use]
    fn starter(&self) -> Self::IdD;
    #[must_use]
    fn decompress_children<'b, S>(&mut self, store: &'b S, x: &Self::IdD) -> Vec<Self::IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
    fn decompress_to<'b, S>(&mut self, store: &'b S, x: &IdS) -> Self::IdD
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
}

pub trait DecompressedTreeStore<'a, T: WithChildren, IdD, IdS = IdD>:
    ShallowDecompressedTreeStore<'a, T, IdD, IdS>
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdS>
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
    fn first_descendant(&self, i: &IdD) -> IdS;
    fn is_descendant(&self, desc: &IdS, of: &IdD) -> bool;
}

/// If you want to add bounds on Self::Slice, make a specialized trait like POBorrowSlice
pub trait ContiguousDescendants<'a, T: WithChildren, IdD, IdS = IdD>:
    DecompressedTreeStore<'a, T, IdD, IdS>
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdS>;

    type Slice<'b>
    where
        Self: 'b;

    /// The contguous slice of descendants of x and x
    fn slice(&self, x: &IdD) -> Self::Slice<'_>;
}

/// Specialize ContiguousDescendants to specify in trait the bound of Self::Slice (here SlicePo)
/// WIP see https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
pub trait POBorrowSlice<'a, T: WithChildren, IdD, IdS = IdD>:
    ContiguousDescendants<'a, T, IdD, IdS>
{
    type SlicePo<'b>: PostOrderKeyRoots<'b, T, IdD>
    where
        Self: 'b;

    fn slice_po(&self, x: &IdD) -> Self::SlicePo<'_>;
}

pub trait LazyPOBorrowSlice<'a, T: WithChildren, IdD, IdS = IdD>:
    ContiguousDescendants<'a, T, IdD, IdS>
{
    type SlicePo<'b>: PostOrderKeyRoots<'b, T, IdD>
    where
        Self: 'b;

    fn slice_po<'b, S>(&mut self, store: &'b S, x: &IdD) -> Self::SlicePo<'_>
    where
        S: NodeStore<T::TreeId, R<'b> = T>;
}

pub trait DecompressedWithParent<'a, T: WithChildren, IdD> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    type PIt<'b>: 'b + Iterator<Item = IdD>
    where
        Self: 'b;
    fn parents(&self, id: IdD) -> Self::PIt<'_>;
    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx>;
    fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<T::ChildIdx>;
    /// lowest common ancestor
    fn lca(&self, a: &IdD, b: &IdD) -> IdD;
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

pub trait DecompressedWithSiblings<'a, T: WithChildren, IdD>:
    DecompressedWithParent<'a, T, IdD>
{
    fn lsib(&self, x: &IdD) -> Option<IdD>;
    // fn siblings_count(&self, id: &IdD) -> Option<IdD>; // TODO improve the return type
    // fn position_in_parent<Idx, S>(&self, store: &S, c: &IdD) -> Idx
    // where
    //     S: 'a + NodeStore<IdC>,
    //     S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait BreadthFirstIterable<'a, T: WithChildren, IdD>:
    DecompressedTreeStore<'a, T, IdD>
{
    type It: Iterator<Item = IdD>;
    fn iter_bf(&'a self) -> Self::It;
}

pub trait PostOrderIterable<'a, T: WithChildren, IdD, IdS = IdD>:
    DecompressedTreeStore<'a, T, IdD, IdS>
{
    type It: Iterator<Item = IdS>;
    fn iter_df_post<const ROOT: bool>(&self) -> Self::It;
}

pub trait BreathFirstContiguousSiblings<'a, T: WithChildren, IdD>:
    DecompressedTreeStore<'a, T, IdD>
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<'a, T: WithChildren, IdD, IdS = IdD>:
    DecompressedTreeStore<'a, T, IdD, IdS>
{
    fn lld(&self, i: &IdD) -> IdS;
    fn tree(&self, id: &IdD) -> T::TreeId;
}

pub trait PostOrderKeyRoots<'a, T: WithChildren, IdD>: PostOrder<'a, T, IdD> {
    type Iter<'b>: Iterator<Item = IdD>
    where
        Self: 'b;
    fn iter_kr(&self) -> Self::Iter<'_>;
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

pub struct IterKr<'a, IdD>(
    bitvec::slice::IterOnes<'a, usize, bitvec::prelude::LocalBits>,
    std::marker::PhantomData<*const IdD>,
);

impl<'a, IdD: PrimInt> Iterator for IterKr<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        num_traits::cast(self.0.next()?)
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
