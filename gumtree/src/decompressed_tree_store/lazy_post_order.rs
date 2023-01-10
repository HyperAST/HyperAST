use std::{collections::HashMap, fmt::Debug, hash::Hash};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive, Zero};

use super::{
    basic_post_order::{BasicPOSlice, BasicPostOrder},
    complete_post_order::CompletePOSlice,
    simple_post_order::{SimplePOSlice, SimplePostOrder},
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, DecompressedWithSiblings,
    Iter, LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder,
    PostOrderIterable, Shallow, ShallowDecompressedTreeStore,
};
use hyper_ast::{
    position::Position,
    types::{
        self, Children, IterableChildren, LabelStore, NodeStore, Stored, Tree, Type, WithChildren,
        WithSerialization, WithStats,
    },
};
use logging_timer::time;

pub struct LazyPostOrder<T: Stored, IdD> {
    pub(super) id_compressed: Box<[T::TreeId]>,
    pub(super) id_parent: Box<[IdD]>,
    /// leftmost leaf descendant of nodes
    pub(crate) llds: Box<[IdD]>,
    _phantom: std::marker::PhantomData<*const T>,
}

impl<T: Stored, IdD: PrimInt> LazyPostOrder<T, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &T::TreeId> {
        self.id_compressed.iter()
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for LazyPostOrder<T, IdD>
where
    T::TreeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("id_parent", &self.id_parent)
            .field("llds", &self.llds)
            .finish()
    }
}

impl<'d, T: WithChildren, IdD: PrimInt> LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
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

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedWithParent<'d, T, IdD> for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
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

    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
        self.position_in_parent(c)
    }

    type PIt<'a> = IterParents<'a, IdD> where IdD: 'a, T::TreeId:'a, T: 'a;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }
    fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<T::ChildIdx> {
        let ref this = self;
        let mut idxs: Vec<T::ChildIdx> = vec![];
        let mut curr = *descendant;
        while &curr != parent {
            let p = this.parent(&curr).expect("reached root before given parent");
            let idx = this._position_in_parent(&curr, &p);
            idxs.push(idx);
            curr = p;
        }
        idxs.reverse();
        idxs
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        todo!()
    }
}
impl<T: WithChildren, IdD: PrimInt> LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn _position_in_parent(&self, c: &IdD, p: &IdD) -> T::ChildIdx {
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

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedWithSiblings<'d, T, IdD>
    for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        let p = self.parent(x)?;
        let p_lld = self.first_descendant(&p);
        LazyPostOrder::lsib(self, x, &p_lld)
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

impl<'a, T: WithChildren, IdD: PrimInt> PostOrder<'a, T, IdD> for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> LazyPostOrder<T, IdD> {
    pub(crate) fn _size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }

    fn _lld(&self, i: usize) -> IdD {
        self.llds[i]
    }
    fn _len(&self) -> usize {
        self.id_parent.len()
    }
    fn _root(&self) -> usize {
        self._len() - 1
    }
}

impl<'a, T: WithChildren + WithStats, IdD: PrimInt + Shallow<IdD> + Debug> LazyPostOrder<T, IdD>
where
    <T as Stored>::TreeId: Clone,
    <T as Stored>::TreeId: Debug,
{
    fn continuous_aux<'b, S>(
        &mut self,
        store: &'b S,
        x: &<Self as LazyDecompressedTreeStore<T, IdD>>::IdD,
    ) -> Option<Vec<T::TreeId>>
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
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
        Some(
            cs.before(cs.child_count() - one())
                .iter_children()
                .cloned()
                .collect(),
        )
    }

    fn decompress_descendants_continuous<'b, S>(
        &mut self,
        store: &'b S,
        x: &<Self as LazyDecompressedTreeStore<T, IdD>>::IdD,
    ) where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
        // PAVE CESAR oO
        let mut c = *x;
        let mut s: Vec<(IdD, Vec<T::TreeId>)> = vec![];
        loop {
            // Termination: when s is empty, done by second loop
            loop {
                // Termination: go down the last child, finish when no more remains
                let rem = self.continuous_aux(store, &c);
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
                let Some((p,mut rem)) = s.pop() else {
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
            let Some((p,z)) = next else {
                assert!(self._size(x) <= one() || self.tree(&(c + one())) != self.tree(x), "{:?} {:?}", self.tree(&(c + one())), self.tree(x));
                assert!(c == self.lld(x) || c + one() == self.lld(x));
                break;
            };
            self.id_parent[c.to_usize().unwrap()] = p;
            self.id_compressed[c.to_usize().unwrap()] = z;
        }
    }

    pub fn decompress_descendants<'b, S>(&mut self, store: &'b S, x: &IdD)
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let mut q = vec![x.clone()];
        while let Some(x) = q.pop() {
            assert!(self.id_parent[x.to_usize().unwrap()] != zero());
            q.extend(self.decompress_children(store, &x));
        }
    }
    // WARN just used for debugging
    // TODO remove
    pub fn go_through_descendants<'b, S>(&mut self, store: &'b S, x: &IdD)
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let mut q = vec![x.clone()];
        while let Some(x) = q.pop() {
            assert!(self.id_parent[x.to_usize().unwrap()] != zero());
            assert_eq!(
                self._size(&x).to_usize().unwrap(),
                store.resolve(&self.original(&x)).size()
            );
            q.extend(self.children(store, &x));
        }
    }
    #[time("warn")]
    pub fn complete<S>(mut self, store: &'a S) -> SimplePostOrder<T, IdD>
    where
        T::TreeId: Eq,
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
        self.complete_subtree(store, &self.root());
        SimplePostOrder {
            basic: BasicPostOrder {
                id_compressed: self.id_compressed,
                llds: self.llds,
                _phantom: std::marker::PhantomData,
            },
            id_parent: self.id_parent,
        }
    }
    pub fn complete_subtree<S>(&mut self, store: &'a S, x: &IdD)
    where
        T::TreeId: Eq,
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
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
                self.decompress_descendants_continuous(store, &i);
                i = self.lld(&i);
            }
            if i == first {
                break;
            }
        }
        self.decompress_children(store, x).len();
    }
}

