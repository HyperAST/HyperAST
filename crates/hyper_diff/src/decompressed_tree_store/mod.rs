//! different decompressed tree layouts optimized for different traversals and exposing different behavior.
//!
//! Here decomressed means that nodes are not shared ie. each node only has one parent.
//!
//! The most important layout is Post Order.
//! We need both post-order traversal and breadth-first.

use hyperast::{
    PrimInt,
    types::{HyperAST, NodeStore, Stored, WithStats},
};

// pub mod breath_first;
pub mod basic_post_order;
pub mod bfs_wrapper;
pub mod breadth_first;
pub mod complete_post_order;
pub mod complete_post_order_ref;
pub mod hidding_wrapper;
pub mod lazy_post_order;
pub mod pre_order_wrapper;
pub mod simple_post_order;
pub use breadth_first::BreadthFirst;
pub mod simple_zs_tree;
pub use complete_post_order::CompletePostOrder;
pub use simple_zs_tree::SimpleZsTree;

pub use hyperast::types::DecompressedSubtree;

// /// show that the decompression can be done
// /// - needed to initialize in matchers
// pub trait Initializable<'a, T: Stored> {
//     /// decompress the tree at [`root`] in [`store`]
//     fn decompress<S>(store: &'a S, root: &T::TreeId) -> Self
//     where
//         S: NodeStore<T::TreeId, N = T>;
// }

/// TODO remove this trait when the specialization feature improves
///
/// NOTE compared to Initializable this trait only adds WithStats bound on T.
///
/// the WithStats bound helps a lot with lazy decompressions
pub trait InitializableWithStats<IdN>: DecompressedSubtree<IdN> {
    fn considering_stats(&self, root: &IdN) -> Self;
}

/// create a lazy decompresed tree store
///
/// You should also implement a way of finalysing the decompression
/// eg. fn finalyze(store: &'a S, lazy: Lazy) -> Self
/// - I do not think it can be easily made into a trait
pub trait LazyInitializable<'a, T: Stored + WithStats> {
    fn create<S>(store: &'a S, root: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, N = T>;
}

pub trait FullyDecompressedTreeStore<HAST: HyperAST + Copy, IdD>:
    ShallowDecompressedTreeStore<HAST, IdD>
{
}

pub trait ShallowDecompressedTreeStore<HAST: HyperAST + Copy, IdD, IdS = IdD> {
    fn len(&self) -> usize;
    fn original(&self, id: &IdD) -> HAST::IdN;
    fn root(&self) -> IdS;
    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdS;
    fn children(&self, x: &IdD) -> Vec<IdS>;
}

pub trait Shallow<T> {
    fn shallow(&self) -> &T;
    fn to_shallow(&self) -> T;
}

macro_rules! shallow_impl {
    ($ty:ty) => {
        impl Shallow<$ty> for $ty {
            fn shallow(&self) -> &$ty {
                self
            }
            fn to_shallow(&self) -> $ty {
                *self
            }
        }
    };
}

shallow_impl! {u64}
shallow_impl! {u32}
shallow_impl! {u16}
shallow_impl! {u8}

pub trait LazyDecompressed<IdS> {
    type IdD: Shallow<IdS>;
}

pub trait LazyDecompressedTreeStore<HAST: HyperAST + Copy, IdS>:
    DecompressedTreeStore<HAST, Self::IdD, IdS> + LazyDecompressed<IdS>
{
    #[must_use]
    fn starter(&self) -> Self::IdD;
    #[must_use]
    fn decompress_children(&mut self, x: &Self::IdD) -> Vec<Self::IdD>;
    fn decompress_to(&mut self, x: &IdS) -> Self::IdD;

    fn decompress_descendants(&mut self, x: &Self::IdD) {
        let mut q = self.decompress_children(x);
        while let Some(x) = q.pop() {
            // assert!(self.id_parent[x.to_usize().unwrap()] != zero());
            q.extend(self.decompress_children(&x));
        }
    }
}

pub trait DecompressedTreeStore<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    ShallowDecompressedTreeStore<HAST, IdD, IdS>
{
    fn descendants(&self, x: &IdD) -> Vec<IdS>;
    fn descendants_count(&self, x: &IdD) -> usize;
    fn descendants2(&self, x: &IdD) -> Vec<IdS> {
        self.descendants(x)
    }
    fn descendants_count2(&self, x: &IdD) -> usize {
        self.descendants_count(x)
    }
    fn first_descendant(&self, i: &IdD) -> IdS;
    fn is_descendant(&self, desc: &IdS, of: &IdD) -> bool;
}

