use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref};

use num_traits::{cast, one, zero, ToPrimitive, Zero};

use hyperast::PrimInt;
use hyperast::{
    position::Position,
    types::{
        self, Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, NodeId, NodeStore,
        Stored, Tree, WithChildren, WithSerialization,
    },
};

use super::{
    basic_post_order::{BasicPOSlice, BasicPostOrder},
    CIdx, ContiguousDescendants, DecendantsLending, DecompressedParentsLending,
    DecompressedTreeStore, DecompressedWithParent, DecompressedWithSiblings,
    FullyDecompressedTreeStore, PostOrder, ShallowDecompressedTreeStore,
};

pub struct SimplePostOrder<T: Stored, IdD> {
    pub(super) basic: BasicPostOrder<T, IdD>,
    pub(super) id_parent: Box<[IdD]>,
}

// impl<'a, IdD> super::Persistable
//     for SimplePostOrder<hyperast::store::nodes::legion::HashedNodeRef<'a>, IdD>
// {
//     type Persisted = SimplePostOrder<
//         super::PersistedNode<
//             <hyperast::store::nodes::legion::HashedNodeRef<'a> as types::Stored>::TreeId,
//         >,
//         IdD,
//     >;

//     fn persist(self) -> Self::Persisted {
//         SimplePostOrder {
//             basic: self.basic.persist(),
//             id_parent: self.id_parent,
//         }
//     }

//     unsafe fn unpersist(this: Self::Persisted) -> Self {
//         Self {
//             basic: <BasicPostOrder<hyperast::store::nodes::legion::HashedNodeRef<'a>,IdD> as super::Persistable>::unpersist(this.basic),
//             id_parent: this.id_parent,
//         }
//     }
// }

impl<T: Stored, IdD> Deref for SimplePostOrder<T, IdD> {
    type Target = BasicPostOrder<T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl<T: Stored, IdD> SimplePostOrder<T, IdD> {
    pub fn as_slice(&self) -> SimplePOSlice<'_, T, IdD> {
        SimplePOSlice {
            basic: self.basic.as_slice(),
            id_parent: &self.id_parent,
        }
    }
}

/// WIP WithParent (need some additional offset computations)
pub struct SimplePOSlice<'a, T: Stored, IdD> {
    pub(super) basic: BasicPOSlice<'a, T, IdD>,
    #[allow(unused)] // WIP
    pub(super) id_parent: &'a [IdD],
}

impl<'a, T: Stored, IdD> Deref for SimplePOSlice<'a, T, IdD> {
    type Target = BasicPOSlice<'a, T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

// impl<T: Stored, IdD: PrimInt> SimplePostOrder<T, IdD> {
//     pub fn iter(&self) -> impl Iterator<Item = &T::TreeId> {
//         self.id_compressed.iter()
//     }
// }

impl<T: Stored, IdD: PrimInt> SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn _position_in_parent(&self, c: &IdD, p: &IdD) -> CIdx<'_, T, T::TreeId> {
        let mut r = 0;
        let mut c = *c;
        let min = self.first_descendant(p);
        loop {
            let lld = self.first_descendant(&c);
            if lld == min {
                break;
            }
            c = lld - one();
            r += 1;
        }
        cast(r).unwrap()
    }
}

// impl<'a, T: Stored, IdD: PrimInt> types::NLending<'a, T::TreeId> for SimplePostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

impl<'a, T: Stored, IdD: PrimInt> DecompressedParentsLending<'a, IdD> for SimplePostOrder<T, IdD>
where
// T: for<'t> types::NLending<'t, T::TreeId>,
// for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
// T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type PIt = IterParents<'a, IdD>;
}

