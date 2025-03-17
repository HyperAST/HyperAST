use std::{collections::HashMap, fmt::Debug, hash::Hash};

use num_traits::{cast, one, zero, ToPrimitive, Zero};

use crate::matchers::Decompressible;

use super::{
    basic_post_order::{BasicPOSlice, BasicPostOrder},
    complete_post_order::CompletePOSlice,
    simple_post_order::{SimplePOSlice, SimplePostOrder},
    ContiguousDescendants, DecendantsLending, DecompressedParentsLending, DecompressedTreeStore,
    DecompressedWithParent, DecompressedWithSiblings, Iter, LazyDecompressed,
    LazyDecompressedTreeStore, LazyPOBorrowSlice, LazyPOSliceLending, PostOrder, PostOrderIterable,
    Shallow, ShallowDecompressedTreeStore,
};
use hyperast::{
    position::Position,
    types::{
        self, AstLending, Children, Childrn, HyperAST, HyperASTShared, WithChildren, WithStats,
    },
    PrimInt,
};

pub struct LazyPostOrder<IdN, IdD> {
    pub(super) id_compressed: Box<[IdN]>,
    pub id_parent: Box<[IdD]>,
    /// leftmost leaf descendant of nodes
    pub(crate) llds: Box<[IdD]>,
    _phantom: std::marker::PhantomData<IdN>,
}

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &IdN> {
        self.id_compressed.iter()
    }
}

impl<IdN: Debug, IdD: PrimInt + Debug> Debug for LazyPostOrder<IdN, IdD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed.len())
            .field("id_parent", &self.id_parent.len())
            .field("llds", &self.llds.len())
            // .field("id_compressed", &self.id_compressed)
            // .field("id_parent", &self.id_parent)
            // .field("llds", &self.llds)
            .finish()
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    pub(super) fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        let p = self.parent(c)?;
        let mut r = 0;
        let mut c = *c;
        let min = self.first_descendant(&p);
        loop {
            let lld = self.first_descendant(&c);
            if lld == min {
                break;
            }
            c = lld - one();
            r += 1;
        }
        Some(cast(r).unwrap())
    }
}

mod impl_ref {
    use super::*;

    impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    {
        fn len(&self) -> usize {
            self._len()
        }

        fn original(&self, id: &IdD) -> HAST::IdN {
            self.id_compressed[id.to_usize().unwrap()].clone()
        }

        fn root(&self) -> IdD {
            cast(self._root()).unwrap()
        }

        fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
            let store = self.hyperast;
            let mut r = *x;
            for d in p {
                let a = self.original(&r);
                let node = store.resolve(&a);
                let cs = node.children().filter(|x| x.is_empty());
                let Some(cs) = cs else {
                    panic!("no children in this tree")
                };
                let mut z = 0;
                let cs = cs.before(cast(*d + one()).unwrap());
                let cs: Vec<_> = cs.iter_children().collect();
                for x in cs {
                    z += self.size2(self.hyperast, &x);
                }
                r = self._first_descendant(&r) + cast(z).unwrap() - one();
            }
            r
        }

