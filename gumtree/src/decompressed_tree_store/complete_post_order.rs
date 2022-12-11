use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive, Zero};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::{
    position::Position,
    types::{LabelStore, Labeled, NodeStore, Tree, Type, Typed, WithChildren, WithSerialization},
};

use super::{
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    size, ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, Initializable,
    Iter, PostOrder, PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<IdC, IdD> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)= l(k’)}.
    kr: Vec<IdD>,
}

// <T:WithChildren + Labeled>
// where T::Label : PrimInt
impl<IdC, IdD: PrimInt + Into<usize>> CompletePostOrder<IdC, IdD> {
    // pub fn fmt<G: Fn(&IdC) -> String>(
    //     &self,
    //     f: &mut std::fmt::Formatter<'_>,
    //     g: G,
    // ) -> std::fmt::Result {
    //     self.id_compressed
    //         .iter()
    //         .enumerate()
    //         .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
    //     write!(f, "")
    // }
    pub fn iter(&self) -> impl Iterator<Item = &IdC> {
        self.id_compressed.iter()
    }
}
impl<IdC: Debug, IdD: PrimInt + Debug> Debug for CompletePostOrder<IdC, IdD> {
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
pub struct DisplayCompletePostOrder<'a, IdC, IdD: PrimInt, S, LS>
where
    S: NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>,
    LS: LabelStore<str>,
{
    pub inner: &'a CompletePostOrder<IdC, IdD>,
    pub node_store: &'a S,
    pub label_store: &'a LS,
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt, S, LS> Display
    for DisplayCompletePostOrder<'a, IdC, IdD, S, LS>
where
    S: NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC> + Typed + WithSerialization,
    <S::R<'a> as Typed>::Type: Debug,
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
        // for i in 0..m.map.len() {
        //     let o = m.map[i];
        //     let node = self.node_store.resolve(&self.inner.original(&o));
        //     writeln!(
        //         f,
        //         "{:>3}:{} {:?}",
        //         o.to_usize().unwrap(),
        //         "  ".repeat(m.depth[i].to_usize().unwrap()),
        //         node.get_type()
        //     )?;
        // }
        // Ok(())
    }
}

impl<'d, IdC: Clone + Eq + Debug, IdD: PrimInt> DecompressedWithParent<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
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
        S: NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
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

    type PIt<'a> = IterParents<'a, IdD> where IdD: 'a, IdC:'a;

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

impl<'d, IdC: Clone + Debug + Eq, IdD: PrimInt> PostOrder<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'d, IdC, IdD: PrimInt> CompletePostOrder<IdC, IdD> {
    fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
}

