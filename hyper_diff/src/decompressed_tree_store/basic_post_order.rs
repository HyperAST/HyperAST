use std::fmt::Debug;

use num_traits::{cast, one, zero, PrimInt, ToPrimitive};

use hyper_ast::types::{self, Children, IterableChildren, NodeStore, Stored, WithChildren};

use super::{
    ContiguousDescendants, DecompressedTreeStore, Iter, PostOrder,
    PostOrderIterable, ShallowDecompressedTreeStore,
};

pub struct BasicPostOrder<T: Stored, IdD> {
    /// Ids of subtrees in HyperAST
    pub(super) id_compressed: Box<[T::TreeId]>,

    /// leftmost leaf descendant of nodes
    ///
    /// it is so powerful even the basic layout should keep it
    pub(crate) llds: Box<[IdD]>,
    pub(super) _phantom: std::marker::PhantomData<*const T>,
}

impl<T: Stored, IdD> BasicPostOrder<T, IdD> {
    pub fn as_slice(&self) -> BasicPOSlice<'_, T, IdD> {
        BasicPOSlice {
            id_compressed: &self.id_compressed,
            llds: &self.llds,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Stored, IdD: PrimInt> BasicPostOrder<T, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &T::TreeId> {
        self.id_compressed.iter()
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for BasicPostOrder<T, IdD>
where
    T::TreeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("llds", &self.llds)
            .finish()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> PostOrder<'a, T, IdD> for BasicPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderIterable<'d, T, IdD>
    for BasicPostOrder<T, IdD>
where
    T::TreeId: Clone,
{
    // TODO add a lifetime to make sure the len does not change
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

impl<'d, T: WithChildren, IdD: PrimInt> BasicPostOrder<T, IdD> {
    pub(crate) fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
}

impl<'a, T, IdD: PrimInt> super::DecompressedSubtree<'a, T> for BasicPostOrder<T, IdD>
where
    T: WithChildren,
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn decompress<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
    {
        let simple = BasicPostOrder::make(store, root);
        let BasicPostOrder::<T, IdD> {
            id_compressed,
            llds,
            _phantom,
        } = simple;

        Self {
            id_compressed,
            llds,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T, IdD: PrimInt> BasicPOSlice<'a, T, IdD>
where
    T: WithChildren,
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    /// WARN oposite order than id_compressed
    pub fn compute_kr(&self) -> Box<[IdD]>
where {
        let node_count = self.id_compressed.len();
        let mut kr = Vec::with_capacity(node_count);
        let mut visited = bitvec::bitvec![0; node_count];
        for i in (1..node_count).rev() {
            if !visited[self._lld(i).to_usize().unwrap()] {
                kr.push(cast(i).unwrap());
                visited.set(self._lld(i).to_usize().unwrap(), true);
            }
        }
        kr.into_boxed_slice()
    }

    /// use a bitset to mark key roots
    ///
    /// should be easier to split and maybe more efficient
    pub fn compute_kr_bitset(&self) -> bitvec::boxed::BitBox
where {
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

impl<'a, T, IdD: PrimInt> BasicPostOrder<T, IdD>
where
    T: WithChildren,
    T::TreeId: Clone,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    /// WARN oposite order than id_compressed
    pub(crate) fn compute_kr(&self) -> Box<[IdD]> {
        self.as_slice().compute_kr()
    }

    /// use a bitset to mark key roots
    ///
    /// should be easier to split and maybe more efficient
    pub(crate) fn compute_kr_bitset(&self) -> bitvec::boxed::BitBox {
        self.as_slice().compute_kr_bitset()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> BasicPostOrder<T, IdD>
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
                let value = if l.is_none() { curr_idx } else { lld };
                if let Some(tmp) = stack.last_mut() {
                    if tmp.idx == one() {
                        tmp.lld = value;
                    }
                    tmp.children.push(curr_idx);
                }
                llds.push(value);
                id_compressed.push(curr);
            }
        }
        let id_compressed = id_compressed.into();
        let llds = llds.into();
        BasicPostOrder {
            id_compressed,
            llds,
            _phantom: Default::default(),
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
    for BasicPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq,
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
}

impl<'d, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD> for BasicPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq,
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
    
    fn is_descendant(&self, desc: &IdD,of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> BasicPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    pub(super) fn slice_range(&self, x: &IdD) -> std::ops::RangeInclusive<usize> {
        self.first_descendant(x).to_usize().unwrap()..=x.to_usize().unwrap()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> ContiguousDescendants<'d, T, IdD>
    for BasicPostOrder<T, IdD>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    type Slice<'b> = BasicPOSlice<'b,T,IdD> where Self: 'b;

    fn slice(&self, x: &IdD) -> Self::Slice<'_> {
        let range = self.slice_range(x);
        BasicPOSlice {
            id_compressed: &self.id_compressed[range.clone()],
            llds: &self.llds[range],
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T: WithChildren, IdD: PrimInt + Eq> BasicPostOrder<T, IdD>
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

impl<'a, T: WithChildren, IdD: PrimInt> BasicPostOrder<T, IdD>
where
    T::TreeId: Clone,
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

pub struct BasicPOSlice<'a, T: Stored, IdD> {
    /// Ids of subtrees in HyperAST
    pub(super) id_compressed: &'a [T::TreeId],
    /// leftmost leaf descendant of nodes
    ///
    /// it is so powerful even the basic layout should keep it
    pub(crate) llds: &'a [IdD],
    pub(super) _phantom: std::marker::PhantomData<*const T>,
}

impl<'d, T: WithChildren, IdD: PrimInt> BasicPOSlice<'d, T, IdD> {
    pub(crate) fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
    fn _lld(&self, i: usize) -> IdD {
        self.llds[i] - self.llds[0]
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> PostOrder<'a, T, IdD> for BasicPOSlice<'a, T, IdD>
where
    T::TreeId: Clone + Eq,
{
    fn lld(&self, i: &IdD) -> IdD {
        self._lld(i.to_usize().unwrap())
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<'a, T, IdD>
    for BasicPOSlice<'a, T, IdD>
where
    T::TreeId: Clone + Eq,
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
            let cs = node.children().filter(|x| x.is_empty());
            let Some(cs) = cs  else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(*d + one());
            let cs: Vec<T::TreeId> = cs.iter_children().cloned().collect();
            for x in cs {
                z += BasicPostOrder::<T, IdD>::size2::<S>(store, &x);
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

impl<'a, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<'a, T, IdD>
    for BasicPOSlice<'a, T, IdD>
where
    T::TreeId: Clone + Eq,
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
        self.lld(i)
    }

    fn descendants_count<'b, S>(&self, _store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }
    
    fn is_descendant(&self, desc: &IdD,of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}