pub trait DecendantsLending<'a, __ImplBound = &'a Self> {
    type Slice: 'a;
}

/// If you want to add bounds on Self::Slice, make a specialized trait like POBorrowSlice
pub trait ContiguousDescendants<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    DecompressedTreeStore<HAST, IdD, IdS> + for<'a> DecendantsLending<'a>
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdS>;

    /// The contiguous slice of descendants of x
    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice;
}

pub trait POSliceLending<'a, HAST: HyperAST + Copy, IdD, __ImplBound = &'a Self> {
    type SlicePo: 'a + PostOrderKeyRoots<HAST, IdD>;
}

/// Specialize ContiguousDescendants to specify in trait the bound of Self::Slice (here SlicePo)
/// WIP see https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
pub trait POBorrowSlice<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    ContiguousDescendants<HAST, IdD, IdS> + for<'a> POSliceLending<'a, HAST, IdD>
{
    fn slice_po(&self, x: &IdD) -> <Self as POSliceLending<'_, HAST, IdD>>::SlicePo;
}

pub trait LazyPOSliceLending<'a, HAST: HyperAST + Copy, IdD, __ImplBound = &'a Self> {
    type SlicePo: 'a + PostOrderKeyRoots<HAST, IdD>;
}

pub trait LazyPOBorrowSlice<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    ContiguousDescendants<HAST, IdD, IdS> + for<'a> LazyPOSliceLending<'a, HAST, IdD>
{
    fn slice_po(&mut self, x: &IdD) -> <Self as LazyPOSliceLending<'_, HAST, IdD>>::SlicePo;
}

pub trait DecompressedParentsLending<'a, IdD, __ImplBound = &'a Self> {
    type PIt: 'a + Iterator<Item = IdD>;
}

pub trait DecompressedWithParent<HAST: HyperAST + Copy, IdD>:
    for<'a> DecompressedParentsLending<'a, IdD>
{
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt;
    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx>;
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx>;
    fn path_rooted<Idx: PrimInt>(&self, descendant: &IdD) -> Vec<Idx>
    where
        Self: ShallowDecompressedTreeStore<HAST, IdD>,
    {
        self.path(&self.root(), descendant)
    }
    /// lowest common ancestor
    fn lca(&self, a: &IdD, b: &IdD) -> IdD;
}
pub trait DecompressedWithSiblings<HAST: HyperAST + Copy, IdD>:
    DecompressedWithParent<HAST, IdD>
{
    fn lsib(&self, x: &IdD) -> Option<IdD>;
}

pub trait BreadthFirstIt<HAST: HyperAST + Copy, IdD>: DecompressedTreeStore<HAST, IdD> {
    type It<'b>: Iterator<Item = IdD>;
}

pub trait BreadthFirstIterable<HAST: HyperAST + Copy, IdD>: BreadthFirstIt<HAST, IdD> {
    fn iter_bf(&self) -> Self::It<'_>;
}

pub trait PostOrderIterable<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    DecompressedTreeStore<HAST, IdD, IdS>
{
    type It: Iterator<Item = IdS>;
    fn iter_df_post<const ROOT: bool>(&self) -> Self::It;
}

pub trait BreadthFirstContiguousSiblings<HAST: HyperAST + Copy, IdD>:
    DecompressedTreeStore<HAST, IdD>
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<HAST: HyperAST + Copy, IdD, IdS = IdD>:
    DecompressedTreeStore<HAST, IdD, IdS>
{
    fn lld(&self, i: &IdD) -> IdS;
    fn tree(&self, id: &IdD) -> HAST::IdN;
}

pub trait PostOrdKeyRoots<'a, HAST: HyperAST + Copy, IdD, __ImplBound = &'a Self>:
    PostOrder<HAST, IdD>
{
    type Iter: 'a + Iterator<Item = IdD>;
}

pub trait PostOrderKeyRoots<HAST: HyperAST + Copy, IdD>:
    for<'a> PostOrdKeyRoots<'a, HAST, IdD>
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter;
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