impl<'a, T, IdD: PrimInt + Debug> super::DecompressedSubtree<'a, T> for LazyPostOrder<T, IdD>
where
    T: WithChildren + WithStats,
    T::TreeId: Clone + Debug,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn decompress<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
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

pub(super) fn init_boxed_slice<T: Clone>(value: T, pred_len: usize) -> Box<[T]> {
    let mut v = Vec::with_capacity(pred_len);
    v.resize(pred_len, value);
    v.into_boxed_slice()
}

impl<'a, T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<'a, T, IdD>
    for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn len(&self) -> usize {
        self._len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn root(&self) -> IdD {
        cast(self._root()).unwrap()
    }

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let node = store.resolve(&a);
            let cs = node.children().filter(|x| x.is_empty());
            let Some(cs) = cs  else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(*d + one());
            let cs: Vec<T::TreeId> = cs.iter_children().cloned().collect();
            for x in cs {
                z += Self::size2(store, &x);
            }
            r = self.first_descendant(&r) + cast(z).unwrap() - one();
        }
        r
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        debug_assert!(
            self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero(),
            "x has not been initialized"
        );
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

impl<'d, T: WithChildren + WithStats, IdD: PrimInt + Shallow<IdD> + Debug>
    LazyDecompressedTreeStore<'d, T, IdD> for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    type IdD = IdD;
    fn starter(&self) -> Self::IdD {
        ShallowDecompressedTreeStore::root(self)
    }

    fn decompress_children<'b, S>(&mut self, store: &'b S, x: &Self::IdD) -> Vec<Self::IdD>
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
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

    fn decompress_to<'b, S>(&mut self, store: &'b S, x: &IdD) -> Self::IdD
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
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
            let cs = self.decompress_children(store, &p);
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

