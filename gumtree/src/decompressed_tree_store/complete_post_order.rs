use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive, Zero};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::{
    position::Position,
    types::{
        self, Children, IterableChildren, LabelStore, NodeStore, Stored, Tree, Type, WithChildren,
        WithSerialization,
    },
};

use super::{
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, Initializable, Iter,
    PostOrder, PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<T: Stored, IdD> {
    leaf_count: IdD,
    id_compressed: Vec<T::TreeId>,
    id_parent: Vec<IdD>,
    /// leftmost leaf descendant of nodes
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)= l(k’)}.
    kr: Vec<IdD>,
    _phantom: std::marker::PhantomData<*const T>,
}

impl<T: Stored, IdD: PrimInt + Into<usize>> CompletePostOrder<T, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &T::TreeId> {
        self.id_compressed.iter()
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for CompletePostOrder<T, IdD>
where
    T::TreeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletePostOrder")
            .field("leaf_count", &self.leaf_count)
            .field("id_compressed", &self.id_compressed)
            .field("id_parent", &self.id_parent)
            .field("llds", &self.llds)
            .field("kr", &self.kr)
            .finish()
    }
}

pub struct DisplayCompletePostOrder<'store: 'a, 'a, T: Stored, IdD: PrimInt, S, LS>
where
    S: NodeStore<T::TreeId, R<'store> = T>,
    LS: LabelStore<str>,
{
    pub inner: &'a CompletePostOrder<T, IdD>,
    pub node_store: &'store S,
    pub label_store: &'store LS,
}

impl<'store: 'a, 'a, T: Tree + WithSerialization, IdD: PrimInt, S, LS> Display
    for DisplayCompletePostOrder<'store, 'a, T, IdD, S, LS>
where
    S: NodeStore<T::TreeId, R<'store> = T>,
    T::TreeId: Clone + Debug,
    T::Type: Debug,
    LS: LabelStore<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = SimplePreOrderMapper::from(self.inner);
        DisplaySimplePreOrderMapper {
            inner: &m,
            node_store: self.node_store,
        }
        .fmt(f)
        .unwrap();
        Ok(())
    }
}

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedWithParent<'d, T, IdD>
    for CompletePostOrder<T, IdD>
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

    fn position_in_parent<'b, S>(
        &self,
        _store: &'b S,
        c: &IdD,
    ) -> <S::R<'b> as WithChildren>::ChildIdx
    where
        S: NodeStore<T::TreeId>,
        S::R<'b>: WithChildren<TreeId = T::TreeId>,
        <S::R<'b> as WithChildren>::ChildIdx: PrimInt,
    {
        let p = self.parent(c).unwrap();
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
        cast(r).unwrap()
    }

    type PIt<'a> = IterParents<'a, IdD> where IdD: 'a, T::TreeId:'a, T: 'a;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        IterParents {
            id,
            id_parent: &self.id_parent,
        }
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

impl<'a, T: 'a + WithChildren, IdD: PrimInt> PostOrder<'a, T, IdD> for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'d, T: WithChildren, IdD: PrimInt> CompletePostOrder<T, IdD> {
    fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrderIterable<'d, T, IdD>
    for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    type It = Iter<IdD>;
    fn iter_df_post(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderKeyRoots<'d, T, IdD>
    for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.to_usize().unwrap()]
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> Initializable<'a, T> for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn new<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
        struct R<IdC, Idx, IdD> {
            curr: IdC,
            idx: Idx,
            lld: IdD,
            children: Vec<IdD>,
        }