impl<T: Stored, IdD: PrimInt> DecompressedWithParent<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        if id == &self.root() {
            None
        } else {
            Some(self.id_parent[id.to_usize().unwrap()])
        }
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.parent(id) != None
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        let p = self.parent(c)?;
        let p = self._position_in_parent(c, &p);
        Some(cast(p).expect("no integer overflow, Idx is too small"))
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        let ref this = self;
        let mut idxs = vec![];
        let mut curr = *descendant;
        while &curr != parent {
            let p = this
                .parent(&curr)
                .expect("reached root before given parent");
            let idx = this._position_in_parent(&curr, &p);
            idxs.push(cast(idx).expect("no integer overflow, Idx is too small"));
            curr = p;
        }
        idxs.reverse();
        idxs
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        let mut a = *a;
        let mut b = *b;
        loop {
            if a == b {
                return a;
            } else if a < b {
                a = self.parent(&a).unwrap();
            } else if b < self.root() {
                b = self.parent(&b).unwrap();
            } else {
                assert!(a == b);
                return a;
            }
        }
    }
}

impl<T: Stored, IdD: PrimInt> DecompressedWithSiblings<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        let p = self.parent(x)?;
        let p_lld = self.first_descendant(&p);
        SimplePostOrder::lsib(self, x, &p_lld)
    }
}

pub struct IterParents<'a, IdD> {
    id: IdD,
    id_parent: &'a [IdD],
}

impl<'a, IdD: PrimInt> Iterator for IterParents<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.id == cast(self.id_parent.len() - 1).unwrap() {
            return None;
        }
        let r = self.id_parent[self.id.to_usize().unwrap()];
        self.id = r.clone();
        Some(r)
    }
}

impl<T: Stored, IdD: PrimInt> PostOrder<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.basic.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.basic.tree(id)
    }
}

// impl<T: Stored, IdD: PrimInt> SimplePostOrder<T, IdD> {
//     pub(crate) fn size(&self, i: &IdD) -> IdD {
//         *i - self.llds[(*i).to_usize().unwrap()] + one()
//     }
// }

// impl<T: Stored, IdD: PrimInt> PostOrderIterable<T, IdD> for SimplePostOrder<T, IdD>
// where
//     T::TreeId: Debug,
// {
//     type It = Iter<IdD>;
//     fn iter_df_post(&self) -> Iter<IdD> {
//         Iter {
//             current: zero(),
//             len: (cast(self.id_compressed.len())).unwrap(),
//         }
//     }
// }

impl<'b, T: Stored, IdD: PrimInt> super::DecompressedSubtree<T> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: NodeId<IdN = T::TreeId>,
{
    type Out = Self;

    fn decompress<S>(store: &S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        let simple = SimplePostOrder::make(store, root);
        Self {
            basic: simple.basic,
            id_parent: simple.id_parent,
        }
    }

    fn decompress2<HAST>(store: &HAST, root: &<T as Stored>::TreeId) -> Self::Out
    where
        // T: for<'t> types::AstLending<'t>,
        // HAST: HyperAST<IdN = <T as Stored>::TreeId, TM = T>,

        T: for<'a> hyperast::types::AstLending<'a>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        HAST: HyperAST<IdN = T::TreeId, TM = T>,
    {
        let simple = SimplePostOrder::make(store, root);
        Self {
            basic: simple.basic,
            id_parent: simple.id_parent,
        }
    }
}

