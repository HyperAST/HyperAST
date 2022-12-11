use std::{fmt::Debug, marker::PhantomData};

/// different decompressed tree layouts optimized for different traversals and exposing different features.
/// Here decomressed means that nodes are not shared ie. they only have one parent.
use num_traits::PrimInt;

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{NodeStore, Tree, WithChildren};

// pub mod breath_first;
pub mod bfs_wrapper;
pub mod breath_first;
pub mod complete_post_order;
pub mod pre_order_wrapper;
pub use breath_first::BreathFirst;
pub mod simple_zs_tree;
pub use simple_zs_tree::SimpleZsTree;
// pub mod complete_post_order;
pub use complete_post_order::CompletePostOrder;

pub trait Initializable<'a, IdC, IdD> {
    fn new<
        // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        //NodeStore<'b, T::TreeId, T>,
        S,
    >(
        store: &'a S,
        root: &IdC,
    ) -> Self
    where
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:
        // WithChildren<TreeId = IdC>,
        S: NodeStore<IdC>, //NodeStoreExt<'a, T, R>,
        S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait ShallowDecompressedTreeStore<'a, IdC, IdD>: Initializable<'a, IdC, IdD> {
    fn len(&self) -> usize;
    // fn node_count(&self) -> IdD {
    //     cast(self.len()).unwrap()
    // }
    fn original(&self, id: &IdD) -> IdC;
    fn leaf_count(&self) -> IdD;
    fn root(&self) -> IdD;
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<Idx>;
    fn child<'b, S>(
        &self,
        store: &'b S,
        x: &IdD,
        p: &[<S::R<'b> as WithChildren>::ChildIdx],
    ) -> IdD
    where
        'a: 'b,
        S: NodeStore<IdC>,
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>;
    // fn child_count<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a,T>>(
    //     &self,
    //     store: &S,
    //     x: &IdD,
    // ) -> IdD;
    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        'a: 'b,
        S: 'b + NodeStore<IdC>,
        // for<'c> < <S as NodeStore2<IdC>>::R  as GenericItem<'c>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>;
}

pub trait DecompressedTreeStore<'a, IdC, IdD>: ShallowDecompressedTreeStore<'a, IdC, IdD> {
    // fn descendants<'a, T: 'a + Tree<TreeId = IdC>, S>(&self, store: &'a S, x: &IdD) -> Vec<IdD>
    // where
    //     S: 'a + NodeStore2<T::TreeId, R<'a> = T> //NodeStore<'a, T::TreeId, T>
    // ;
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>;
    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<IdC>,
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>;
    // S: 'a + NodeStore2<T::TreeId, R<'a> = T> //NodeStore<'a, T::TreeId, T>
    fn first_descendant(&self, i: &IdD) -> IdD;
}
pub trait ContiguousDescendants<'a, IdC, IdD>: DecompressedTreeStore<'a, IdC, IdD> {
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD>;
}
// pub struct IterNormPath<'x, IdC, IdD: Clone, D: DecompressedWithParent<'x, IdC, IdD>> {
//     id: IdD,
//     internal: &'x D,
//     phantom: PhantomData<*const IdC>,
// }

// impl<'x, IdC, IdD: Clone, D: DecompressedWithParent<'x, IdC, IdD>> Iterator
//     for IterNormPath<'x, IdC, IdD, D>
// {
//     type Item = IdD;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.id = self.internal.parent(&self.id)?;
//         Some(self.id.clone())
//     }
// }

pub trait DecompressedWithParent<'a, IdC, IdD: Clone> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    type PIt<'b>: 'b + Iterator<Item=IdD> where Self: 'b;
    fn parents(
        &self,
        id: IdD,
    ) -> Self::PIt<'_>;
    // fn norm_path(
    //     &self,
    //     id: IdD,
    // ) -> Self::PIt<'_>;
    // fn parents<'b,D:DecompressedWithParent<'b, IdC, IdD>>(
    //     d: &'b D,
    //     id: IdD,
    // ) -> impl Iterator<Item=IdD>
    // where
    //     Self: Sized,
    // {
    //     IterParents {
    //         id: id.clone(),
    //         internal: d,
    //         phantom: PhantomData,
    //     }
    // }
    fn position_in_parent<'b, S>(
        &self,
        store: &'b S,
        c: &IdD,
    ) -> <S::R<'b> as WithChildren>::ChildIdx
    where
        S: NodeStore<IdC>,
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>;
    // S: 'a + NodeStore2<T::TreeId, R<'a> = T> //NodeStore<'a, T::TreeId, T>
}

pub trait DecompressedWithSiblings<'a, IdC, IdD> {
    fn siblings_count(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<Idx, S>(&self, store: &S, c: &IdD) -> Idx
    where
        S: 'a + NodeStore<IdC>,
        // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait BreathFirstIterable<'a, IdC, IdD>: DecompressedTreeStore<'a, IdC, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_bf(&'a self) -> Self::It;
}

pub trait PostOrderIterable<'a, IdC, IdD>: DecompressedTreeStore<'a, IdC, IdD> {
    type It: Iterator<Item = IdD>;
    fn iter_df_post(&self) -> Self::It;
}

pub trait BreathFirstContiguousSiblings<'a, IdC, IdD>: DecompressedTreeStore<'a, IdC, IdD> {
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<'a, IdC, IdD>: PostOrderIterable<'a, IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD;
    fn tree(&self, id: &IdD) -> IdC;
}

pub trait PostOrderKeyRoots<'a, IdC, IdD: PrimInt>: PostOrder<'a, IdC, IdD> {
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

fn size<'a, IdC, S>(store: &'a S, x: &IdC) -> usize
where
    S: 'a + NodeStore<IdC>,
    // for<'a> < <S as NodeStore2<IdC>>::R  as GenericItem<'a>>::Item:WithChildren<TreeId = IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>, // S: 'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>
{
    let tmp = store.resolve(&x);
    let cs = if let Some(cs) = tmp.try_get_children() {
        cs
    } else {
        return 1;
    };

    let mut z = 0;
    for x in cs {
        z += size(store, &x);
    }
    z + 1
}

pub trait MapDecompressed<'a, IdC, IdD: PrimInt, D: DecompressedTreeStore<'a, IdC, IdD>>:
    Sized
{
    /// Converts to this type from the input type.
    fn map_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>;
}

pub trait WrapDecompressed<'a, IdC, IdD: PrimInt, D: DecompressedTreeStore<'a, IdC, IdD>>:
    Sized
{
    /// Converts to this type from the input type.
    fn wrap_it<S>(_: &'a S, _: &'a D) -> Self
    where
        S: NodeStore<IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>;
}

// pub struct SimpleMapper<'a, IdC, IdD, D: DecompressedTreeStore<'a, IdC, IdD>> {
//     map: Vec<IdD>,
//     // fc: Vec<IdD>,
//     rev: Vec<IdD>,
//     back: &'a D,
//     phantom: PhantomData<*const IdC>,
// }

// impl<'a, IdC, IdD: Debug, D: Debug + DecompressedTreeStore<'a, IdC, IdD>> Debug
//     for SimpleMapper<'a, IdC, IdD, D>
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("SD")
//             .field("map", &self.map)
//             .field("rev", &self.rev)
//             .field("back", &self.back)
//             .field("phantom", &self.phantom)
//             .finish()
//     }
// }
