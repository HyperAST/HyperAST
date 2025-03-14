//! different decompressed tree layouts optimized for different traversals and exposing different behavior.
//!
//! Here decomressed means that nodes are not shared ie. each node only has one parent.
//!
//! The most important layout is Post Order.
//! We need both post-order traversal and breadth-first.

use hyperast::{
    types::{self, HyperAST, LendN, NodeId, NodeStore, Stored, WithChildren, WithStats},
    PrimInt,
};
use std::fmt::Debug;

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
pub trait InitializableWithStats<T: Stored>: DecompressedSubtree<T>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithStats,
{
    fn considering_stats<S>(store: &S, root: &T::TreeId) -> Self
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>;
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

pub trait FullyDecompressedTreeStore<T: Stored, IdD>: ShallowDecompressedTreeStore<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
}

pub trait ShallowDecompressedTreeStore<T: Stored, IdD, IdS = IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn len(&self) -> usize;
    fn original(&self, id: &IdD) -> T::TreeId;
    fn root(&self) -> IdS;
    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdS
    where
        // S: for<'b> types::NLending<'b, T::TreeId, N = LendN<'b, T, T::TreeId>>,
        S: types::NodeStore<T::TreeId, NMarker = T>;
    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>,
        S: NodeStore<T::TreeId>;
    fn child2<HAST>(&self, store: &HAST, x: &IdD, p: &[impl PrimInt]) -> IdS
    where
        // S: for<'t> types::AstLending<'t, RT = <T as types::NLending<'t, T::TreeId>>::N>
        // T: for<'a> hyperast::types::AstLending<'a, IdN = T::TreeId, N = <T as types::NLending<'a, T::TreeId>>::N>,
        T: for<'a> types::AstLending<'a>,
        // T: for<'t> types::NLending<'t, T::TreeId>,
        HAST: HyperAST<TM = T, IdN = T::TreeId>,
    {
        todo!()
        // self.child(store.node_store(), x, p)
    }
    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdS
where;
    fn children2<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        // S: for<'t> types::AstLending<'t, RT = <T as types::NLending<'t, T::TreeId>>::N>
        //     + HyperAST<IdN = T::TreeId>,
        T: for<'a> hyperast::types::AstLending<'a>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        S: HyperAST<IdN = T::TreeId, TM = T>,
    {
        todo!()
        // self.children(store.node_store(), x)
    }
    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
where;
    fn children5<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        // S: HyperAST<IdN = T::TreeId>
        //     + for<'a> types::AstLending<'a, RT = <T as types::NLending<'a, T::TreeId>>::N>,
        T: for<'a> hyperast::types::AstLending<'a>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        S: HyperAST<IdN = T::TreeId, TM = T>,
    {
        self.children4(store.node_store(), x)
    }
    // fn change_t<T2: WithChildren>(&self) -> impl ShallowDecompressedTreeStore<T2, IdD, IdS>;
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

pub trait LazyDecompressed<IdS> {
    type IdD: Shallow<IdS>;
}

pub trait LazyDecompressedTreeStore<T: Stored, IdS>:
    DecompressedTreeStore<T, Self::IdD, IdS> + LazyDecompressed<IdS>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    #[must_use]
    fn starter(&self) -> Self::IdD;
    #[must_use]
    fn decompress_children<S>(&mut self, store: &S, x: &Self::IdD) -> Vec<Self::IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>;
    fn decompress_to<S>(&mut self, store: &S, x: &IdS) -> Self::IdD
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>;
    // fn decompress_to2<S>(&mut self, store: &S, x: &IdS) -> Self::IdD
    // where
    //     S: for<'b> NodeStore<T::TreeId>,
    // {
    //     todo!()
    // }
    // #[must_use]
    // fn decompress_children2<S>(&mut self, store: &S, x: &Self::IdD) -> Vec<Self::IdD>
    // where
    //     S: for<'b> NodeStore<T::TreeId>,
    // {
    //     todo!()
    // }
}

pub trait DecompressedTreeStore<T: Stored, IdD, IdS = IdD>:
    ShallowDecompressedTreeStore<T, IdD, IdS>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>;
    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>;
    fn descendants2<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        // S: for<'t> types::AstLending<'t, RT = <T as types::NLending<'t, T::TreeId>>::N>
        //     + HyperAST<IdN = T::TreeId>,
        T: for<'a> hyperast::types::AstLending<'a>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        S: HyperAST<IdN = T::TreeId, TM = T>,
    {
        self.descendants(store.node_store(), x)
    }
    fn descendants_count2<S>(&self, store: &S, x: &IdD) -> usize
    where
        // S: for<'t> types::AstLending<'t, RT = <T as types::NLending<'t, T::TreeId>>::N>
        //     + HyperAST<IdN = T::TreeId>,
        T: for<'a> hyperast::types::AstLending<'a>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        S: HyperAST<IdN = T::TreeId, TM = T>,
    {
        self.descendants_count(store.node_store(), x)
    }
    fn first_descendant(&self, i: &IdD) -> IdS;
    fn is_descendant(&self, desc: &IdS, of: &IdD) -> bool;
}

pub(crate) trait DecendantsLending<'a, __ImplBound = &'a Self> {
    type Slice: 'a;
}

