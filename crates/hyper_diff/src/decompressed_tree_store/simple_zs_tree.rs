use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use num_traits::{cast, one, zero};

use hyperast::types::{
    self, Children, Childrn, NodeId, NodeStore, Stored, WithChildren, WithStats,
};
use hyperast::PrimInt;

use super::{
    basic_post_order::BasicPostOrder, simple_post_order::SimplePostOrder, CIdx, CompletePostOrder,
    DecompressedTreeStore, InitializableWithStats, Iter, IterKr, PostOrdKeyRoots, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for the zs diff algo
/// - post order
/// - key roots
/// Compared to simple and complete post order it does not have parents
pub struct SimpleZsTree<T: Stored, IdD> {
    basic: BasicPostOrder<T, IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)=l(k’)}.
    pub(crate) kr: bitvec::boxed::BitBox,
}

impl<T: Stored, IdD> Deref for SimpleZsTree<T, IdD> {
    type Target = BasicPostOrder<T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl<T: Stored, IdD: PrimInt> From<SimplePostOrder<T, IdD>> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn from(simple: SimplePostOrder<T, IdD>) -> Self {
        let kr = simple.compute_kr_bitset();
        let basic = simple.basic;
        Self { basic, kr }
    }
}

impl<T: Stored, IdD: PrimInt> From<CompletePostOrder<T, IdD>> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn from(complete: CompletePostOrder<T, IdD>) -> Self {
        let basic = complete.simple.basic;
        Self {
            basic,
            kr: complete.kr,
        }
    }
}

// impl<'a, T: Stored, IdD: PrimInt> types::NLending<'a, T::TreeId> for SimpleZsTree<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

impl<T: Stored, IdD: PrimInt> PostOrder<T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.basic.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.basic.tree(id)
    }
}

impl<T: Stored, IdD: PrimInt> PostOrderIterable<T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.basic.iter_df_post::<ROOT>()
    }
}

impl<'a, T: Stored, IdD: PrimInt> PostOrdKeyRoots<'a, T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    type Iter = IterKr<'a, IdD>;
}

impl<T: Stored, IdD: PrimInt> PostOrderKeyRoots<T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, T, IdD>>::Iter {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

impl<'a, T: Stored, IdD: PrimInt + Debug> super::DecompressedSubtree<T> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    // T::TreeId: Clone + NodeId<IdN = T::TreeId>,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
    // T::Type: Copy + Eq + Send + Sync,
{
    type Out = Self;

    // #[time("warn")]
    fn decompress<S>(store: &S, root: &T::TreeId) -> SimpleZsTree<T, IdD>
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        let basic = BasicPostOrder::<T, IdD>::decompress(store, root);
        let kr = basic.compute_kr_bitset();
        Self { basic, kr }
    }

    fn decompress2<HAST>(store: &HAST, root: &<T as Stored>::TreeId) -> Self::Out
    where
        T: for<'t> types::AstLending<'t>,
        HAST: types::HyperAST<IdN = <T as Stored>::TreeId, TM = T>,
    {
        let basic = BasicPostOrder::<T, IdD>::decompress2(store, root);
        let kr = basic.compute_kr_bitset();
        Self { basic, kr }
    }
}

impl<T: Stored, IdD: PrimInt + Debug> InitializableWithStats<T> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn considering_stats<S>(store: &S, root: &<T as Stored>::TreeId) -> Self
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let pred_len = store.resolve(root).size();
        struct R<IdC, Idx, IdD> {
            curr: IdC,
            idx: Idx,
            lld: IdD,
        }

        let mut stack = vec![R {
            curr: root.clone(),
            idx: zero(),
            lld: zero(),
        }];
        let mut llds: Vec<IdD> = vec![];
        let mut id_compressed: Vec<T::TreeId> = vec![];
        while let Some(ele) = stack.pop() {
            let R { curr, idx, lld } = ele;
            let x = store.resolve(&curr);
            let l = x.children();
            let l = l.as_ref().filter(|x| !x.is_empty());
            if let Some(child) = l.and_then(|l| l.get(idx)) {
                stack.push(R {
                    curr,
                    idx: idx + one(),
                    lld,
                });
                stack.push(R {
                    curr: child.clone(),
                    idx: zero(),
                    lld: zero(),
                });
            } else {
                let value = if l.is_none() {
                    cast(id_compressed.len()).unwrap()
                } else {
                    lld
                };
                if let Some(tmp) = stack.last_mut() {
                    if tmp.idx == one() {
                        tmp.lld = value;
                    }
                }
                id_compressed.push(curr.clone());
                llds.push(value);
            }
        }

        id_compressed.shrink_to_fit();
        let id_compressed = id_compressed.into_boxed_slice();
        llds.shrink_to_fit();
        let llds = llds.into_boxed_slice();
        assert_eq!(id_compressed.len(), pred_len);
        assert_eq!(llds.len(), pred_len);

        let basic = BasicPostOrder {
            id_compressed,
            llds,
            _phantom: std::marker::PhantomData,
        };
        let kr = basic.compute_kr_bitset();
        Self { basic, kr }
    }
}

impl<T: Stored, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn len(&self) -> usize {
        self.basic.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.basic.original(id)
    }

    fn root(&self) -> IdD {
        self.basic.root()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: NodeStore<T::TreeId, NMarker = T>,
    {
        self.basic.child(store, x, p)
    }
    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.basic.child4(store, x, p)
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.basic.children(store, x)
    }
    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.basic.children4(store, x)
    }
}

impl<T: Stored, IdD: PrimInt> DecompressedTreeStore<T, IdD> for SimpleZsTree<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.basic.descendants(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.basic.first_descendant(i)
    }

    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let r = (self.lld(x) + one() - *x).to_usize().unwrap();
        assert!(r == self.basic.descendants_count(store, x));
        r
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for SimpleZsTree<T, IdD>
where
    T::TreeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("llds", &self.llds)
            .field("kr", &self.kr)
            .finish()
    }
}
