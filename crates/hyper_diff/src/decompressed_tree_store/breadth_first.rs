use std::marker::PhantomData;

use num_traits::{cast, zero};

use hyperast::types::{self, Childrn, NodeId};
use hyperast::types::{NodeStore, Stored, WithChildren};
use hyperast::PrimInt;

use super::{BreadthFirstIt, CIdx, DecompressedTreeStore, Iter, ShallowDecompressedTreeStore};

use super::{
    BreadthFirstContiguousSiblings, BreadthFirstIterable, DecompressedParentsLending,
    DecompressedWithParent,
};

/// Decompressed subtree of an HyperAST layed out in breadth-first ie. contiguous siblings
pub struct BreadthFirst<T: Stored, IdD: PrimInt, IdN = <T as Stored>::TreeId> {
    id_compressed: Vec<IdN>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
    phantom: PhantomData<*const T>,
}

// impl<'a, T: Stored, IdD: PrimInt> types::NLending<'a, T::TreeId> for BreadthFirst<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

impl<'d, T: Stored, IdD: PrimInt> BreadthFirstContiguousSiblings<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn has_children(&self, id: &IdD) -> bool {
        self.first_child(id) != None
    }

    fn first_child(&self, id: &IdD) -> Option<IdD> {
        let r = self.id_first_child[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }
}

impl<T: Stored, IdD: PrimInt> BreadthFirstIt<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    type It<'a> = Iter<IdD>;
}

impl<T: Stored, IdD: PrimInt> BreadthFirstIterable<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn iter_bf(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'a, T: Stored, IdD: PrimInt> DecompressedParentsLending<'a, IdD> for BreadthFirst<T, IdD> {
    type PIt = IterParents<'a, IdD>;
}

impl<T: Stored, IdD: PrimInt> DecompressedWithParent<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        let r = self.id_parent[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.parent(id) != None
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        let p = self.parent(c)?;
        Some(cast(*c - self.first_child(&p).unwrap()).unwrap())
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> Vec<Idx> {
        todo!()
    }

    fn lca(&self, _a: &IdD, _b: &IdD) -> IdD {
        todo!()
    }
}

pub struct IterParents<'a, IdD> {
    id: IdD,
    id_parent: &'a Vec<IdD>,
}

impl<'a, IdD: PrimInt> Iterator for IterParents<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.id_parent[self.id.to_usize().unwrap()];
        if r == num_traits::zero() {
            return None;
        }
        self.id = r.clone();
        Some(r)
    }
}

impl<T: Stored, IdD: PrimInt> super::DecompressedSubtree<T> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    type Out = Self;

    fn decompress<S>(store: &S, root: &T::TreeId) -> Self
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        let mut id_compressed: Vec<T::TreeId> = vec![root.clone()];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            store.scoped_mut(&id_compressed[i].clone(), |x| {
                let l = x.children();
                let value = if l.as_ref().map_or(false, |x| !types::Childrn::is_empty(x)) {
                    cast(id_compressed.len()).unwrap()
                } else {
                    num_traits::zero()
                };
                id_first_child.push(value);
                if let Some(l) = l {
                    id_parent.extend(l.iter_children().map(|_| cast::<usize, IdD>(i).unwrap()));
                    id_compressed.extend(l.iter_children());
                }
            });

            i += 1;
        }

        BreadthFirst {
            id_compressed,
            id_parent,
            id_first_child,
            phantom: PhantomData,
        }
    }

    fn decompress2<HAST>(store: &HAST, root: &<T as Stored>::TreeId) -> Self::Out
    where
        T: for<'t> types::AstLending<'t>,
        HAST: types::HyperAST<IdN = <T as Stored>::TreeId, TM = T>,
    {
        let mut id_compressed: Vec<T::TreeId> = vec![root.clone()];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            let x = store.resolve(&id_compressed[i].clone());

            let l = x.children();
            let value = if l.as_ref().map_or(false, |x| !types::Childrn::is_empty(x)) {
                cast(id_compressed.len()).unwrap()
            } else {
                num_traits::zero()
            };
            id_first_child.push(value);
            if let Some(l) = l {
                id_parent.extend(l.iter_children().map(|_| cast::<usize, IdD>(i).unwrap()));
                id_compressed.extend(l.iter_children());
            }

            i += 1;
        }

        BreadthFirst {
            id_compressed,
            id_parent,
            id_first_child,
            phantom: PhantomData,
        }
    }
}

impl<T: Stored, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
{
    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    // fn leaf_count(&self) -> IdD {
    //     self.leaf_count
    // }

    fn root(&self) -> IdD {
        zero()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);

            let cs = {
                let n = store.resolve(&a);
                n.child_count()
            };
            if cs > zero() {
                r = self.first_child(&r).unwrap() + cast(*d).unwrap();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        unimplemented!()
        // let mut r = *x;
        // for d in p {
        //     let a = self.original(&r);

        //     let cs = store.scoped(&a, |n| n.child_count());
        //     if cs > zero() {
        //         r = self.first_child(&r).unwrap() + cast(*d).unwrap();
        //     } else {
        //         panic!("no children in this tree")
        //     }
        // }
        // r
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let node = store.resolve(&self.original(x));
        let l: usize = cast(node.child_count()).unwrap();
        let s: usize = cast(*x).unwrap();
        let r = s + 1..s + l;
        r.map(|x| cast::<usize, IdD>(x).unwrap())
            .collect::<Vec<_>>()
            .to_owned()
    }

    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        unimplemented!()
        // let cs = store.scoped(&self.original(x), |n| n.child_count());
        // let l: usize = cast(cs).unwrap();
        // let s: usize = cast(*x).unwrap();
        // let r = s + 1..s + l;
        // r.map(|x| cast::<usize, IdD>(x).unwrap())
        //     .collect::<Vec<_>>()
        //     .to_owned()
    }
}

impl<'a, T: Stored, IdD: PrimInt> DecompressedTreeStore<T, IdD> for BreadthFirst<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        // TODO possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let x = store.resolve(&self.original(&id[i]));
            let child_count = x.child_count();
            let l: usize = cast(child_count).unwrap();
            let s: usize = cast(id[i]).unwrap();
            let r = s + 1..s + l;
            id.extend(r.map(|x| cast::<usize, IdD>(x).unwrap()));
            i += 1;
        }
        id
    }

    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        // TODO possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let x = store.resolve(&self.original(&id[i]));
            let child_count = x.child_count();
            let l: usize = cast(child_count).unwrap();
            let s: usize = cast(id[i]).unwrap();
            let r = s + 1..s + l;
            id.extend(r.map(|x| cast::<usize, IdD>(x).unwrap()));
            i += 1;
        }
        id.len()
    }

    fn first_descendant(&self, _i: &IdD) -> IdD {
        todo!()
    }

    fn is_descendant(&self, _desc: &IdD, _of: &IdD) -> bool {
        todo!()
    }
}