        fn children(&self, x: &IdD) -> Vec<IdD>
where
    // S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
    //     + NodeStore<T::TreeId>,
        {
            debug_assert!(
                self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero(),
                "x has not been initialized"
            );
            let a = self.original(x);
            let node = self.hyperast.resolve(&a);
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
                let s = self._size(&c);
                c = c - s;
                r[i] = c;
            }
            assert_eq!(
                self._lld(x.to_usize().unwrap()).to_usize().unwrap(),
                self._lld(c.to_usize().unwrap()).to_usize().unwrap()
            );
            r
        }
    }

    impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    {
        type PIt = IterParents<'a, IdD>;
    }

    impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    {
        fn parent(&self, id: &IdD) -> Option<IdD> {
            if id == &ShallowDecompressedTreeStore::root(self) {
                None
            } else {
                Some(self.id_parent[id.to_usize().unwrap()])
            }
        }

        fn has_parent(&self, id: &IdD) -> bool {
            self.parent(id) != None
        }

        fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
            let i = self._position_in_parent(c, &self.parent(c)?);
            Some(cast(i).unwrap())
        }

        fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
            IterParents {
                id,
                id_parent: &self.id_parent,
            }
        }
        fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
            let ref this = self;
            let mut idxs: Vec<Idx> = vec![];
            let mut curr = *descendant;
            while &curr != parent {
                let p = this
                    .parent(&curr)
                    .expect("reached root before given parent");
                let idx = this._position_in_parent(&curr, &p);
                idxs.push(cast(idx).unwrap());
                curr = p;
            }
            idxs.reverse();
            idxs
        }

        fn lca(&self, _a: &IdD, _b: &IdD) -> IdD {
            todo!()
        }
    }

    impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    {
        fn descendants(&self, x: &IdD) -> Vec<IdD> {
            (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
                .map(|x| cast(x).unwrap())
                .collect()
        }

        fn first_descendant(&self, i: &IdD) -> IdD {
            self.llds[(*i).to_usize().unwrap()]
        }

        fn descendants_count(&self, x: &IdD) -> usize {
            (self._size(x)).to_usize().unwrap() - 1
        }

        fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
            desc < of && &self.first_descendant(of) <= desc
        }
    }

    impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    {
        fn lld(&self, i: &IdD) -> IdD {
            self.llds[(*i).to_usize().unwrap()]
        }

        fn tree(&self, id: &IdD) -> HAST::IdN {
            self.id_compressed[id.to_usize().unwrap()].clone()
        }
    }

    impl<HAST: HyperAST + Copy, IdD: PrimInt + Debug> PostOrderIterable<HAST, IdD>
        for Decompressible<HAST, &LazyPostOrder<HAST::IdN, IdD>>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    {
        type It = Iter<IdD>;
        fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
            let len = if ROOT {
                cast(self.id_compressed.len()).unwrap()
            } else {
                self.root()
            };
            Iter {
                current: zero(),
                len,
            }
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
{
    type PIt = IterParents<'a, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        if id == &ShallowDecompressedTreeStore::root(self) {
            None
        } else {
            Some(self.id_parent[id.to_usize().unwrap()])
        }
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.parent(id) != None
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.position_in_parent(c)
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }
    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        let ref this = self;
        let mut idxs: Vec<Idx> = vec![];
        let mut curr = *descendant;
        while &curr != parent {
            let p = this
                .parent(&curr)
                .expect("reached root before given parent");
            let idx = this._position_in_parent(&curr, &p);
            idxs.push(cast(idx).unwrap());
            curr = p;
        }
        idxs.reverse();
        idxs
    }

    fn lca(&self, _a: &IdD, _b: &IdD) -> IdD {
        todo!()
    }
}

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    fn _position_in_parent(&self, c: &IdD, p: &IdD) -> usize {
        let mut r = 0;
        let mut c = *c;
        let min = self._first_descendant(p);
        loop {
            let lld = self._first_descendant(&c);
            if lld == min {
                break;
            }
            c = lld - one();
            r += 1;
        }
        cast(r).unwrap()
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithSiblings<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        let p = self.parent(x)?;
        let p_lld = self.first_descendant(&p);
        self.lsib(x, &p_lld)
    }
}

pub struct IterParents<'a, IdD> {
    pub(super) id: IdD,
    pub(super) id_parent: &'a [IdD],
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub(crate) fn _size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }

    pub(crate) fn _lld(&self, i: usize) -> IdD {
        self.llds[i]
    }
    pub(crate) fn _len(&self) -> usize {
        self.id_parent.len()
    }
    pub(crate) fn _root(&self) -> usize {
        self._len() - 1
    }
}

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub fn len(&self) -> usize {
        self._len()
    }

    pub fn root(&self) -> IdD {
        cast(self._len() - 1).unwrap()
    }
}