/// If you want to add bounds on Self::Slice, make a specialized trait like POBorrowSlice
pub trait ContiguousDescendants<T: Stored, IdD, IdS = IdD>:
    DecompressedTreeStore<T, IdD, IdS> + for<'a> DecendantsLending<'a>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdS>;

    /// The contiguous slice of descendants of x
    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice;
}

pub(crate) trait POSliceLending<'a, T: Stored, IdD, __ImplBound = &'a Self>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type SlicePo: 'a + PostOrderKeyRoots<T, IdD>;
}

/// Specialize ContiguousDescendants to specify in trait the bound of Self::Slice (here SlicePo)
/// WIP see https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
pub trait POBorrowSlice<T: Stored, IdD, IdS = IdD>:
    ContiguousDescendants<T, IdD, IdS> + for<'a> POSliceLending<'a, T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn slice_po(&self, x: &IdD) -> <Self as POSliceLending<'_, T, IdD>>::SlicePo;
}

pub(crate) trait LazyPOSliceLending<'a, T: Stored, IdD, __ImplBound = &'a Self>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    type SlicePo: 'a + PostOrderKeyRoots<T, IdD>;
}

pub trait LazyPOBorrowSlice<T: Stored, IdD, IdS = IdD>:
    ContiguousDescendants<T, IdD, IdS> + for<'a> LazyPOSliceLending<'a, T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn slice_po<S>(
        &mut self,
        store: &S,
        x: &IdD,
    ) -> <Self as LazyPOSliceLending<'_, T, IdD>>::SlicePo
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>;
}

pub(super) type CIdx<'a, S, IdN> = <<S as types::NLending<'a, IdN>>::N as WithChildren>::ChildIdx;

pub(crate) trait DecompressedParentsLending<'a, IdD, __ImplBound = &'a Self> {
    type PIt: 'a + Iterator<Item = IdD>;
}

pub trait DecompressedWithParent<T:Stored, IdD>:
    // for<'a> types::NLending<'a, IdN> + 
    for<'a> DecompressedParentsLending<'a, IdD>
    where
        T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt;
    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx>;
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx>;
    fn path_rooted<Idx: PrimInt>(&self, descendant: &IdD) -> Vec<Idx>
    where
        Self: ShallowDecompressedTreeStore<T, IdD>,
    {
        self.path(&self.root(), descendant)
    }
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

// pub trait PosInParent<T: Stored, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
// {
//     fn position_in_parent_with_store<S>(&self, store: &S, c: &IdD) -> CIdx<'_, T, T::TreeId>
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
//             + NodeStore<T::TreeId>;
// }

pub trait DecompressedWithSiblings<T: Stored, IdD>: DecompressedWithParent<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD>;
    // fn siblings_count(&self, id: &IdD) -> Option<IdD>; // TODO improve the return type
    // fn position_in_parent<Idx, S>(&self, store: &S, c: &IdD) -> Idx
    // where
    //     S: 'a + NodeStore<IdC>,
    //     S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait BreadthFirstIt<T: Stored, IdD>: DecompressedTreeStore<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    type It<'b>: Iterator<Item = IdD>;
}

pub trait BreadthFirstIterable<T: Stored, IdD>: BreadthFirstIt<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn iter_bf(&self) -> Self::It<'_>;
}

pub trait PostOrderIterable<T: Stored, IdD, IdS = IdD>: DecompressedTreeStore<T, IdD, IdS>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    type It: Iterator<Item = IdS>;
    fn iter_df_post<const ROOT: bool>(&self) -> Self::It;
}

pub trait BreadthFirstContiguousSiblings<T: Stored, IdD>: DecompressedTreeStore<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<T: Stored, IdD, IdS = IdD>: DecompressedTreeStore<T, IdD, IdS>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn lld(&self, i: &IdD) -> IdS;
    fn tree(&self, id: &IdD) -> T::TreeId;
}

pub trait PostOrdKeyRoots<'a, T: Stored, IdD, __ImplBound = &'a Self>: PostOrder<T, IdD>
// TODO should be moved ?
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    type Iter: 'a + Iterator<Item = IdD>;
}

pub trait PostOrderKeyRoots<T: Stored, IdD>: for<'a> PostOrdKeyRoots<'a, T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, T, IdD>>::Iter;
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

pub trait MapDecompressed<'a, T: Stored + 'a, IdD: PrimInt, D: DecompressedTreeStore<T, IdD>>:
    Sized
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    /// Converts to this type from the input type.
    fn map_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, N = T>;
}

pub trait WrapDecompressed<'a, T: Stored + 'a, IdD: PrimInt, D: DecompressedTreeStore<T, IdD>>:
    Sized
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    /// Converts to this type from the input type.
    fn wrap_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, N = T>;
}

// /// Used as a workaround to cache stuff that uses phantom types ie. HashedNodeRef in HyperAST
// /// TODO provide a cache wrapping this concern.
// pub trait Persistable {
//     type Persisted;
//     fn persist(self) -> Self::Persisted;
//     unsafe fn unpersist(this: Self::Persisted) -> Self;
// }

pub struct PersistedNode<I>(I);

impl<I> hyperast::types::Node for PersistedNode<I> {}

impl<I: Eq + NodeId> hyperast::types::Stored for PersistedNode<I> {
    type TreeId = I;
}
