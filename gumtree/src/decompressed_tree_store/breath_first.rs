use num_traits::{cast, one, zero, PrimInt};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{NodeStore, WithChildren};

use super::{DecompressedTreeStore, Initializable, Iter, ShallowDecompressedTreeStore};

use super::{BreathFirstContiguousSiblings, BreathFirstIterable, DecompressedWithParent};

/// vec of decompressed nodes layed out in pre order with contiguous siblings
pub struct BreathFirst<IdC, IdD: PrimInt> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
}

impl<'d, IdC: Clone, IdD: PrimInt> BreathFirstContiguousSiblings<'d, IdC, IdD>
    for BreathFirst<IdC, IdD>
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

impl<'a, IdC: Clone, IdD: PrimInt> BreathFirstIterable<'a, IdC, IdD> for BreathFirst<IdC, IdD> {
    type It = Iter<IdD>;
    fn iter_bf(&'a self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}
impl<'d, IdC: Clone, IdD: PrimInt> DecompressedWithParent<'d, IdC, IdD> for BreathFirst<IdC, IdD> {
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

    fn position_in_parent<'b, S>(
        &self,
        _store: &'b S,
        c: &IdD,
    ) -> <S::R<'b> as WithChildren>::ChildIdx
    where
        S: NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let p = self.parent(c).unwrap();
        cast(*c - self.first_child(&p).unwrap()).unwrap()
    }

    type PIt<'b>=IterParents<'b, IdD> where IdD: 'b, IdC:'b;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
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

impl<'d, IdC: Clone, IdD: PrimInt> Initializable<'d, IdC, IdD> for BreathFirst<IdC, IdD> {
    fn new<
        // 'a,
        // T: 'a + Tree<TreeId = IdC>,
        // HK: HashKind, HP: PrimInt,
        S, //: 'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
    >(
        store: &'d S,
        root: &IdC,
    ) -> Self
    where
        S: NodeStore<IdC>,
        // for<'a> <<S as NodeStore2<IdC>>::R as GenericItem<'a>>::Item: WithChildren<TreeId = IdC>,
        S::R<'d>: WithChildren<TreeId = IdC>,
    {
        let mut leaf_count = zero();
        let mut id_compressed: Vec<IdC> = vec![root.clone()];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            let node = store.resolve(&id_compressed[i]);
            let l = node.try_get_children();
            id_first_child.push(if l.map_or(0, |x| x.len()) > 0 {
                cast(id_compressed.len()).unwrap()
            } else {
                num_traits::zero()
            });
            if l.map_or(0, |x| x.len()) == 0 {
                leaf_count = leaf_count + one();
            }
            if let Some(l) = l {
                id_parent.extend(l.iter().map(|_| cast::<usize, IdD>(i).unwrap()));
                id_compressed.extend_from_slice(l);
            }

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

impl<'a, IdC: Clone, IdD: PrimInt> ShallowDecompressedTreeStore<'a, IdC, IdD>
    for BreathFirst<IdC, IdD>
{
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

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[<S::R<'b> as WithChildren>::ChildIdx]) -> IdD
    where
        S: NodeStore<IdC>,
        // for<'a> <<S as NodeStore2<IdC>>::R as GenericItem<'a>>::Item: WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);

            let cs: Vec<_> = {
                let n = store.resolve(&a);
                let cs = n.get_children();
                cs.to_owned()
            };
            if cs.len() > 0 {
                r = self.first_child(&r).unwrap() + cast(*d).unwrap();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        // for<'a> <<S as NodeStore2<IdC>>::R as GenericItem<'a>>::Item: WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
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

impl<'a, IdC: Clone, IdD: PrimInt> DecompressedTreeStore<'a, IdC, IdD> for BreathFirst<IdC, IdD> {
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        // for<'b> <<S as NodeStore2<IdC>>::R as GenericItem<'b>>::Item: WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>, // S: 'b + NodeStore2<T::TreeId, R<'b> = T>, //NodeStore<'b, T::TreeId, T>
    {
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

    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<IdC>,
        // for<'b> <<S as NodeStore2<IdC>>::R as GenericItem<'b>>::Item: WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
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