impl<'a, T: WithChildren, IdD: PrimInt + Shallow<IdD> + Debug> LazyPostOrder<T, IdD>
where
    <T as Stored>::TreeId: Clone,
    <T as Stored>::TreeId: Debug,
{
    fn is_decompressed(&self, x: &IdD) -> bool {
        self.id_parent.len() == 0 || self.id_parent[x.to_usize().unwrap()] != zero()
    }
}

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD> for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants<'b, S>(&self, _store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()] // TODO use ldd
    }

    fn descendants_count<'b, S>(&self, _store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        (self._size(x)).to_usize().unwrap() - 1
    }

    fn is_descendant(&self, desc: &IdD,of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> ContiguousDescendants<'d, T, IdD, IdD>
    for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    type Slice<'b> = SimplePOSlice<'b,T,IdD> where Self: 'b;

    /// WIP
    fn slice(&self, _x: &IdD) -> Self::Slice<'_> {
        // Would need to complete the subtree under x, need the node store to do so
        // TODO could also make a lazy slice !!
        todo!()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    pub(super) fn slice_range(&self, x: &IdD) -> std::ops::RangeInclusive<usize> {
        self.first_descendant(x).to_usize().unwrap()..=x.to_usize().unwrap()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> LazyPOBorrowSlice<'d, T, IdD, IdD>
    for LazyPostOrder<T, IdD>
where
    T: WithStats,
    T::TreeId: Clone + Eq + Debug,
    IdD: Shallow<IdD> + Debug,
{
    type SlicePo<'b> = CompletePOSlice<'b,T,IdD, bitvec::boxed::BitBox>
    where
        Self: 'b;

    fn slice_po<'b, S>(&mut self, store: &'b S, x: &IdD) -> Self::SlicePo<'_>
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
        self.complete_subtree(store, x);
        let range = self.slice_range(x);
        let basic = BasicPOSlice {
            id_compressed: &self.id_compressed[range.clone()],
            llds: &self.llds[range.clone()],
            _phantom: std::marker::PhantomData,
        };
        let kr = basic.compute_kr_bitset();
        let simple = SimplePOSlice {
            basic,
            id_parent: &self.id_parent[range],
        };
        CompletePOSlice { simple, kr }
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderIterable<'d, T, IdD> for LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Debug,
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

pub struct RecCachedPositionProcessor<'a, T: WithChildren, IdD: Hash + Eq> {
    pub(crate) ds: &'a LazyPostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, Position>,
}

impl<'a, T: WithChildren, IdD: PrimInt + Eq> LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Debug,
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

impl<'a, T: WithChildren, IdD: PrimInt> LazyPostOrder<T, IdD>
where
    T::TreeId: Clone + Debug,
{
    fn size2<'b, S>(store: &'b S, x: &T::TreeId) -> usize
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let tmp = store.resolve(x);
        let Some(cs) = tmp.children() else {
                return 1;
            };

        let mut z = 0;
        for x in cs.iter_children() {
            z += Self::size2(store, x);
        }
        z + 1
    }
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq> From<(&'a LazyPostOrder<T, IdD>, T::TreeId)>
    for RecCachedPositionProcessor<'a, T, IdD>
{
    fn from((ds, root): (&'a LazyPostOrder<T, IdD>, T::TreeId)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

impl<'a, T: Tree, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD> {
    pub fn position<'b, S, LS>(&mut self, store: &'b S, lstore: &'b LS, c: &IdD) -> &Position
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
        T::TreeId: Clone + Debug,
        LS: LabelStore<str>,
        T: Tree<Type = Type, Label = LS::I> + WithSerialization,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let p_r = store.resolve(&self.ds.original(&p));
            let p_t = p_r.get_type();
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = store.resolve(&ori);
                    return self.cache.entry(*c).or_insert(Position::new(
                        lstore.resolve(&r.get_label()).into(),
                        0,
                        r.try_bytes_len().unwrap_or(0),
                    ));
                }
                let mut pos = self
                    .cache
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| self.position(store, lstore, &p).clone());
                let r = store.resolve(&ori);
                pos.inc_path(lstore.resolve(&r.get_label()));
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
                    .unwrap_or_else(|| self.position(store, lstore, &lsib).clone());
                pos.inc_offset(pos.range().end - pos.range().start);
                let r = store.resolve(&self.ds.original(&c));
                pos.set_len(r.try_bytes_len().unwrap());
                self.cache.entry(*c).or_insert(pos)
            } else {
                assert!(
                    self.ds.position_in_parent(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = store.resolve(&ori);
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
                    .unwrap_or_else(|| self.position(store, lstore, &p).clone());
                let r = store.resolve(&ori);
                pos.set_len(
                    r.try_bytes_len()
                        .unwrap_or_else(|| panic!("{:?}", r.get_type())),
                );
                self.cache.entry(*c).or_insert(pos)
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            let r = store.resolve(&ori);
            let t = r.get_type();
            let pos = if t.is_directory() || t.is_file() {
                let file = lstore.resolve(&r.get_label()).into();
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
    pub(crate) ds: &'a LazyPostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq, U, F, G>
    From<(&'a LazyPostOrder<T, IdD>, T::TreeId, F, G)> for RecCachedProcessor<'a, T, IdD, U, F, G>
{
    fn from((ds, root, with_p, with_lsib): (&'a LazyPostOrder<T, IdD>, T::TreeId, F, G)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
            with_p,
            with_lsib,
        }
    }
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, T, IdD, U, F, G>
where
    F: Fn(U, T::TreeId) -> U,
    G: Fn(U, T::TreeId) -> U,
{
    pub fn position<'b, S>(&mut self, store: &'b S, c: &IdD) -> &U
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
        T::TreeId: Clone + Debug,
        T: Tree<Type = Type> + WithSerialization,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let p_r = store.resolve(&self.ds.original(&p));
            let p_t = p_r.get_type();
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                }
                let pos = self.position(store, &p).clone();
                return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
            }

            let p_lld = self.ds.first_descendant(&p);
            if let Some(lsib) = self.ds.lsib(c, &p_lld) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position(store, &lsib).clone();
                self.cache
                    .entry(*c)
                    .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
            } else {
                assert!(
                    self.ds.position_in_parent(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                }
                let pos = self.position(store, &p).clone();
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
