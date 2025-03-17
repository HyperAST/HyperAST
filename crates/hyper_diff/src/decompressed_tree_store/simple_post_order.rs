use super::{
    basic_post_order::{BasicPOSlice, BasicPostOrder},
    ContiguousDescendants, DecendantsLending, DecompressedParentsLending, DecompressedTreeStore,
    DecompressedWithParent, DecompressedWithSiblings, FullyDecompressedTreeStore, PostOrder,
    ShallowDecompressedTreeStore,
};
use crate::matchers::Decompressible;
use hyperast::PrimInt;
use hyperast::{
    position::Position,
    types::{
        self, Children, Childrn, HyperAST, HyperASTShared, HyperType, LabelStore, Labeled,
        NodeStore, Stored, WithChildren, WithSerialization,
    },
};
use num_traits::{cast, one, zero, ToPrimitive, Zero};
use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Deref};

pub struct SimplePostOrder<IdN, IdD> {
    pub(super) basic: BasicPostOrder<IdN, IdD>,
    pub(super) id_parent: Box<[IdD]>,
}

impl<IdN, IdD> Deref for SimplePostOrder<IdN, IdD> {
    type Target = BasicPostOrder<IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl<IdN, IdD> SimplePostOrder<IdN, IdD> {
    pub(crate) fn as_slice(&self) -> SimplePOSlice<'_, IdN, IdD> {
        SimplePOSlice {
            basic: self.basic.as_slice(),
            id_parent: &self.id_parent,
        }
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>> {
    pub(crate) fn as_basic(&self) -> Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = &self.basic;
        Decompressible { hyperast, decomp }
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>> {
    pub(crate) fn as_basic(&self) -> Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = &self.basic;
        Decompressible { hyperast, decomp }
    }
}

/// WIP WithParent (need some additional offset computations)
pub struct SimplePOSlice<'a, IdN, IdD> {
    pub(super) basic: BasicPOSlice<'a, IdN, IdD>,
    #[allow(unused)] // WIP
    pub(super) id_parent: &'a [IdD],
}

impl<'a, IdN, IdD> Clone for SimplePOSlice<'a, IdN, IdD> {
    fn clone(&self) -> Self {
        Self {
            basic: self.basic.clone(),
            id_parent: self.id_parent.clone(),
        }
    }
}

impl<'a, IdN, IdD> Copy for SimplePOSlice<'a, IdN, IdD> {}

impl<'a, T: Stored, IdD> Deref for SimplePOSlice<'a, T, IdD> {
    type Target = BasicPOSlice<'a, T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn _position_in_parent(&self, c: &IdD, p: &IdD) -> HAST::Idx {
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>
{
    type PIt = IterParents<'a, IdD>;
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
{
    type PIt = IterParents<'a, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithSiblings<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        let p = self.parent(x)?;
        let p_lld = self.first_descendant(&p);
        Self::lsib(self, x, &p_lld)
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
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.as_basic().lld(i)
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.as_basic().tree(id)
    }
}

// impl<HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>> {
//     pub(crate) fn size(&self, i: &IdD) -> IdD {
//         *i - self.llds[(*i).to_usize().unwrap()] + one()
//     }
// }

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> types::DecompressedFrom<HAST>
    for SimplePostOrder<HAST::IdN, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(hyperast: HAST, root: &HAST::IdN) -> Self {
        SimplePostOrder::make(hyperast, root)
    }
}

impl<'b, HAST: HyperAST + Copy, IdD: PrimInt> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(self, root: &HAST::IdN) -> Self {
        let hyperast = self.hyperast;
        let decomp = SimplePostOrder::make(hyperast, root);
        Decompressible { hyperast, decomp }
    }
}