impl<T: Stored, IdD: PrimInt> SimplePostOrder<T, IdD>
where
    T::TreeId: NodeId<IdN = T::TreeId>,
    // <T as WithChildren>::ChildIdx: PrimInt,
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn make<S>(store: &S, root: &T::TreeId) -> Self
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        let aaa = Element::<
            T::TreeId,
            <<T as types::NLending<'_, T::TreeId>>::N as WithChildren>::ChildIdx,
            IdD,
        > {
            curr: root.clone(),
            idx: zero(),
            lld: IdD::zero(),
            children: vec![],
        };
        let mut stack = vec![aaa];
        let mut llds: Vec<IdD> = vec![];
        let mut id_compressed = vec![];
        let mut id_parent = vec![];
        while let Some(Element {
            curr,
            idx,
            lld,
            children,
        }) = stack.pop()
        {
            let l = {
                let x = store.resolve(&curr);
                x.children()
                    .filter(|x| !x.is_empty())
                    .map(|l| l.get(idx).cloned())
            };
            if let Some(Some(child)) = l {
                stack.push(Element {
                    curr,
                    idx: idx + one(),
                    lld,
                    children,
                });
                stack.push(Element {
                    curr: child.clone(),
                    idx: zero(),
                    lld: zero(),
                    children: vec![],
                });
            } else {
                let curr_idx = cast(id_compressed.len()).unwrap();
                let value = if l.is_some() {
                    curr_idx
                } else {
                    for x in children {
                        id_parent[x.to_usize().unwrap()] = curr_idx;
                    }
                    lld
                };
                if let Some(tmp) = stack.last_mut() {
                    if tmp.idx == one() {
                        tmp.lld = value;
                    }
                    tmp.children.push(curr_idx);
                }
                llds.push(value);
                id_compressed.push(curr);
                id_parent.push(zero());
            }
        }
        let id_compressed = id_compressed.into();
        let id_parent = id_parent.into();
        let llds = llds.into();
        SimplePostOrder {
            basic: BasicPostOrder {
                id_compressed,
                llds,
                _phantom: std::marker::PhantomData,
            },
            id_parent,
        }
    }
}

struct Element<IdC, Idx, IdD> {
    curr: IdC,
    idx: Idx,
    lld: IdD,
    children: Vec<IdD>,
}

impl<T: Stored, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let node = store.resolve(&a);
            let cs = node.children().filter(|x| !types::Childrn::is_empty(x));
            let Some(cs) = cs else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(cast(*d + one()).unwrap());
            let cs: Vec<T::TreeId> = cs.iter_children().collect();
            for x in cs {
                z += Self::size2(store, &x);
            }
            r = self.first_descendant(&r) + cast(z).unwrap() - one();
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
        //     r = store.scoped(&a, |node| {
        //         let cs = node.children().filter(|x| !x.is_empty());
        //         let Some(cs) = cs else {
        //             panic!("no children in this tree")
        //         };
        //         let mut z = 0;
        //         let cs = cs.before(*d + one());
        //         let cs: Vec<T::TreeId> = cs.iter_children().cloned().collect();
        //         for x in cs {
        //             z += Self::size4(store, &x);
        //         }
        //         self.first_descendant(&r) + cast(z).unwrap() - one()
        //     });
        // }
        // r
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let a = self.original(x);
        let node = store.resolve(&a);
        let cs_len = node.child_count().to_usize().unwrap();
        if cs_len == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs_len];
        let mut c = *x - one();
        let mut i = cs_len - 1;
        r[i] = c;
        while i > 0 {
            i -= 1;
            let s = self.size(&c);
            c = c - s;
            r[i] = c;
        }
        assert_eq!(
            self.lld(x).to_usize().unwrap(),
            self.lld(&c).to_usize().unwrap()
        );
        r
    }

    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        unimplemented!()
        // let a = self.original(x);
        // store.scoped(&a, |node| {
        //     let cs_len = node.child_count().to_usize().unwrap();
        //     if cs_len == 0 {
        //         return vec![];
        //     }
        //     let mut r = vec![zero(); cs_len];
        //     let mut c = *x - one();
        //     let mut i = cs_len - 1;
        //     r[i] = c;
        //     while i > 0 {
        //         i -= 1;
        //         let s = self.size(&c);
        //         c = c - s;
        //         r[i] = c;
        //     }
        //     assert_eq!(
        //         self.lld(x).to_usize().unwrap(),
        //         self.lld(&c).to_usize().unwrap()
        //     );
        //     r
        // })
    }
}

impl<T: Stored, IdD: PrimInt> FullyDecompressedTreeStore<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
}

impl<T: Stored, IdD: PrimInt> DecompressedTreeStore<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn descendants<S>(&self, _store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()] // TODO use ldd
    }

    fn descendants_count<S>(&self, _store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.basic.is_descendant(desc, of)
    }
}

impl<'a, T: Stored, IdD: PrimInt> DecendantsLending<'a> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type Slice = SimplePOSlice<'a, T, IdD>;
}

