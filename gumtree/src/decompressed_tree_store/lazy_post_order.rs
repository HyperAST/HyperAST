use std::{collections::HashMap, fmt::Debug, hash::Hash};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive, Zero};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::{
    position::Position,
    types::{
        self, Children, IterableChildren, LabelStore, NodeStore, Stored, Tree, Type, WithChildren,
        WithSerialization, WithStats,
    },
};

use super::{
    basic_post_order::BasicPostOrder,
    simple_post_order::{SimplePOSlice, SimplePostOrder},
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, DecompressedWithSiblings,
    Initializable, LazyDecompressedTreeStore, PostOrder, Shallow, ShallowDecompressedTreeStore,
};

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

    fn path(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<T::ChildIdx> {
        let mut idxs: Vec<T::ChildIdx> = vec![];
        let mut curr = *descendant;
        loop {
            if let Some(p) = self.parent(&curr) {
                let lld: usize = cast(self.llds[p.to_usize().unwrap()]).unwrap();
                let lld = lld - 1;
                // TODO use other llds to skip nodes for count
                let idx = self.id_parent[lld..cast(curr).unwrap()]
                    .iter()
                    .filter(|x| **x == p)
                    .count();
                let idx = cast(idx).unwrap();
                idxs.push(idx);
                if &p == parent {
                    break;
                }
                curr = p;
            } else {
                break;
            }
        }
        idxs.reverse();
        idxs.into()
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
        // *i + one() - self.llds[(*i).to_usize().unwrap()] = size
        //  self.llds[(*i).to_usize().unwrap()]  = size - i - 1
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
    // <Self as LazyDecompressedTreeStore<'a, T, IdD>>::IdD: PrimInt,
    <T as Stored>::TreeId: Clone,
    <T as Stored>::TreeId: Debug,
{
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
            q.extend(self.children(store, &x));
        }
    }
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
        let first = self.first_descendant(x);
        let mut i = x.clone();
        while i > first {
            if self.id_parent[i.to_usize().unwrap() - 1] != zero() {
                i = i - one();
            } else {
                self.decompress_descendants(store, &i);
                i = self.lld(&i);
            }
            if i == first {
                break;
            }
        }
    }
}

impl<'a, T, IdD: PrimInt + Debug> Initializable<'a, T> for LazyPostOrder<T, IdD>
where
    T: WithChildren + WithStats,
    T::TreeId: Clone + Debug,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn new<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
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
        // TODO do some refactoring, this is ugly
        let a = self.original(x);
        let node = store.resolve(&a);
        let Some(cs) = node.children() else {
            return vec![];
        };
        let cs_len = cs.child_count().to_usize().unwrap();
        if cs_len == 0 {
            return vec![];
        }
        let x_size = node.size() - 1;
        assert_eq!(
            x.to_usize().unwrap() - x_size,
            self._lld(x.to_usize().unwrap()).to_usize().unwrap()
        );
        let mut r = vec![zero(); cs_len];
        let mut c = *x - one();
        let mut i = cs_len - 1;
        let mut child_size = 0;
        r[i] = c;
        self.id_compressed[c.to_usize().unwrap()] = cs[cast(i).unwrap()].clone();
        self.id_parent[c.to_usize().unwrap()] = x.clone();
        let mut s = store.resolve(&cs[cast(i).unwrap()]).size();
        child_size += store.resolve(&cs[cast(i).unwrap()]).size();
        self.llds[c.to_usize().unwrap()] = *x - cast::<_, IdD>(child_size).unwrap();
        assert_eq!(self._size(&c).to_usize().unwrap(), s);
        while i > 0 {
            c = c - cast(s).unwrap();
            i -= 1;
            r[i] = c;
            self.id_compressed[c.to_usize().unwrap()] = cs[cast(i).unwrap()].clone();
            self.id_parent[c.to_usize().unwrap()] = x.clone();
            s = store.resolve(&cs[cast(i).unwrap()]).size();
            child_size += store.resolve(&cs[cast(i).unwrap()]).size();
            self.llds[c.to_usize().unwrap()] = *x - cast::<_, IdD>(child_size).unwrap();
            assert_eq!(self._size(&c).to_usize().unwrap(), s);
        }
        assert_eq!(x_size, child_size);
        assert_eq!(
            self._lld(x.to_usize().unwrap()).to_usize().unwrap(),
            self._lld(c.to_usize().unwrap()).to_usize().unwrap()
        );
        r
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
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> ContiguousDescendants<'d, T, IdD>
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
