//! different decompressed tree layouts optimized for different traversals and exposing different behavior.
//!
//! Here decomressed means that nodes are not shared ie. each node only has one parent.
//!
//! The most important layout is Post Order.
//! We need both post-order traversal and breadth-first.
use num_traits::PrimInt;

use hyperast::types::{HyperAST, NodStore, NodeId, NodeStore, Stored, WithChildren, WithStats};

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
pub use breadth_first::BreathFirst;
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
//         S: NodeStore<T::TreeId, R<'a> = T>;
// }

/// TODO remove this trait when the specialization feature improves
///
/// NOTE compared to Initializable this trait only adds WithStats bound on T.
///
/// the WithStats bound helps a lot with lazy decompressions
pub trait InitializableWithStats<'a, T: Stored + WithStats>: DecompressedSubtree<T> {
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

pub trait FullyDecompressedTreeStore<T: WithChildren, IdD>:
    ShallowDecompressedTreeStore<T, IdD>
{
}

pub trait ShallowDecompressedTreeStore<T: WithChildren, IdD, IdS = IdD> {
    fn len(&self) -> usize;
    fn original(&self, id: &IdD) -> T::TreeId;
    fn root(&self) -> IdS;
    fn child<S>(&self, store: &S, x: &IdD, p: &[T::ChildIdx]) -> IdS
    where
        S: for<'b> NodStore<T::TreeId, R<'b> = T> + NodeStore<T::TreeId>;
    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: for<'b> NodStore<T::TreeId, R<'b> = T> + NodeStore<T::TreeId>;
    fn child2<S>(&self, store: &S, x: &IdD, p: &[T::ChildIdx]) -> IdS
    where
        S: for<'t> HyperAST<T<'t> = T, IdN = T::TreeId>,
    {
        self.child(store.node_store(), x, p)
    }
    fn child4<S>(&self, store: &S, x: &IdD, p: &[T::ChildIdx]) -> IdS
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
    fn children2<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: for<'t> HyperAST<T<'t> = T, IdN = T::TreeId>,
    {
        self.children(store.node_store(), x)
    }
    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
    fn children5<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: HyperAST<IdN = T::TreeId, RT = T>,
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

pub trait LazyDecompressedTreeStore<T: WithChildren + WithStats, IdS>:
    DecompressedTreeStore<T, Self::IdD, IdS> + LazyDecompressed<IdS>
{
    #[must_use]
    fn starter(&self) -> Self::IdD;
    #[must_use]
    fn decompress_children<S>(&mut self, store: &S, x: &Self::IdD) -> Vec<Self::IdD>
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
    fn decompress_to<S>(&mut self, store: &S, x: &IdS) -> Self::IdD
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
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

pub trait DecompressedTreeStore<T: WithChildren, IdD, IdS = IdD>:
    ShallowDecompressedTreeStore<T, IdD, IdS>
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
    fn descendants2<S>(&self, store: &S, x: &IdD) -> Vec<IdS>
    where
        S: HyperAST<IdN = T::TreeId, RT = T>,
    {
        self.descendants(store.node_store(), x)
    }
    fn descendants_count2<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: HyperAST<IdN = T::TreeId, RT = T>,
    {
        self.descendants_count(store.node_store(), x)
    }
    fn first_descendant(&self, i: &IdD) -> IdS;
    fn is_descendant(&self, desc: &IdS, of: &IdD) -> bool;
}

/// If you want to add bounds on Self::Slice, make a specialized trait like POBorrowSlice
pub trait ContiguousDescendants<T: WithChildren, IdD, IdS = IdD>:
    DecompressedTreeStore<T, IdD, IdS>
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
pub trait POBorrowSlice<T: WithChildren, IdD, IdS = IdD>:
    ContiguousDescendants<T, IdD, IdS>
{
    type SlicePo<'b>: PostOrderKeyRoots<T, IdD>
    where
        Self: 'b;

    fn slice_po(&self, x: &IdD) -> Self::SlicePo<'_>;
}

pub trait LazyPOBorrowSlice<T: WithChildren, IdD, IdS = IdD>:
    ContiguousDescendants<T, IdD, IdS>
{
    type SlicePo<'b>: PostOrderKeyRoots<T, IdD>
    where
        Self: 'b;

    fn slice_po<S>(&mut self, store: &S, x: &IdD) -> Self::SlicePo<'_>
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>;
}

pub trait DecompressedWithParent<T: WithChildren, IdD> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    type PIt<'b>: 'b + Iterator<Item = IdD>
    where
        Self: 'b;
    fn parents(&self, id: IdD) -> Self::PIt<'_>;
    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx>;
    fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<T::ChildIdx>;
    fn path_rooted(&self, descendant: &IdD) -> Vec<T::ChildIdx>
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

pub trait PosInParent<T: WithChildren, IdD> {
    fn position_in_parent_with_store<'b, S>(
        &self,
        store: &'b S,
        c: &IdD,
    ) -> <S::R<'b> as WithChildren>::ChildIdx
    where
        S: for<'t> NodeStore<T::TreeId, R<'t> = T>,
        for<'t> <S::R<'t> as WithChildren>::ChildIdx: PrimInt;
}

pub trait DecompressedWithSiblings<T: WithChildren, IdD>: DecompressedWithParent<T, IdD> {
    fn lsib(&self, x: &IdD) -> Option<IdD>;
    // fn siblings_count(&self, id: &IdD) -> Option<IdD>; // TODO improve the return type
    // fn position_in_parent<Idx, S>(&self, store: &S, c: &IdD) -> Idx
    // where
    //     S: 'a + NodeStore<IdC>,
    //     S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait BreadthFirstIt<T: WithChildren, IdD>: DecompressedTreeStore<T, IdD> {
    type It<'b>: Iterator<Item = IdD>;
}

pub trait BreadthFirstIterable<T: WithChildren, IdD>: BreadthFirstIt<T, IdD> {
    fn iter_bf(&self) -> Self::It<'_>;
}

pub trait PostOrderIterable<T: WithChildren, IdD, IdS = IdD>:
    DecompressedTreeStore<T, IdD, IdS>
{
    type It: Iterator<Item = IdS>;
    fn iter_df_post<const ROOT: bool>(&self) -> Self::It;
}

pub trait BreathFirstContiguousSiblings<T: WithChildren, IdD>:
    DecompressedTreeStore<T, IdD>
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<T: WithChildren, IdD, IdS = IdD>: DecompressedTreeStore<T, IdD, IdS> {
    fn lld(&self, i: &IdD) -> IdS;
    fn tree(&self, id: &IdD) -> T::TreeId;
}

pub trait PostOrdKeyRoots<T: WithChildren, IdD>: PostOrder<T, IdD> {
    type Iter<'b>: Iterator<Item = IdD>;
}

pub trait PostOrderKeyRoots<T: WithChildren, IdD>: PostOrdKeyRoots<T, IdD> {
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

pub trait MapDecompressed<'a, T: WithChildren + 'a, IdD: PrimInt, D: DecompressedTreeStore<T, IdD>>:
    Sized
{
    /// Converts to this type from the input type.
    fn map_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait WrapDecompressed<'a, T: WithChildren + 'a, IdD: PrimInt, D: DecompressedTreeStore<T, IdD>>:
    Sized
{
    /// Converts to this type from the input type.
    fn wrap_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
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