impl<T: Stored, IdD: PrimInt> ContiguousDescendants<T, IdD> for SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    // type Slice<'b>
    //     = SimplePOSlice<'b, T, IdD>
    // where
    //     Self: 'b;

    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        let range = self.slice_range(x);
        SimplePOSlice {
            id_parent: &self.id_parent[range.clone()],
            basic: BasicPOSlice {
                id_compressed: &self.id_compressed[range.clone()],
                llds: &self.llds[range],
                _phantom: std::marker::PhantomData,
            },
        }
    }
}

impl<T: Stored, IdD: PrimInt + Eq> SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    pub fn lsib(&self, c: &IdD, p_lld: &IdD) -> Option<IdD> {
        assert!(p_lld <= c, "{:?}<={:?}", p_lld.to_usize(), c.to_usize());
        let lld = self.first_descendant(c);
        assert!(lld <= *c);
        if lld.is_zero() {
            return None;
        }
        let sib = lld - num_traits::one();
        if &sib < p_lld {
            None
        } else {
            Some(sib)
        }
    }
}

impl<'a, T: Stored, IdD: PrimInt> SimplePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn size2<S>(store: &S, x: &T::TreeId) -> usize
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        let tmp = store.resolve(x);
        let Some(cs) = tmp.children() else {
            return 1;
        };

        let mut z = 0;
        for x in cs.iter_children() {
            z += Self::size2(store, &x);
        }
        z + 1
    }
    fn size4<S>(store: &S, x: &T::TreeId) -> usize
where
        // S: types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        unimplemented!()
        // store.scoped(x, |tmp| {
        //     let Some(cs) = tmp.children() else {
        //         return 1;
        //     };

        //     let mut z = 0;
        //     for x in cs.iter_children() {
        //         z += Self::size4(store, x);
        //     }
        //     z + 1
        // })
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for SimplePostOrder<T, IdD>
where
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("id_parent", &self.id_parent)
            .field("llds", &self.llds)
            .finish()
    }
}

pub struct RecCachedPositionProcessor<'a, T: Stored, IdD: Hash + Eq> {
    pub(crate) ds: &'a SimplePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, Position>,
}