impl<IdN: Clone, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub fn original(&self, id: &IdD) -> IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'a, IdN, IdD: PrimInt + Shallow<IdD> + Debug> LazyPostOrder<IdN, IdD> {
    // #[time("warn")]
    pub fn complete<HAST: HyperAST<IdN = IdN> + Copy>(
        mut self,
        store: HAST,
    ) -> SimplePostOrder<IdN, IdD>
    where
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
        HAST::IdN: Debug,
        for<'t> <HAST as AstLending<'t>>::RT: WithStats,
    {
        let mut dec = Decompressible {
            hyperast: store,
            decomp: &mut self,
        };
        let root = dec.root();
        dec.complete_subtree(&root);
        SimplePostOrder {
            basic: BasicPostOrder {
                id_compressed: self.id_compressed,
                llds: self.llds,
            },
            id_parent: self.id_parent,
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD> + Debug>
    Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Debug,
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    fn continuous_aux(&mut self, x: &IdD) -> Option<Vec<HAST::IdN>> {
        let store = self.hyperast;
        let node = store.resolve(&self.original(x));
        self.llds[x.to_usize().unwrap()] = *x + one() - cast(node.size()).unwrap();
        let cs = node.children()?;
        let cs_len = cs.child_count().to_usize().unwrap();
        if cs_len == 0 {
            return None;
        }
        let c = *x - one();
        let i = cs_len - 1;
        let c_n = &cs[cast(i).unwrap()];
        let offset = c.to_usize().unwrap();
        self.id_compressed[offset] = c_n.clone();
        self.id_parent[offset] = x.clone();
        let r = cs
            .before(cs.child_count() - one())
            .iter_children()
            .collect();
        Some(r)
    }

    fn decompress_descendants_continuous(&mut self, x: &<Self as LazyDecompressed<IdD>>::IdD) {
        let store = self.hyperast;
        // PAVE CESAR oO
        let mut c = *x;
        let mut s: Vec<(IdD, Vec<HAST::IdN>)> = vec![];
        loop {
            // Termination: when s is empty, done by second loop
            loop {
                // Termination: go down the last child, finish when no more remains
                let rem = self.continuous_aux(&c);
                // let ran = self.first_descendant(&c).to_usize().unwrap()..=c.to_usize().unwrap();
                // println!(
                //     "{}",
                //     ran.clone()
                //         .collect::<Vec<_>>()
                //         .iter()
                //         .zip(self.id_parent[ran.clone()].iter())
                //         .zip(self.llds[ran.clone()].iter())
                //         .map(|((x1, x2), x3)| (x1, x2, x3))
                //         .fold(" i, p, l".to_string(), |s, x| format!("{s}\n{:?}", x))
                // );
                let Some(rem) = rem else {
                    if c > zero() {
                        c = c - one();
                    }
                    break;
                };
                s.push((c, rem));
                c = c - one();
            }
            let mut next = None;
            loop {
                // Termination: either rem diminish, or if rem is empty s diminish
                let Some((p, mut rem)) = s.pop() else {
                    break;
                };
                let Some(z) = rem.pop() else {
                    assert!(c <= self.lld(&p));
                    continue;
                };
                assert!(self.lld(&p) <= c);
                next = Some((p, z));
                s.push((p, rem));
                break;
            }
            let Some((p, z)) = next else {
                assert!(
                    self._size(x) <= one() || self.tree(&(c + one())) != self.tree(x),
                    "{:?} {:?}",
                    self.tree(&(c + one())),
                    self.tree(x)
                );
                assert!(c == self.lld(x) || c + one() == self.lld(x));
                break;
            };
            self.id_parent[c.to_usize().unwrap()] = p;
            self.id_compressed[c.to_usize().unwrap()] = z;
        }
    }

    pub fn decompress_descendants(&mut self, x: &IdD) {
        let store = self.hyperast;
        let mut q = vec![x.clone()];
        while let Some(x) = q.pop() {
            assert!(self.id_parent[x.to_usize().unwrap()] != zero());
            q.extend(self.decompress_children(&x));
        }
    }
    // WARN just used for debugging
    // TODO remove
    pub fn go_through_descendants(&mut self, x: &IdD) {
        let store = self.hyperast;
        let mut q = vec![x.clone()];
        while let Some(x) = q.pop() {
            assert!(self.id_parent[x.to_usize().unwrap()] != zero());
            assert_eq!(
                self._size(&x).to_usize().unwrap(),
                store.resolve(&self.original(&x)).size()
            );
            q.extend(self.children(&x));
        }
    }
    pub fn complete_subtree(&mut self, x: &IdD) {
        assert!(
            self.parent(x).map_or(true, |p| p != zero()),
            "x is not initialized"
        );
        // self.decompress_descendants_continuous(store, &x);
        // // self.go_through_descendants(store, &x);
        let first = self.first_descendant(x);
        let mut i = x.clone();
        while i > first {
            // dbg!(i);
            // dbg!(self.parent(&i));
            // dbg!(self.lld(&i));
            if self.id_parent[i.to_usize().unwrap() - 1] != zero() {
                i = i - one();
            } else {
                assert!(
                    self.parent(&i).map_or(true, |p| p != zero()),
                    "i is not initialized"
                );

                // self.decompress_descendants(store, &i);
                self.decompress_descendants_continuous(&i);
                i = self.lld(&i);
            }
            if i == first {
                break;
            }
        }
        self.decompress_children(x).len();
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Debug> types::DecompressedFrom<HAST>
    for LazyPostOrder<HAST::IdN, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    type Out = Self;

    // #[time("warn")]
    fn decompress(hyperast: HAST, root: &HAST::IdN) -> Self {
        let store = hyperast;
        let pred_len = store.resolve(root).size();
        let mut simple = LazyPostOrder {
            id_compressed: init_boxed_slice(root.clone(), pred_len), // TODO micro bench it and maybe use uninit
            id_parent: init_boxed_slice(zero(), pred_len),
            llds: init_boxed_slice(zero(), pred_len),
            _phantom: Default::default(),
        };
        simple.id_compressed[simple._root()] = root.clone();
        simple.id_parent[simple._root()] = cast(simple._root()).unwrap();
        simple.llds[simple._root()] = zero();
        simple
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, LazyPostOrder<HAST::IdN, IdD>>
where
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    type Out = Self;
    fn decompress(self, root: &HAST::IdN) -> Self {
        let store = self.hyperast;
        let pred_len = store.resolve(root).size();
        let mut simple = LazyPostOrder {
            id_compressed: init_boxed_slice(root.clone(), pred_len), // TODO micro bench it and maybe use uninit
            id_parent: init_boxed_slice(zero(), pred_len),
            llds: init_boxed_slice(zero(), pred_len),
            _phantom: Default::default(),
        };
        simple.id_compressed[simple._root()] = root.clone();
        simple.id_parent[simple._root()] = cast(simple._root()).unwrap();
        simple.llds[simple._root()] = zero();
        Decompressible {
            hyperast: store,
            decomp: simple,
        }
    }
}

pub(super) fn init_boxed_slice<T: Clone>(value: T, pred_len: usize) -> Box<[T]> {
    let mut v = Vec::with_capacity(pred_len);
    v.resize(pred_len, value);
    v.into_boxed_slice()
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self._len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn root(&self) -> IdD {
        cast(self._root()).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD
    {
        let store = self.hyperast;
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let node = store.resolve(&a);
            let cs = node.children().filter(|x| x.is_empty());
            let Some(cs) = cs else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(cast(*d + one()).unwrap());
            let cs: Vec<_> = cs.iter_children().collect();
            for x in cs {
                z += self.size2(self.hyperast, &x);
            }
            r = self.first_descendant(&r) + cast(z).unwrap() - one();
        }
        r
    }

    fn children(&self, x: &IdD) -> Vec<IdD>
    {
        debug_assert!(
            self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero(),
            "x has not been initialized"
        );
        let a = self.original(x);
        let node = self.hyperast.resolve(&a);
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
            let s = self._size(&c);
            c = c - s;
            r[i] = c;
        }
        assert_eq!(
            self._lld(x.to_usize().unwrap()).to_usize().unwrap(),
            self._lld(c.to_usize().unwrap()).to_usize().unwrap()
        );
        r
    }
}

impl<'d, IdN, IdD: PrimInt + Shallow<IdD> + Debug> LazyPostOrder<IdN, IdD>
{
    pub fn child_decompressed<HAST>(
        mut self,
        store: HAST,
        x: &IdD,
        p: impl Iterator<Item = HAST::Idx>,
    ) -> IdD
    where
        HAST: HyperAST<IdN = IdN> + Copy,
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
        for<'t> <HAST as AstLending<'t>>::RT: WithStats,
    {
        let mut r = *x;
        for d in p {
            let mut dec = Decompressible {
                decomp: &mut self,
                hyperast: store,
            };
            let cs = dec.decompress_children(&r);
            let mut z: IdD = zero();
            let cs = cs
                .get(..(d + one()).to_usize().unwrap())
                .expect("no child corresponding to given path");
            for x in cs {
                z = z + self._size(x); //Self::size2(store, x);
            }
            let dec = Decompressible {
                decomp: &mut self,
                hyperast: store,
            };
            r = dec.first_descendant(&r) + cast(z).unwrap() - one();
        }
        r
    }
}

impl<'d, HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD> + Debug>
    Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    pub fn child_decompressed(&mut self, x: &IdD, p: impl Iterator<Item = HAST::Idx>) -> IdD {
        let mut r = *x;
        for d in p {
            let cs = self.decompress_children(&r);
            let mut z: IdD = zero();
            let cs = cs
                .get(..(d + one()).to_usize().unwrap())
                .expect("no child corresponding to given path");
            for x in cs {
                z = z + self._size(x); //Self::size2(store, x);
            }
            r = self.first_descendant(&r) + cast(z).unwrap() - one();
        }
        r
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD>> LazyDecompressed<IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
{
    type IdD = IdD;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD> + Debug>
    LazyDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    fn starter(&self) -> Self::IdD {
        ShallowDecompressedTreeStore::root(self)
    }

    fn decompress_children(&mut self, x: &Self::IdD) -> Vec<Self::IdD> {
        let store = self.hyperast;
        debug_assert!(
            self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero(),
            "x has not been initialized"
        );
        let node = store.resolve(&self.original(x));
        let Some(cs) = node.children() else {
            return vec![];
        };
        let cs_len = cs.child_count().to_usize().unwrap();
        if cs_len == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs_len];
        let mut c = *x - one();
        let mut i = cs_len - 1;
        loop {
            let c_n = &cs[cast(i).unwrap()];
            let s = store.resolve(c_n).size();
            let offset = c.to_usize().unwrap();
            r[i] = c;
            self.id_compressed[offset] = c_n.clone();
            self.id_parent[offset] = x.clone();
            self.llds[offset] = c + one() - cast(s).unwrap();
            if i == 0 {
                break;
            }
            c = c - cast(s).unwrap();
            i -= 1;
        }

        assert_eq!(
            self._lld(x.to_usize().unwrap()).to_usize().unwrap(),
            self._lld(c.to_usize().unwrap()).to_usize().unwrap()
        );
        r
    }

    fn decompress_to(&mut self, x: &IdD) -> Self::IdD {
        let mut p = *x;
        // TODO do some kind of dichotomy
        loop {
            if self.is_decompressed(&p) {
                while x < &self.lld(&p) {
                    p = self.parent(&p).unwrap();
                }
                break;
            }
            p = p + one();
        }
        while &p > x {
            debug_assert!(&self.lld(&p) <= x);
            let cs = self.decompress_children(&p);
            let cs = cs.into_iter().rev();
            // TODO do some kind of dichotomy
            for a in cs {
                if &a < x {
                    break;
                }
                p = a;
                if &a == x {
                    break;
                }
            }
        }
        assert_eq!(&p, x);
        *x
    }
}

impl<IdN, IdD: PrimInt + Shallow<IdD> + Debug> LazyPostOrder<IdN, IdD>
{
    fn is_decompressed(&self, x: &IdD) -> bool {
        self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero()
    }
}

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub(crate) fn _first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        (self._size(x)).to_usize().unwrap() - 1
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecendantsLending<'a>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
{
    type Slice = SimplePOSlice<'a, HAST::IdN, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ContiguousDescendants<HAST, IdD, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    /// WIP
    fn slice(&self, _x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        // Would need to complete the subtree under x, need the node store to do so
        // TODO could also make a lazy slice !!
        todo!()
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    pub(super) fn slice_range(&self, x: &IdD) -> std::ops::RangeInclusive<usize> {
        self.first_descendant(x).to_usize().unwrap()..=x.to_usize().unwrap()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> LazyPOSliceLending<'a, HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type SlicePo = Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, bitvec::boxed::BitBox>>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD> + Debug> LazyPOBorrowSlice<HAST, IdD, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Debug,
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
{
    fn slice_po(&mut self, x: &IdD) -> <Self as LazyPOSliceLending<'_, HAST, IdD>>::SlicePo
    {
        self.complete_subtree(x);
        let range = self.slice_range(x);
        let basic = BasicPOSlice {
            id_compressed: &self.id_compressed[range.clone()],
            llds: &self.llds[range.clone()],
        };
        let kr = Decompressible {
            hyperast: self.hyperast,
            decomp: basic,
        }
        .compute_kr_bitset();
        let simple = SimplePOSlice {
            basic,
            id_parent: &self.id_parent[range],
        };
        Decompressible {
            hyperast: self.hyperast,
            decomp: CompletePOSlice { simple, kr },
        }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Debug> PostOrderIterable<HAST, IdD>
    for Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        let len = if ROOT {
            cast(self.id_compressed.len()).unwrap()
        } else {
            self.root()
        };
        Iter {
            current: zero(),
            len,
        }
    }
}

pub struct RecCachedPositionProcessor<'a, HAST: HyperASTShared + Copy, IdD: Hash + Eq> {
    pub(crate) ds: Decompressible<HAST, &'a LazyPostOrder<HAST::IdN, IdD>>,
    root: HAST::IdN,
    cache: HashMap<IdD, Position>,
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Eq>
    Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
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

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD>
where
    IdN: types::NodeId<IdN = IdN>,
{
    fn size2<HAST: HyperAST<IdN = IdN> + Copy>(&self, store: HAST, x: &IdN) -> usize {
        let tmp = store.resolve(x);
        let Some(cs) = tmp.children() else {
            return 1;
        };

        let mut z = 0;
        for x in cs.iter_children() {
            z += self.size2(store, &x);
        }
        z + 1
    }
}

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq>
//     From<(&'a LazyPostOrder<T, IdD>, T::TreeId)> for RecCachedPositionProcessor<'a, T, IdD>
// {
//     fn from((ds, root): (&'a LazyPostOrder<T, IdD>, T::TreeId)) -> Self {
//         Self {
//             ds,
//             root,
//             cache: Default::default(),
//         }
//     }
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
// {
//     pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &Position
//     where
//         // HAST: for<'t> types::AstLending<'t, RT = hyperast::types::LendN<'t, T, T::TreeId>>,
//         HAST: HyperAST<IdN = T::TreeId, TM = T>,
//         for<'t> <HAST as types::AstLending<'t>>::RT: WithSerialization + Labeled + WithStats,
//         // S: NodeStore<T::TreeId, R<'b> = T>,
//         T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
//         // LS: LabelStore<str>,
//         // T: Tree<Label = LS::I> + WithSerialization,
//     {
//         if self.cache.contains_key(&c) {
//             return self.cache.get(&c).unwrap();
//         } else if let Some(p) = self.ds.parent(c) {
//             let id = self.ds.original(&p);
//             let p_r = stores.node_store().resolve(&id);
//             let p_t = stores.resolve_type(&id);
//             if p_t.is_directory() {
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     let r = stores.node_store().resolve(&ori);
//                     return self.cache.entry(*c).or_insert(Position::new(
//                         stores
//                             .label_store()
//                             .resolve(&r.get_label_unchecked())
//                             .into(),
//                         0,
//                         WithSerialization::try_bytes_len(&r).unwrap_or(0),
//                     ));
//                 }
//                 let mut pos = self
//                     .cache
//                     .get(&p)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &p).clone());
//                 let r = stores.node_store().resolve(&ori);
//                 pos.inc_path(stores.label_store().resolve(&r.get_label_unchecked()));
//                 pos.set_len(r.try_bytes_len().unwrap_or(0));
//                 return self.cache.entry(*c).or_insert(pos);
//             }

//             let p_lld = self.ds.first_descendant(&p);
//             if let Some(lsib) = LazyPostOrder::lsib(self.ds, c, &p_lld) {
//                 assert_ne!(lsib.to_usize(), c.to_usize());
//                 let mut pos = self
//                     .cache
//                     .get(&lsib)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &lsib).clone());
//                 pos.inc_offset(pos.range().end - pos.range().start);
//                 let r = stores.node_store().resolve(&self.ds.original(&c));
//                 pos.set_len(r.try_bytes_len().unwrap());
//                 self.cache.entry(*c).or_insert(pos)
//             } else {
//                 assert!(
//                     self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
//                     "{:?}",
//                     self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
//                 );
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     let r = stores.node_store().resolve(&ori);
//                     return self.cache.entry(*c).or_insert(Position::new(
//                         "".into(),
//                         0,
//                         r.try_bytes_len().unwrap(),
//                     ));
//                 }
//                 let mut pos = self
//                     .cache
//                     .get(&p)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &p).clone());
//                 let r = stores.node_store().resolve(&ori);
//                 pos.set_len(
//                     r.try_bytes_len()
//                         .unwrap_or_else(|| panic!("{:?}", stores.resolve_type(&ori))),
//                 );
//                 self.cache.entry(*c).or_insert(pos)
//             }
//         } else {
//             let ori = self.ds.original(&c);
//             assert_eq!(self.root, ori);
//             let r = stores.node_store().resolve(&ori);
//             let t = stores.resolve_type(&ori);
//             let pos = if t.is_directory() || t.is_file() {
//                 let file = stores
//                     .label_store()
//                     .resolve(&r.get_label_unchecked())
//                     .into();
//                 let offset = 0;
//                 let len = r.try_bytes_len().unwrap_or(0);
//                 Position::new(file, offset, len)
//             } else {
//                 let file = "".into();
//                 let offset = 0;
//                 let len = r.try_bytes_len().unwrap_or(0);
//                 Position::new(file, offset, len)
//             };
//             self.cache.entry(*c).or_insert(pos)
//         }
//     }
// }
// pub struct RecCachedProcessor<'a, T: Stored, IdD: Hash + Eq, U, F, G> {
//     pub(crate) ds: &'a LazyPostOrder<T, IdD>,
//     root: T::TreeId,
//     cache: HashMap<IdD, U>,
//     with_p: F,
//     with_lsib: G,
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq, U, F, G>
//     From<(&'a LazyPostOrder<T, IdD>, T::TreeId, F, G)> for RecCachedProcessor<'a, T, IdD, U, F, G>
// {
//     fn from((ds, root, with_p, with_lsib): (&'a LazyPostOrder<T, IdD>, T::TreeId, F, G)) -> Self {
//         Self {
//             ds,
//             root,
//             cache: Default::default(),
//             with_p,
//             with_lsib,
//         }
//     }
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
//     RecCachedProcessor<'a, T, IdD, U, F, G>
// where
//     F: Fn(U, T::TreeId) -> U,
//     G: Fn(U, T::TreeId) -> U,
// {
//     pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &U
//     where
//         // HAST: for<'t> HyperAST<IdN = T::TreeId, T<'t> = T, Label = T::Label>,
//         T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
//         // T: Tree + WithSerialization,
//         T: for<'t> types::NLending<'t, T::TreeId>,
//         for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,

//         HAST: HyperAST<IdN = T::TreeId, TM = T>,
//         for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithSerialization + Labeled + WithStats,
//     {
//         if self.cache.contains_key(&c) {
//             return self.cache.get(&c).unwrap();
//         } else if let Some(p) = self.ds.parent(c) {
//             let id = self.ds.original(&p);
//             let p_r = stores.node_store().resolve(&id);
//             let p_t = stores.resolve_type(&id);
//             if p_t.is_directory() {
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     return self
//                         .cache
//                         .entry(*c)
//                         .or_insert((self.with_p)(Default::default(), ori));
//                 }
//                 let pos = self.position(stores, &p).clone();
//                 return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
//             }

//             let p_lld = self.ds.first_descendant(&p);
//             if let Some(lsib) = self.ds.lsib(c, &p_lld) {
//                 assert_ne!(lsib.to_usize(), c.to_usize());
//                 let pos = self.position(stores, &lsib).clone();
//                 self.cache
//                     .entry(*c)
//                     .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
//             } else {
//                 assert!(
//                     self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
//                     "{:?}",
//                     self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
//                 );
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     return self
//                         .cache
//                         .entry(*c)
//                         .or_insert((self.with_p)(Default::default(), ori));
//                 }
//                 let pos = self.position(stores, &p).clone();
//                 self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
//             }
//         } else {
//             let ori = self.ds.original(&c);
//             assert_eq!(self.root, ori);
//             self.cache
//                 .entry(*c)
//                 .or_insert((self.with_p)(Default::default(), ori))
//         }
//     }
// }

// &mut wrappers
// use std::{ops::DerefMut};

// impl<'a, T, IdD: PrimInt + Debug, AAA:DerefMut<Target=LazyPostOrder<T, IdD>>> super::DecompressedSubtree<T> for AAA
// where
//     T: Stored + WithStats,
//     T::TreeId: Clone + Debug,
//     <T as WithChildren>::ChildIdx: PrimInt,
// {
//     fn decompress<S>(store: &'a S, id: &<T as Stored>::TreeId) -> Self::Out
//     where
//         S: NodeStore<<T as Stored>::TreeId, R<'a> = T>,
//     {
//         <LazyPostOrder<T, IdD> as super::DecompressedSubtree<T>>::decompress(store, id)
//     }

//     type Out=LazyPostOrder<T, IdD>;
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> types::NLending<'a, T::TreeId> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecendantsLending<'a> for &mut LazyPostOrder<T, IdD>
// where
//     // T: for<'t> types::NLending<'t, T::TreeId>,
//     // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
//     // T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     type Slice = SimplePOSlice<'a, T, IdD>;
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> super::DecompressedSubtree<T>
//     for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Clone + Debug,
//     // <T as WithChildren>::ChildIdx: PrimInt,
// {
//     fn decompress<S>(store: &S, id: &<T as Stored>::TreeId) -> Self::Out
//     where
//         S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
//             + types::NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD> as super::DecompressedSubtree<T>>::decompress(store, id)
//     }

//     fn decompress2<HAST>(store: &HAST, id: &<T as Stored>::TreeId) -> Self::Out
//     where
//         T: for<'t> types::AstLending<'t>,
//         HAST: HyperAST<IdN = <T as Stored>::TreeId, TM = T>,
//     {
//         <LazyPostOrder<T, IdD> as super::DecompressedSubtree<T>>::decompress2(store, id)
//     }

//     type Out = LazyPostOrder<T, IdD>;
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn lld(&self, i: &IdD) -> IdD {
//         <LazyPostOrder<T, IdD>>::lld(&self, i)
//     }

//     fn tree(&self, id: &IdD) -> T::TreeId {
//         <LazyPostOrder<T, IdD>>::tree(&self, id)
//     }
// }
// impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrderIterable<T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
// {
//     type It = Iter<IdD>;
//     fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
//         <LazyPostOrder<T, IdD>>::iter_df_post::<ROOT>(&self)
//     }
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> ContiguousDescendants<T, IdD, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
//         <LazyPostOrder<T, IdD>>::descendants_range(&self, x)
//     }

//     // type Slice<'b>
//     //     = SimplePOSlice<'b, T, IdD>
//     // where
//     //     Self: 'b;

//     fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
//         <LazyPostOrder<T, IdD>>::slice(&self, x)
//     }
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
//     for &mut LazyPostOrder<T, IdD>
// {
//     type PIt = IterParents<'a, IdD>;
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn has_parent(&self, id: &IdD) -> bool {
//         <LazyPostOrder<T, IdD>>::has_parent(&self, id)
//     }

//     fn parent(&self, id: &IdD) -> Option<IdD> {
//         <LazyPostOrder<T, IdD>>::parent(&self, id)
//     }

//     fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
//         <LazyPostOrder<T, IdD>>::parents(&self, id)
//     }

//     fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
//         <LazyPostOrder<T, IdD>>::position_in_parent(&self, c)
//     }

//     fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
//         <LazyPostOrder<T, IdD>>::path(&self, parent, descendant)
//     }

//     fn lca(&self, a: &IdD, b: &IdD) -> IdD {
//         <LazyPostOrder<T, IdD>>::lca(&self, a, b)
//     }
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn len(&self) -> usize {
//         <LazyPostOrder<T, IdD>>::len(&self)
//     }

//     fn original(&self, id: &IdD) -> T::TreeId {
//         <LazyPostOrder<T, IdD>>::original(&self, id)
//     }

//     fn root(&self) -> IdD {
//         <LazyPostOrder<T, IdD>>::root(&self)
//     }

//     fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         todo!("deprecated")
//     }

//     fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
// where
//         // 'a: 'b,
//         // S: for<'b> NodeStore<T::TreeId, R<'b> = T>,
//     {
//         todo!("deprecated")
//     }

//     fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
// where
//         // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
//     {
//         <LazyPostOrder<T, IdD>>::child4(&self, store, x, p)
//     }

//     fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
// where
//         // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
//     {
//         <LazyPostOrder<T, IdD>>::children4(&self, store, x)
//     }
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt + Shallow<IdD>> LazyDecompressed<IdD> for &mut LazyPostOrder<T, IdD> {
//     type IdD = IdD;
// }

// impl<T: Stored + WithStats, IdD: PrimInt + Shallow<IdD> + Debug> LazyDecompressedTreeStore<T, IdD>
//     for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn starter(&self) -> Self::IdD {
//         <LazyPostOrder<T, IdD>>::starter(&self)
//     }

//     fn decompress_children<S>(&mut self, store: &S, x: &Self::IdD) -> Vec<Self::IdD>
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD>>::decompress_children(self, store, x)
//     }

//     fn decompress_to<S>(&mut self, store: &S, x: &IdD) -> Self::IdD
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD>>::decompress_to(self, store, x)
//     }
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD>>::descendants(&self, store, x)
//     }

//     fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD>>::descendants_count(&self, store, x)
//     }

//     fn first_descendant(&self, i: &IdD) -> IdD {
//         <LazyPostOrder<T, IdD>>::first_descendant(&self, i)
//     }

//     fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
//         <LazyPostOrder<T, IdD>>::is_descendant(&self, desc, of)
//     }
// }

// impl<'a, T: 'a + Stored, IdD: 'a + PrimInt> LazyPOSliceLending<'a, T, IdD, IdD>
//     for &mut LazyPostOrder<T, IdD>
// where
//     // T: for<'t> types::NLending<'t, T::TreeId>,
//     // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
//     // T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     type SlicePo = CompletePOSlice<'a, T, IdD, bitvec::boxed::BitBox>;
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> LazyPOSliceLending<'a, T, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     // T: for<'t> types::NLending<'t, T::TreeId>,
//     // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
//     // T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     type SlicePo = CompletePOSlice<'a, T, IdD, bitvec::boxed::BitBox>;
// }

// impl<HAST: HyperAST + Copy, IdD: PrimInt> LazyPOBorrowSlice<T, IdD, IdD> for &mut LazyPostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
//     for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
//     IdD: Shallow<IdD> + Debug,
// {
//     fn slice_po<S>(
//         &mut self,
//         store: &S,
//         x: &IdD,
//     ) -> <Self as LazyPOSliceLending<'_, T, IdD>>::SlicePo
//     where
//         S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
//             + NodeStore<T::TreeId>,
//     {
//         <LazyPostOrder<T, IdD>>::slice_po(self, store, x)
//     }
// }

impl<IdN, IdD: PrimInt> LazyPostOrder<IdN, IdD> {
    pub(crate) fn _compute_kr_bitset(&self) -> bitvec::boxed::BitBox {
        // use bitvec::prelude::Lsb0;
        let node_count = self.id_compressed.len();
        let mut kr = bitvec::bitbox!(0;node_count);
        // let mut kr = Vec::with_capacity(node_count);
        let mut visited = bitvec::bitbox!(0; node_count);
        for i in (1..node_count).rev() {
            if !visited[self._lld(i).to_usize().unwrap()] {
                kr.set(i, true);
                // kr.push(cast(i + 1).unwrap());
                visited.set(self._lld(i).to_usize().unwrap(), true);
            }
        }
        // kr.into_boxed_slice()
        kr
    }
}
