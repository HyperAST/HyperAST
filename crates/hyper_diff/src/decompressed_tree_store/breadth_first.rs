use std::marker::PhantomData;

use num_traits::{cast, zero};

use hyperast::types::WithChildren;
use hyperast::types::{self, Childrn, HyperAST};
use hyperast::PrimInt;

use super::{BreadthFirstIt, DecompressedTreeStore, Iter, ShallowDecompressedTreeStore};

use super::{
    BreadthFirstContiguousSiblings, BreadthFirstIterable, DecompressedParentsLending,
    DecompressedWithParent,
};
use crate::matchers::Decompressible;

/// Decompressed subtree of an HyperAST layed out in breadth-first ie. contiguous siblings
pub struct BreadthFirst<IdN, IdD: PrimInt> {
    id_compressed: Vec<IdN>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
    phantom: PhantomData<*const IdN>,
}

impl<'d, HAST: HyperAST + Copy, IdD: PrimInt> BreadthFirstContiguousSiblings<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn has_children(&self, id: &IdD) -> bool {
        BreadthFirstContiguousSiblings::first_child(self, id) != None
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> BreadthFirstIt<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type It<'a> = Iter<IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> BreadthFirstIterable<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn iter_bf(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
{
    type PIt = IterParents<'a, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(self, root: &HAST::IdN) -> Self {
        let store = self.hyperast;
        let mut id_compressed: Vec<HAST::IdN> = vec![root.clone()];
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
        Decompressible {
            hyperast: store,
            decomp: BreadthFirst {
                id_compressed,
                id_parent,
                id_first_child,
                phantom: PhantomData,
            },
        }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn original(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn root(&self) -> IdD {
        zero()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        let store = self.hyperast;
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

    fn children(&self, x: &IdD) -> Vec<IdD> {
        let store = self.hyperast;
        let node = store.resolve(&self.original(x));
        let l: usize = cast(node.child_count()).unwrap();
        let s: usize = cast(*x).unwrap();
        let r = s + 1..s + l;
        r.map(|x| cast::<usize, IdD>(x).unwrap())
            .collect::<Vec<_>>()
            .to_owned()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, BreadthFirst<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        let store = self.hyperast;
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

    fn descendants_count(&self, x: &IdD) -> usize {
        let store = self.hyperast;
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
