use std::marker::PhantomData;

use num_traits::{cast, zero, PrimInt};

use hyperast::types::{IterableChildren, NodeId};
use hyperast::types::{NodeStore, Stored, WithChildren};

use super::{BreadthFirstIt, DecompressedTreeStore, Iter, ShallowDecompressedTreeStore};

use super::{BreadthFirstIterable, BreathFirstContiguousSiblings, DecompressedWithParent};

/// Decompressed subtree of an HyperAST layed out in breadth-first ie. contiguous siblings
pub struct BreathFirst<T: Stored, IdD: PrimInt> {
    id_compressed: Vec<T::TreeId>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
    phantom: PhantomData<*const T>,
}

impl<'d, T: WithChildren, IdD: PrimInt> BreathFirstContiguousSiblings<T, IdD>
    for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
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

impl<T: WithChildren, IdD: PrimInt> BreadthFirstIt<T, IdD> for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
{
    type It<'a> = Iter<IdD>;
}

impl<T: WithChildren, IdD: PrimInt> BreadthFirstIterable<T, IdD> for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
{
    fn iter_bf(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}
impl<T: WithChildren, IdD: PrimInt> DecompressedWithParent<T, IdD> for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
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

    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
        let p = self.parent(c)?;
        Some(cast(*c - self.first_child(&p).unwrap()).unwrap())
    }

    type PIt<'b>
        = IterParents<'b, IdD>
    where
        IdD: 'b,
        T::TreeId: 'b,
        T: 'b;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }

    fn path(&self, _parent: &IdD, _descendant: &IdD) -> Vec<T::ChildIdx> {
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

impl<T: WithChildren, IdD: PrimInt> super::DecompressedSubtree<T> for BreathFirst<T, IdD>
where
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    type Out = Self;

    fn decompress<S>(store: &S, root: &T::TreeId) -> Self
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        let mut id_compressed: Vec<T::TreeId> = vec![root.clone()];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            store.scoped_mut(&id_compressed[i].clone(), |x| {
                let l = x.children();
                let value = if l.map_or(false, |x| !x.is_empty()) {
                    cast(id_compressed.len()).unwrap()
                } else {
                    num_traits::zero()
                };
                id_first_child.push(value);
                if let Some(l) = l {
                    id_parent.extend(l.iter_children().map(|_| cast::<usize, IdD>(i).unwrap()));
                    id_compressed.extend(l.iter_children().cloned());
                }
            });

            i += 1;
        }

        BreathFirst {
            id_compressed,
            id_parent,
            id_first_child,
            phantom: PhantomData,
        }
    }
}

impl<T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
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

    fn child<S>(&self, store: &S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: for<'b> NodeStore<T::TreeId, R<'b> = T>,
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

    fn child4<S>(&self, store: &S, x: &IdD, p: &[<T as WithChildren>::ChildIdx]) -> IdD
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);

            let cs = store.scoped(&a, |n| n.child_count());
            if cs > zero() {
                r = self.first_child(&r).unwrap() + cast(*d).unwrap();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> NodeStore<T::TreeId, R<'b> = T>,
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
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        let cs = store.scoped(&self.original(x), |n| n.child_count());
        let l: usize = cast(cs).unwrap();
        let s: usize = cast(*x).unwrap();
        let r = s + 1..s + l;
        r.map(|x| cast::<usize, IdD>(x).unwrap())
            .collect::<Vec<_>>()
            .to_owned()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<T, IdD> for BreathFirst<T, IdD>
where
    T::TreeId: Clone,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        // TODO possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let child_count = store.scoped(&self.original(&id[i]), |x| x.child_count());
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
        S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        // TODO possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let child_count = store.scoped(&self.original(&id[i]), |x| x.child_count());
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
