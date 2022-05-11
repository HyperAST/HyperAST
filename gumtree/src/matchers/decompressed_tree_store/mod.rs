/// different decompressed tree layouts optimized for different traversals and exposing different features.
/// Here decomressed means that nodes are not shared ie. they only have one parent.

use num_traits::PrimInt;

use crate::tree::{
    tree::{NodeStore, Tree, WithChildren},
    tree_path::CompressedTreePath,
};

pub trait Initializable<IdC, IdD> {
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: for<'b> NodeStore<'b, T::TreeId, &'b T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self;
}

pub trait ShallowDecompressedTreeStore<IdC, IdD>: Initializable<IdC, IdD> {
    fn len(&self) -> usize;
    // fn node_count(&self) -> IdD {
    //     cast(self.len()).unwrap()
    // }
    fn original(&self, id: &IdD) -> IdC;
    fn leaf_count(&self) -> IdD;
    fn root(&self) -> IdD;
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<Idx>;
    fn child<T: WithChildren<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD;
    // fn child_count<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a,T>>(
    //     &self,
    //     store: &S,
    //     x: &IdD,
    // ) -> IdD;
    fn children<T: WithChildren<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD>;
}

pub trait DecompressedTreeStore<IdC, IdD>: ShallowDecompressedTreeStore<IdC, IdD> {
    fn descendants<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD>;
    fn descendants_count<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize;
    fn first_descendant(&self, i: &IdD) -> IdD;
}

pub trait DecompressedWithParent<IdD> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<T: WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        c: &IdD,
    ) -> T::ChildIdx;
}

pub trait DecompressedWithSiblings<IdD> {
    fn siblings_count(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<T: Tree, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        c: &IdD,
    ) -> T::ChildIdx;
}

pub trait BreathFirstIterable<'a, IdC, IdD>: DecompressedTreeStore<IdC, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_bf(&'a self) -> Self::It;
}

pub trait PostOrderIterable<IdC, IdD>: DecompressedTreeStore<IdC, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_df_post(&self) -> Self::It;
}

pub trait BreathFirstContiguousSiblings<IdC, IdD>: DecompressedTreeStore<IdC, IdD> {
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<IdC, IdD>: PostOrderIterable<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD;
    fn tree(&self, id: &IdD) -> IdC;
}

pub trait PostOrderKeyRoots<IdC, IdD: PrimInt + Into<usize>>: PostOrder<IdC, IdD> {
    fn kr(&self, x: IdD) -> IdD;
}

pub mod breath_first;
pub use breath_first::BreathFirst;
pub mod simple_zs_tree;
pub use simple_zs_tree::SimpleZsTree;
pub mod complete_post_order;
pub use complete_post_order::CompletePostOrder;

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

fn size<T: WithChildren, NS: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
    store: &NS,
    x: &T::TreeId,
) -> usize {
    let tmp = store.resolve(&x);
    let cs = tmp.get_children().to_owned();

    let mut z = 0;
    for x in cs {
        z += size(store, &x);
    }
    z + 1
}
