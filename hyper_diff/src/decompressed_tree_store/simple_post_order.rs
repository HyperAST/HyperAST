use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive, Zero};

use hyper_ast::{
    position::Position,
    types::{
        self, Children, IterableChildren, LabelStore, NodeStore, Stored, Tree, Type, WithChildren,
        WithSerialization,
    },
};

use super::{
    basic_post_order::{BasicPOSlice, BasicPostOrder},
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, DecompressedWithSiblings,
    PostOrder, ShallowDecompressedTreeStore,
};

pub struct SimplePostOrder<T: Stored, IdD> {
    pub(super) basic: BasicPostOrder<T, IdD>,
    pub(super) id_parent: Box<[IdD]>,
}

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

impl<'d, T: WithChildren, IdD: PrimInt> SimplePostOrder<T, IdD>
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

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedWithParent<'d, T, IdD>
    for SimplePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
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

    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
        let p = self.parent(c)?;
        Some(self._position_in_parent(c, &p))
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

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedWithSiblings<'d, T, IdD>
    for SimplePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
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

impl<'a, T: WithChildren, IdD: PrimInt> PostOrder<'a, T, IdD> for SimplePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.basic.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.basic.tree(id)
    }
}

// impl<'d, T: WithChildren, IdD: PrimInt> SimplePostOrder<T, IdD> {
//     pub(crate) fn size(&self, i: &IdD) -> IdD {
//         *i - self.llds[(*i).to_usize().unwrap()] + one()
//     }
// }

// impl<'d, T: WithChildren, IdD: PrimInt> PostOrderIterable<'d, T, IdD> for SimplePostOrder<T, IdD>
// where
//     T::TreeId: Clone + Eq + Debug,
// {
//     type It = Iter<IdD>;
//     fn iter_df_post(&self) -> Iter<IdD> {
//         Iter {
//             current: zero(),
//             len: (cast(self.id_compressed.len())).unwrap(),
//         }
//     }
// }

impl<'a, T, IdD: PrimInt> super::DecompressedSubtree<'a, T> for SimplePostOrder<T, IdD>
where
    T: WithChildren,
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn decompress<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
        let simple = SimplePostOrder::make(store, root);
        Self {
            basic: simple.basic,
            id_parent: simple.id_parent,
        }
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> SimplePostOrder<T, IdD>
where
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn make<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
        let mut stack = vec![Element {
            curr: root.clone(),
            idx: zero(),
            lld: IdD::zero(),
            children: vec![],
        }];
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
            let x = store.resolve(&curr);
            let l = x.children().filter(|x| !x.is_empty());
            if let Some(child) = l.and_then(|l| l.get(idx)) {
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
                let value = if l.is_none() {
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

impl<'a, T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<'a, T, IdD>
    for SimplePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
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

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let node = store.resolve(&a);
            let cs = node.children().filter(|x| !x.is_empty());
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
}

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD>
    for SimplePostOrder<T, IdD>
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
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.basic.is_descendant(desc, of)
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> ContiguousDescendants<'d, T, IdD>
    for SimplePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    type Slice<'b> = SimplePOSlice<'b,T,IdD> where Self: 'b;

    fn slice(&self, x: &IdD) -> Self::Slice<'_> {
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

impl<'a, T: WithChildren, IdD: PrimInt + Eq> SimplePostOrder<T, IdD>
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

impl<'a, T: WithChildren, IdD: PrimInt> SimplePostOrder<T, IdD>
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

impl<T: Stored, IdD: PrimInt + Debug> Debug for SimplePostOrder<T, IdD>
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

pub struct RecCachedPositionProcessor<'a, T: WithChildren, IdD: Hash + Eq> {
    pub(crate) ds: &'a SimplePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, Position>,
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq> From<(&'a SimplePostOrder<T, IdD>, T::TreeId)>
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
    pub(crate) ds: &'a SimplePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq, U, F, G>
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