impl<IdN, IdD: PrimInt> SimplePostOrder<IdN, IdD> {
    fn make<HAST: HyperAST<IdN = IdN> + Copy>(stores: HAST, root: &IdN) -> Self
    where
        IdN: types::NodeId<IdN = IdN>,
    {
        let aaa = Element::<_, _, IdD> {
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
                let x = stores.resolve(&curr);
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let node = self.hyperast.resolve(&a);
            let cs = node.children().filter(|x| !types::Childrn::is_empty(x));
            let Some(cs) = cs else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(cast(*d + one()).unwrap());
            let cs: Vec<_> = cs.iter_children().collect();
            for x in cs {
                z += size2(self.hyperast, &x);
            }
            r = self.first_descendant(&r) + cast(z).unwrap() - one();
        }
        r
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> FullyDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
}

impl<IdN, IdD: PrimInt> SimplePostOrder<IdN, IdD> {
    pub(crate) fn _first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()] // TODO use ldd
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.as_basic().is_descendant(desc, of)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecendantsLending<'a>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
{
    type Slice = SimplePOSlice<'a, HAST::IdN, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ContiguousDescendants<HAST, IdD>
    for Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        let range = self.as_basic().slice_range(x);
        SimplePOSlice {
            id_parent: &self.id_parent[range.clone()],
            basic: BasicPOSlice {
                id_compressed: &self.id_compressed[range.clone()],
                llds: &self.llds[range],
            },
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt>
    Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    pub(super) fn _slice(&self, x: &IdD) -> SimplePOSlice<'a, HAST::IdN, IdD> {
        let range = self.as_basic().slice_range(x);
        SimplePOSlice {
            id_parent: &self.id_parent[range.clone()],
            basic: BasicPOSlice {
                id_compressed: &self.id_compressed[range.clone()],
                llds: &self.llds[range],
            },
        }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Eq>
    Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>>
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

fn size2<HAST: HyperAST + Copy>(store: HAST, x: &HAST::IdN) -> usize
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    let tmp = store.resolve(x);
    let Some(cs) = tmp.children() else {
        return 1;
    };

    let mut z = 0;
    for x in cs.iter_children() {
        z += size2(store, &x);
    }
    z + 1
}

impl<IdN, IdD: PrimInt + Debug> Debug for SimplePostOrder<IdN, IdD>
where
    IdN: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("id_parent", &self.id_parent)
            .field("llds", &self.llds)
            .finish()
    }
}

pub struct RecCachedPositionProcessor<'a, HAST: HyperASTShared + Copy, IdD: Hash + Eq> {
    pub(crate) ds: Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
    root: HAST::IdN,
    cache: HashMap<IdD, Position>,
}

impl<'a, HAST: HyperASTShared + Copy, IdD: PrimInt + Hash + Eq>
    From<(
        Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
        HAST::IdN,
    )> for RecCachedPositionProcessor<'a, HAST, IdD>
{
    fn from(
        (ds, root): (
            Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
            HAST::IdN,
        ),
    ) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, HAST, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Debug,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithSerialization,
{
    pub fn position<'b>(&mut self, c: &IdD) -> &Position
    {
        let stores = self.ds.hyperast;
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
                    .unwrap_or_else(|| self.position(&p).clone());
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
                    .unwrap_or_else(|| self.position(&lsib).clone());
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
                    .unwrap_or_else(|| self.position(&p).clone());
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
pub struct RecCachedProcessor<'a, HAST: HyperASTShared + Copy, IdD: Hash + Eq, U, F, G> {
    pub(crate) ds: Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
    root: HAST::IdN,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, HAST: HyperASTShared + Copy, IdD: PrimInt + Hash + Eq, U, F, G>
    From<(
        Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
        HAST::IdN,
        F,
        G,
    )> for RecCachedProcessor<'a, HAST, IdD, U, F, G>
{
    fn from(
        (ds, root, with_p, with_lsib): (
            Decompressible<HAST, &'a SimplePostOrder<HAST::IdN, IdD>>,
            HAST::IdN,
            F,
            G,
        ),
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, HAST, IdD, U, F, G>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Debug,
    F: Fn(U, HAST::IdN) -> U,
    G: Fn(U, HAST::IdN) -> U,
{
    pub fn position<'b>(&mut self, c: &IdD) -> &U
    {
        let stores = self.ds.hyperast;
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
                let pos = self.position(&p).clone();
                return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
            }

            let p_lld = self.ds.first_descendant(&p);
            if let Some(lsib) = self.ds.lsib(c, &p_lld) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position(&lsib).clone();
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
                let pos = self.position(&p).clone();
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