        let mut leaf_count = 0;
        let mut stack = vec![R {
            curr: root.clone(),
            idx: zero(),
            lld: IdD::zero(),
            children: vec![],
        }];
        let mut llds: Vec<IdD> = vec![];
        let mut id_compressed = vec![];
        let mut id_parent = vec![];
        while let Some(R {
            curr,
            idx,
            lld,
            children,
        }) = stack.pop()
        {
            let x = store.resolve(&curr);
            let l = x.children().filter(|x| !x.is_empty());
            if let Some(child) = l.and_then(|l| l.get(idx)) {
                stack.push(R {
                    curr,
                    idx: idx + one(),
                    lld,
                    children,
                });
                stack.push(R {
                    curr: child.clone(),
                    idx: zero(),
                    lld: zero(),
                    children: vec![],
                });
            } else {
                let curr_idx = cast(id_compressed.len()).unwrap();
                let value = if l.is_none() {
                    leaf_count += 1;
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

        let node_count = id_compressed.len();
        let mut kr = vec![num_traits::zero(); leaf_count + 1];
        let mut visited = vec![false; node_count];
        let mut k = kr.len() - 1;
        for i in (1..node_count).rev() {
            if !visited[llds[i].to_usize().unwrap()] {
                kr[k] = cast(i + 1).unwrap();
                visited[llds[i].to_usize().unwrap()] = true;
                if k > 0 {
                    k -= 1;
                }
            }
        }
        let leaf_count = cast(leaf_count).unwrap();
        id_compressed.shrink_to_fit();
        llds.shrink_to_fit();
        kr.shrink_to_fit();
        Self {
            leaf_count,
            id_compressed,
            llds,
            kr,
            id_parent,
            _phantom: Default::default(),
        }
    }
}
impl<'a, T: 'a + WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<'a, T, IdD>
    for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
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

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<Idx> {
        let mut idxs: Vec<Idx> = vec![];
        let mut curr = *descendant;
        loop {
            if let Some(p) = self.parent(&curr) {
                let lld: usize = cast(self.llds[p.to_usize().unwrap()]).unwrap();
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

impl<'d, T: 'd + WithChildren, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD>
    for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants<'b, S>(&self, _store: &S, x: &IdD) -> Vec<IdD>
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

    fn descendants_count<'b, S>(&self, _store: &S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> ContiguousDescendants<'d, T, IdD>
    for CompletePostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }
}

pub struct RecCachedPositionProcessor<'a, T: WithChildren, IdD: Hash + Eq> {
    pub(crate) ds: &'a CompletePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, Position>,
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq> CompletePostOrder<T, IdD>
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

impl<'a, T: WithChildren, IdD: PrimInt> CompletePostOrder<T, IdD>
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

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq> From<(&'a CompletePostOrder<T, IdD>, T::TreeId)>
    for RecCachedPositionProcessor<'a, T, IdD>
{
    fn from((ds, root): (&'a CompletePostOrder<T, IdD>, T::TreeId)) -> Self {
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
        // S: NodeStore<IdC>,
        LS: LabelStore<str>,
        T: Tree<Type = Type, Label = LS::I> + WithSerialization,
        // <S::R<'b> as WithChildren>::ChildIdx: PrimInt,
        // for<'c> <S::R<'b> as WithChildren>::Children<'c>:
        //     types::Children<<S::R<'b> as WithChildren>::ChildIdx, IdC>,
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
                    self.ds.position_in_parent(store, c).is_zero(),
                    "{:?}",
                    self.ds.position_in_parent(store, c).to_usize()
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
    pub(crate) ds: &'a CompletePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, T: WithChildren, IdD: PrimInt + Hash + Eq, U, F, G>
    From<(&'a CompletePostOrder<T, IdD>, T::TreeId, F, G)>
    for RecCachedProcessor<'a, T, IdD, U, F, G>
{
    fn from(
        (ds, root, with_p, with_lsib): (&'a CompletePostOrder<T, IdD>, T::TreeId, F, G),
    ) -> Self {
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
                    // let r = store.resolve(&ori);
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                    // Position::new(
                    //     lstore.resolve(&r.get_label()).into(),
                    //     0,
                    //     r.try_bytes_len().unwrap_or(0),
                    // )
                }
                let pos = self.position(store, &p).clone();
                // let r = store.resolve(&ori);
                // pos.inc_path(lstore.resolve(&r.get_label()));
                // pos.set_len(r.try_bytes_len().unwrap_or(0));
                // return self.cache.entry(*c).or_insert(pos);
                return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
            }

            let p_lld = self.ds.first_descendant(&p);
            if let Some(lsib) = self.ds.lsib(c, &p_lld) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position(store, &lsib).clone();
                // pos.inc_offset(pos.range().end - pos.range().start);
                // let r = store.resolve(&self.ds.original(&c));
                // pos.set_len(r.try_bytes_len().unwrap());
                // self.cache.entry(*c).or_insert(pos)
                self.cache
                    .entry(*c)
                    .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
            } else {
                assert!(
                    self.ds.position_in_parent(store, c).is_zero(),
                    "{:?}",
                    self.ds.position_in_parent(store, c).to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    // let r = store.resolve(&ori);
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                    // Position::new(
                    //     "".into(),
                    //     0,
                    //     r.try_bytes_len().unwrap(),
                    // )
                }
                let pos = self.position(store, &p).clone();
                // let r = store.resolve(&ori);
                // pos.set_len(
                //     r.try_bytes_len()
                //         .unwrap_or_else(|| panic!("{:?}", r.get_type())),
                // );
                // self.cache.entry(*c).or_insert(pos)
                self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            // let r = store.resolve(&ori);
            // let t = r.get_type();
            // let pos = if t.is_directory() || t.is_file() {
            //     let file = lstore.resolve(&r.get_label()).into();
            //     let offset = 0;
            //     let len = r.try_bytes_len().unwrap_or(0);
            //     Position::new(file, offset, len)
            // } else {
            //     let file = "".into();
            //     let offset = 0;
            //     let len = r.try_bytes_len().unwrap_or(0);
            //     Position::new(file, offset, len)
            // };
            // self.cache.entry(*c).or_insert(pos)
            self.cache
                .entry(*c)
                .or_insert((self.with_p)(Default::default(), ori))
        }
    }
}