impl<'d, IdC: Clone + Debug + Eq, IdD: PrimInt> PostOrderIterable<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    type It = Iter<IdD>;
    fn iter_df_post(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'d, IdC: Clone + Debug + Eq, IdD: PrimInt> PostOrderKeyRoots<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.to_usize().unwrap()]
    }
}
impl<'d, IdC: Clone, IdD: PrimInt> Initializable<'d, IdC, IdD> for CompletePostOrder<IdC, IdD> {
    fn new<
        // 'a,
        // T: 'a + Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S, //: 'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
    >(
        store: &'d S,
        root: &IdC,
    ) -> Self
    where
        S: 'd + NodeStore<IdC>,
        // for<'c> < <S as NodeStore2<IdC>>::R  as GenericItem<'c>>::Item:WithChildren<TreeId = IdC>,
        S::R<'d>: WithChildren<TreeId = IdC>,
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
            lld: zero(),
            children: vec![],
        }];
        let mut llds: Vec<IdD> = vec![];
        let mut id_compressed = vec![];
        let mut id_parent = vec![];
        loop {
            if let Some(R {
                curr,
                idx,
                lld,
                children,
            }) = stack.pop()
            {
                let idx: <S::R<'d> as WithChildren>::ChildIdx = idx;
                let x = store.resolve(&curr);

                let l = x.try_get_children();
                if l.is_none() || l.unwrap().len() == 0 {
                    // leaf
                    let curr_idx = cast(id_compressed.len())
                        .unwrap_or_else(|| panic!("{}", id_compressed.len()));
                    if let Some(tmp) = stack.last_mut() {
                        if tmp.idx == one() {
                            tmp.lld = curr_idx;
                        }
                        tmp.children.push(curr_idx);
                    }
                    llds.push(curr_idx);
                    id_compressed.push(curr);
                    id_parent.push(zero());
                    leaf_count += 1;
                } else if idx.to_usize().unwrap() < l.unwrap().len() {
                    //
                    let child = x.get_child(&idx);
                    stack.push(R {
                        curr,
                        idx: idx + one(),
                        lld,
                        children,
                    });
                    stack.push(R {
                        curr: child,
                        idx: zero(),
                        lld: zero(),
                        children: vec![],
                    });
                } else {
                    let curr_idx = cast(id_compressed.len()).unwrap();
                    if let Some(tmp) = stack.last_mut() {
                        if tmp.idx == one() {
                            tmp.lld = lld;
                        }
                        tmp.children.push(curr_idx);
                    }
                    for x in children {
                        id_parent[x.to_usize().unwrap()] = curr_idx;
                    }
                    id_compressed.push(curr);
                    id_parent.push(zero());
                    llds.push(lld);
                }
            } else {
                break;
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
        }
    }
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt> ShallowDecompressedTreeStore<'a, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[<S::R<'b> as WithChildren>::ChildIdx]) -> IdD
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store
                .resolve(&a)
                .try_get_children()
                .map_or(vec![], |x| x.to_owned());
            if cs.len() > 0 {
                let mut z = 0;
                for x in cs[0..d.to_usize().unwrap() + 1].to_owned() {
                    z += size(store, &x);
                }
                r = self.first_descendant(&r) + cast(z).unwrap() - one();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let a = self.original(x);
        let cs_len = store.resolve(&a).child_count().to_usize().unwrap();
        if cs_len == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs_len];
        let mut c = *x - one(); // = self.first_descendant(x);
        let mut i = cs_len - 1;
        // let mut it = (0..cs_len).rev();
        r[i] = c;
        while i > 0 {
            // let y = it.next().unwrap();
            // println!(
            //     "i={:?} c={:?} size={:?} r={:?}", i, c.to_usize().unwrap(), size(store, &cs[y]),
            //     r.iter().map(|x| x.to_usize().unwrap()).collect::<Vec<_>>()
            // );
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

impl<'d, IdC: Clone + Eq + Debug, IdD: PrimInt> DecompressedTreeStore<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn descendants<'b, S>(&self, _store: &S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
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
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }
}

impl<'d, IdC: Clone + Eq + Debug, IdD: PrimInt> ContiguousDescendants<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }
}

pub struct RecCachedPositionProcessor<'a, IdC, IdD: Hash + Eq> {
    pub(crate) ds: &'a CompletePostOrder<IdC, IdD>,
    root: IdC,
    cache: HashMap<IdD, Position>,
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt + Hash + Eq> CompletePostOrder<IdC, IdD> {
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

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt + Hash + Eq>
    From<(&'a CompletePostOrder<IdC, IdD>, IdC)> for RecCachedPositionProcessor<'a, IdC, IdD>
{
    fn from((ds, root): (&'a CompletePostOrder<IdC, IdD>, IdC)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt + Hash + Eq>
    RecCachedPositionProcessor<'a, IdC, IdD>
{
    pub fn position<'b, S, LS>(&mut self, store: &'b S, lstore: &'b LS, c: &IdD) -> &Position
    where
        S: NodeStore<IdC>,
        LS: LabelStore<str>,
        S::R<'b>: Tree<Type = Type, TreeId = IdC, Label = LS::I> + WithSerialization,
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
pub struct RecCachedProcessor<'a, IdC, IdD: Hash + Eq, T, F, G> {
    pub(crate) ds: &'a CompletePostOrder<IdC, IdD>,
    root: IdC,
    cache: HashMap<IdD, T>,
    with_p: F,
    with_lsib: G,
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt + Hash + Eq, T, F, G>
    From<(&'a CompletePostOrder<IdC, IdD>, IdC, F, G)>
    for RecCachedProcessor<'a, IdC, IdD, T, F, G>
{
    fn from((ds, root, with_p, with_lsib): (&'a CompletePostOrder<IdC, IdD>, IdC, F, G)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
            with_p,
            with_lsib,
        }
    }
}

impl<'a, IdC: Clone + Eq + Debug, IdD: PrimInt + Hash + Eq, T: Clone + Default, F, G>
    RecCachedProcessor<'a, IdC, IdD, T, F, G>
where
    F: Fn(T, IdC) -> T,
    G: Fn(T, IdC) -> T,
{
    pub fn position<'b, S>(&mut self, store: &'b S, c: &IdD) -> &T
    where
        S: NodeStore<IdC>,
        S::R<'b>: Tree<Type = Type, TreeId = IdC> + WithSerialization,
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
