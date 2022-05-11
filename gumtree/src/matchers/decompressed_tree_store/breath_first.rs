use num_traits::{cast, one, zero, PrimInt};

use crate::tree::{
    tree::{NodeStore, Stored, Tree, WithChildren},
    tree_path::CompressedTreePath,
};

use super::{DecompressedTreeStore, Initializable, Iter, ShallowDecompressedTreeStore};

use super::{BreathFirstContiguousSiblings, BreathFirstIterable, DecompressedWithParent};

/// vec of decompressed nodes layed out in pre order with contiguous siblings
pub struct BreathFirst<IdC, IdD: PrimInt> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
}

impl<IdC: Clone, IdD: PrimInt> BreathFirstContiguousSiblings<IdC, IdD> for BreathFirst<IdC, IdD> {
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

impl<'a, IdC: Clone, IdD: PrimInt> BreathFirstIterable<'a, IdC, IdD> for BreathFirst<IdC, IdD> {
    type It = Iter<IdD>;
    fn iter_bf(&'a self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}
impl<IdC: Clone, IdD: PrimInt> DecompressedWithParent<IdD> for BreathFirst<IdC, IdD> {
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

    fn position_in_parent<T: WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        _store: &S,
        c: &IdD,
    ) -> T::ChildIdx {
        let p = self.parent(c).unwrap();
        cast(*c - self.first_child(&p).unwrap()).unwrap()
    }
}

impl<IdC: Clone, IdD: PrimInt> Initializable<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn new<
        T: Tree<TreeId = IdC>,
        // HK: HashKind, HP: PrimInt,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
        let mut leaf_count = zero();
        let mut id_compressed: Vec<IdC> = vec![root.clone()];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            let node = store.resolve(&id_compressed[i]);
            let l = node.get_children();
            id_first_child.push(if l.len() > 0 {
                cast(id_compressed.len()).unwrap()
            } else {
                num_traits::zero()
            });
            if l.len() == 0 {
                leaf_count = leaf_count + one();
            }
            id_parent.extend(l.iter().map(|_| cast::<usize, IdD>(i).unwrap()));
            id_compressed.extend_from_slice(l);

            i += 1;
        }

        BreathFirst {
            leaf_count,
            id_compressed,
            id_parent,
            id_first_child,
        }
    }
}

impl<IdC: Clone, IdD: PrimInt> ShallowDecompressedTreeStore<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn leaf_count(&self) -> IdD {
        self.leaf_count
    }

    fn root(&self) -> IdD {
        zero()
    }

    fn child<T: Stored<TreeId = IdC> + WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.resolve(&a).get_children().to_owned();
            if cs.len() > 0 {
                r = self.first_child(&r).unwrap() + cast(*d).unwrap();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<
        T: Stored<TreeId = IdC> + WithChildren,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        let node = store.resolve(&self.original(x));
        let l: usize = cast(node.child_count()).unwrap();
        let s: usize = cast(*x).unwrap();
        let r = s + 1..s + l;
        r.map(|x| cast::<usize, IdD>(x).unwrap())
            .collect::<Vec<_>>()
            .to_owned()
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> CompressedTreePath<Idx> {
        todo!()
    }
}

impl<IdC: Clone, IdD: PrimInt> DecompressedTreeStore<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn descendants<
        T: Stored<TreeId = IdC> + WithChildren,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        // todo possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let node = store.resolve(&self.original(&id[i]));
            let l: usize = cast(node.child_count()).unwrap();
            let s: usize = cast(id[i]).unwrap();
            let r = s + 1..s + l;
            id.extend(r.map(|x| cast::<usize, IdD>(x).unwrap()));
            i += 1;
        }
        id
    }

    fn descendants_count<
        T: Stored<TreeId = IdC> + WithChildren,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize {
        // todo possible opti by also making descendants contiguous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let node = store.resolve(&self.original(&id[i]));
            let l: usize = cast(node.child_count()).unwrap();
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
}
