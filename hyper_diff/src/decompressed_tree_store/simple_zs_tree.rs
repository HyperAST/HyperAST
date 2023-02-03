use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use num_traits::{cast, one, zero, PrimInt};

use hyper_ast::types::{Children, IterableChildren, NodeStore, Stored, WithChildren, WithStats};

use super::{
    basic_post_order::BasicPostOrder, simple_post_order::SimplePostOrder, CompletePostOrder,
    DecompressedTreeStore, InitializableWithStats, Iter, IterKr, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

use logging_timer::time;

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

impl<T: WithChildren, IdD: PrimInt> From<SimplePostOrder<T, IdD>> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn from(simple: SimplePostOrder<T, IdD>) -> Self {
        let kr = simple.compute_kr_bitset();
        let basic = simple.basic;
        Self { basic, kr }
    }
}

impl<T: WithChildren, IdD: PrimInt> From<CompletePostOrder<T, IdD>> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn from(complete: CompletePostOrder<T, IdD>) -> Self {
        let basic = complete.simple.basic;
        Self {
            basic,
            kr: complete.kr,
        }
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrder<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.basic.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.basic.tree(id)
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderIterable<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.basic.iter_df_post::<ROOT>()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrderKeyRoots<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    //     fn kr(&self, x: IdD) -> IdD {
    //         self.kr[x.to_usize().unwrap()]
    //     }

    type Iter<'b> = IterKr<'b,IdD>
    where
        Self: 'b;

    fn iter_kr(&self) -> Self::Iter<'_> {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

impl<'a, T: WithChildren, IdD: PrimInt + Debug> super::DecompressedSubtree<'a, T> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    #[time("warn")]
    fn decompress<S>(store: &'a S, root: &T::TreeId) -> SimpleZsTree<T, IdD>
    where
        S: NodeStore<T::TreeId, R<'a> = T>,
    {
        let basic = BasicPostOrder::<T, IdD>::decompress(store, root);
        let kr = basic.compute_kr_bitset();
        Self { basic, kr }
    }
}

impl<'a, T: WithChildren + WithStats, IdD: PrimInt + Debug> InitializableWithStats<'a, T>
    for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn considering_stats<S>(store: &'a S, root: &<T as Stored>::TreeId) -> Self
    where
        S: NodeStore<<T as Stored>::TreeId, R<'a> = T>,
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
            let l = l.filter(|x| !x.is_empty());
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

impl<'d, T: WithChildren + 'd, IdD: PrimInt> ShallowDecompressedTreeStore<'d, T, IdD>
    for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
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

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        self.basic.child(store, x, p)
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        self.basic.children(store, x)
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD>
    for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        self.basic.descendants(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.basic.first_descendant(i)
    }

    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let r = (self.lld(x) + one() - *x).to_usize().unwrap();
        assert!(r == self.basic.descendants_count(store, x));
        r
    }

    fn is_descendant(&self, desc: &IdD,of: &IdD) -> bool {
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
