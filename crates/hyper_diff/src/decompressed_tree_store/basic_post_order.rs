use super::{
    ContiguousDescendants, DecendantsLending, DecompressedTreeStore, Iter, PostOrder,
    PostOrderIterable, ShallowDecompressedTreeStore,
};
use crate::matchers::Decompressible;
use hyperast::types::{self, Children, Childrn, HyperAST, WithChildren};
use hyperast::PrimInt;
use num_traits::{cast, one, zero, ToPrimitive};
use std::fmt::Debug;

pub struct BasicPostOrder<IdN, IdD> {
    /// Ids of subtrees in HyperAST
    pub(super) id_compressed: Box<[IdN]>,

    /// leftmost leaf descendant of nodes
    ///
    /// it is so powerful even the basic layout should keep it
    pub(crate) llds: Box<[IdD]>,
}

impl<IdN, IdD> BasicPostOrder<IdN, IdD> {
    pub fn as_slice(&self) -> BasicPOSlice<'_, IdN, IdD> {
        BasicPOSlice {
            id_compressed: &self.id_compressed,
            llds: &self.llds,
        }
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, BasicPostOrder<HAST::IdN, IdD>> {
    pub fn as_slice(&self) -> Decompressible<HAST, BasicPOSlice<'_, HAST::IdN, IdD>> {
        Decompressible {
            hyperast: self.hyperast,
            decomp: self.decomp.as_slice(),
        }
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>> {
    pub fn as_slice(&self) -> Decompressible<HAST, BasicPOSlice<'_, HAST::IdN, IdD>> {
        Decompressible {
            hyperast: self.hyperast,
            decomp: self.decomp.as_slice(),
        }
    }
}

impl<IdN, IdD: PrimInt> BasicPostOrder<IdN, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &IdN> {
        self.id_compressed.iter()
    }
}

impl<T: Debug, IdD: PrimInt + Debug> Debug for BasicPostOrder<T, IdD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("llds", &self.llds)
            .finish()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrderIterable<HAST, IdD>
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
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

impl<'d, T, IdD: PrimInt> BasicPostOrder<T, IdD> {
    pub(crate) fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, BasicPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;
    fn decompress(self, root: &HAST::IdN) -> Self {
        let hyperast = self.hyperast;
        let simple = BasicPostOrder::make(hyperast, root);
        Decompressible {
            hyperast,
            decomp: simple,
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> types::DecompressedFrom<HAST>
    for BasicPostOrder<HAST::IdN, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(hyperast: HAST, root: &HAST::IdN) -> Self {
        BasicPostOrder::make(hyperast, root)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt>
    Decompressible<HAST, BasicPOSlice<'a, HAST::IdN, IdD>>
{
    /// WARN oposite order than id_compressed
    pub fn compute_kr(&self) -> Box<[IdD]> {
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
    pub fn compute_kr_bitset(&self) -> bitvec::boxed::BitBox {
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, BasicPostOrder<HAST::IdN, IdD>> {
    /// WARN oposite order than id_compressed
    pub fn compute_kr(&self) -> Box<[IdD]> {
        self.as_slice().compute_kr()
    }

    /// use a bitset to mark key roots
    ///
    /// should be easier to split and maybe more efficient
    pub(crate) fn compute_kr_bitset(&self) -> bitvec::boxed::BitBox {
        self.as_slice().compute_kr_bitset()
    }
}

impl<IdN, IdD: PrimInt> BasicPostOrder<IdN, IdD> {
    fn make<HAST: HyperAST<IdN = IdN> + Copy>(store: HAST, root: &IdN) -> Self
    where
        IdN: types::NodeId<IdN = IdN>,
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
            let children1: Option<
                _, // &<<S as NLending<'_, T::TreeId>>::N as types::CLending<'_, _, T::TreeId>>::Children,
            > = WithChildren::children(&x);
            let l: Option<Option<_>> = children1
                .filter(|x| !x.is_empty())
                .map(|l| l.get(idx).cloned());
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
                let value = if l.is_some() { curr_idx } else { lld };
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
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
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
            use WithChildren;
            let cs = node.children();
            let cs = cs.filter(|x| !types::Childrn::is_empty(x));
            let Some(cs) = cs else {
                panic!("no children in this tree")
            };
            let mut z = 0;
            let cs = cs.before(cast(*d + one()).unwrap());
            let cs: Vec<HAST::IdN> = cs.iter_children().collect();
            for x in cs {
                z += tree_size(self.hyperast, x);
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
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
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    pub(super) fn slice_range(&self, x: &IdD) -> std::ops::RangeInclusive<usize> {
        self.first_descendant(x).to_usize().unwrap()..=x.to_usize().unwrap()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecendantsLending<'a>
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
{
    type Slice = BasicPOSlice<'a, HAST::IdN, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ContiguousDescendants<HAST, IdD>
    for Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        let range = self.slice_range(x);
        BasicPOSlice {
            id_compressed: &self.id_compressed[range.clone()],
            llds: &self.llds[range],
        }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Eq>
    Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>>
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

fn tree_size<HAST>(store: HAST, x: HAST::IdN) -> usize
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST + Copy,
{
    let tmp = store.resolve(&x);
    let Some(cs) = tmp.children() else {
        return 1;
    };

    let mut z = 0;
    for x in cs {
        z += tree_size(store, x);
    }
    z + 1
}

pub struct BasicPOSlice<'a, IdN, IdD> {
    /// Ids of subtrees in HyperAST
    pub(super) id_compressed: &'a [IdN],
    /// leftmost leaf descendant of nodes
    ///
    /// it is so powerful even the basic layout should keep it
    pub(crate) llds: &'a [IdD],
}

impl<'a, IdN, IdD> Clone for BasicPOSlice<'a, IdN, IdD> {
    fn clone(&self) -> Self {
        Self {
            id_compressed: self.id_compressed,
            llds: self.llds,
        }
    }
}

impl<'a, IdN, IdD> Copy for BasicPOSlice<'a, IdN, IdD> {}

impl<'d, IdN, IdD: PrimInt> BasicPOSlice<'d, IdN, IdD> {
    pub(crate) fn size(&self, i: &IdD) -> IdD {
        *i - self.llds[(*i).to_usize().unwrap()] + one()
    }
    fn _lld(&self, i: usize) -> IdD {
        self.llds[i] - self.llds[0]
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, BasicPOSlice<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self._lld(i.to_usize().unwrap())
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, BasicPOSlice<'a, HAST::IdN, IdD>>
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
            r = {
                let cs = node.children().filter(|x| x.is_empty());
                let Some(cs) = cs else {
                    panic!("no children in this tree")
                };
                let mut z = 0;
                let cs = cs.before(cast(*d + one()).unwrap());
                let cs = cs.iter_children();
                let cs: Vec<_> = cs.collect();
                for x in cs {
                    z += tree_size(self.hyperast, x);
                }
                self.first_descendant(&r) + cast(z).unwrap() - one()
            };
        }
        r
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        let a = self.original(x);
        let cs_len = self.hyperast.resolve(&a).child_count().to_usize().unwrap();
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, BasicPOSlice<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        (self.first_descendant(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.lld(i)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        (*x - self.first_descendant(x) + one()).to_usize().unwrap()
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}