impl<'a, T: Stored, IdD: PrimInt + Hash + Eq> From<(&'a SimplePostOrder<T, IdD>, T::TreeId)>
    for RecCachedPositionProcessor<'a, T, IdD>
{
    fn from((ds, root): (&'a SimplePostOrder<T, IdD>, T::TreeId)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

impl<'a, T: Stored, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &Position
    where
        // HAST: for<'t> types::AstLending<'t, RT = types::LendN<'t, T, T::TreeId>, IdN = T::TreeId>
        //     + HyperAST,
        // HAST: for<'t> HyperAST<IdN = T::TreeId, T<'t> = T, Label = T::Label>,
        T::TreeId: Debug + NodeId<IdN = T::TreeId>,
        // T::Type: Copy + Send + Sync,
        // T: WithSerialization,
        for<'t> <T as types::NLending<'t, T::TreeId>>::N:
            Tree<Label = HAST::Label> + WithSerialization,

        T: for<'t> hyperast::types::AstLending<'t>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        HAST: HyperAST<IdN = T::TreeId, TM = T>,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let id = self.ds.original(&p);
            let p_r = stores.node_store().resolve(&id);
            let p_t = stores.resolve_type(&id);
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = stores.node_store().resolve(&ori);
                    return self.cache.entry(*c).or_insert(Position::new(
                        stores
                            .label_store()
                            .resolve(&r.get_label_unchecked())
                            .into(),
                        0,
                        r.try_bytes_len().unwrap_or(0),
                    ));
                }
                let mut pos = self
                    .cache
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &p).clone());
                let r = stores.node_store().resolve(&ori);
                pos.inc_path(stores.label_store().resolve(&r.get_label_unchecked()));
                pos.set_len(r.try_bytes_len().unwrap_or(0));
                return self.cache.entry(*c).or_insert(pos);
            }

            let p_lld = self.ds.first_descendant(&p);
            if let Some(lsib) = self.ds.lsib(c, &p_lld) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let mut pos = self
                    .cache
                    .get(&lsib)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &lsib).clone());
                pos.inc_offset(pos.range().end - pos.range().start);
                let r = stores.node_store().resolve(&self.ds.original(&c));
                pos.set_len(r.try_bytes_len().unwrap());
                self.cache.entry(*c).or_insert(pos)
            } else {
                assert!(
                    self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = stores.node_store().resolve(&ori);
                    return self.cache.entry(*c).or_insert(Position::new(
                        "".into(),
                        0,
                        r.try_bytes_len().unwrap(),
                    ));
                }
                let mut pos = self
                    .cache
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &p).clone());
                let r = stores.node_store().resolve(&ori);
                pos.set_len(
                    r.try_bytes_len()
                        .unwrap_or_else(|| panic!("{:?}", stores.resolve_type(&ori))),
                );
                self.cache.entry(*c).or_insert(pos)
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            let r = stores.node_store().resolve(&ori);
            let t = stores.resolve_type(&ori);
            let pos = if t.is_directory() || t.is_file() {
                let file = stores
                    .label_store()
                    .resolve(&r.get_label_unchecked())
                    .into();
                let offset = 0;
                let len = r.try_bytes_len().unwrap_or(0);
                Position::new(file, offset, len)
            } else {
                let file = "".into();
                let offset = 0;
                let len = r.try_bytes_len().unwrap_or(0);
                Position::new(file, offset, len)
            };
            self.cache.entry(*c).or_insert(pos)
        }
    }
}
pub struct RecCachedProcessor<'a, T: Stored, IdD: Hash + Eq, U, F, G> {
    pub(crate) ds: &'a SimplePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, T: Stored, IdD: PrimInt + Hash + Eq, U, F, G>
    From<(&'a SimplePostOrder<T, IdD>, T::TreeId, F, G)>
    for RecCachedProcessor<'a, T, IdD, U, F, G>
{
    fn from((ds, root, with_p, with_lsib): (&'a SimplePostOrder<T, IdD>, T::TreeId, F, G)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
            with_p,
            with_lsib,
        }
    }
}

impl<'a, T: Stored, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, T, IdD, U, F, G>
where
    F: Fn(U, T::TreeId) -> U,
    G: Fn(U, T::TreeId) -> U,
{
    pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &U
    where
        T: for<'t> types::NLending<'t, T::TreeId>,
        // HAST: for<'t> types::AstLending<'t, RT = <T as types::NLending<'t, T::TreeId>>::N>
        //     + HyperAST<IdN = T::TreeId>,
        T::TreeId: Debug + NodeId<IdN = T::TreeId>,
        // T: Tree + WithSerialization,
        // HAST: for<'t> types::AstLending<'t, RT = types::LendN<'t, T, T::TreeId>, IdN = T::TreeId>
        //     + HyperAST,
        // HAST: for<'t> HyperAST<IdN = T::TreeId, T<'t> = T, Label = T::Label>,
        T::TreeId: NodeId<IdN = T::TreeId>,
        // T::Type: Copy + Send + Sync,
        // T: WithSerialization,
        for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithSerialization,
        T: for<'t> hyperast::types::AstLending<'t>,
        T: for<'t> types::NLending<'t, T::TreeId, N = <T as types::AstLending<'t>>::RT>,
        HAST: HyperAST<IdN = T::TreeId, TM = T>,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let id = self.ds.original(&p);
            let p_r = stores.node_store().resolve(&id);
            let p_t = stores.resolve_type(&id);
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                }
                let pos = self.position(stores, &p).clone();
                return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
            }

            let p_lld = self.ds.first_descendant(&p);
            if let Some(lsib) = self.ds.lsib(c, &p_lld) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position(stores, &lsib).clone();
                self.cache
                    .entry(*c)
                    .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
            } else {
                assert!(
                    self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                }
                let pos = self.position(stores, &p).clone();
                self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            self.cache
                .entry(*c)
                .or_insert((self.with_p)(Default::default(), ori))
        }
    }
}
