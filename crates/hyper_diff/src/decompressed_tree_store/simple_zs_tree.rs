use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use num_traits::{cast, one, zero};

use hyperast::types::{self, Children, Childrn, HyperAST, WithChildren, WithStats};
use hyperast::PrimInt;

use crate::matchers::Decompressible;

use super::{
    basic_post_order::BasicPostOrder, simple_post_order::SimplePostOrder, CompletePostOrder,
    DecompressedTreeStore, InitializableWithStats, Iter, IterKr, PostOrdKeyRoots, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for the zs diff algo
/// - post order
/// - key roots
/// Compared to simple and complete post order it does not have parents
pub struct SimpleZsTree<IdN, IdD> {
    basic: BasicPostOrder<IdN, IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)=l(k’)}.
    pub(crate) kr: bitvec::boxed::BitBox,
}

impl<IdN, IdD> Deref for SimpleZsTree<IdN, IdD> {
    type Target = BasicPostOrder<IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.basic
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>> {
    pub(crate) fn as_basic(&self) -> Decompressible<HAST, &BasicPostOrder<HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = &self.basic;
        Decompressible { hyperast, decomp }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt>
    From<Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
{
    fn from(simple: Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>) -> Self {
        let decomp = simple.decomp.basic;
        let hyperast = simple.hyperast;
        let basic = Decompressible { hyperast, decomp };
        let kr = basic.compute_kr_bitset();
        let basic = basic.decomp;
        let decomp = SimpleZsTree { basic, kr };
        Decompressible { hyperast, decomp }
    }
}

impl<IdN, IdD: PrimInt> From<CompletePostOrder<IdN, IdD>> for SimpleZsTree<IdN, IdD> {
    fn from(complete: CompletePostOrder<IdN, IdD>) -> Self {
        let basic = complete.simple.basic;
        Self {
            basic,
            kr: complete.kr,
        }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrderIterable<HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.as_basic().iter_df_post::<ROOT>()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrdKeyRoots<'a, HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Iter = IterKr<'a, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrderKeyRoots<HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(self, root: &HAST::IdN) -> Self {
        let hyperast = self.hyperast;
        let basic = self.decomp.basic;
        let basic = Decompressible {
            hyperast,
            decomp: basic,
        };
        let basic = basic.decompress(root);
        let kr = basic.compute_kr_bitset();
        let basic = basic.decomp;
        let decomp = SimpleZsTree { basic, kr };
        Decompressible { hyperast, decomp }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> types::DecompressedFrom<HAST>
    for SimpleZsTree<HAST::IdN, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(hyperast: HAST, root: &HAST::IdN) -> Self {
        let basic = BasicPostOrder::decompress(hyperast, root);

        let basic = Decompressible {
            hyperast,
            decomp: basic,
        };
        let kr = basic.compute_kr_bitset();
        let basic = basic.decomp;
        let decomp = SimpleZsTree { basic, kr };
        decomp
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Debug> InitializableWithStats<HAST::IdN>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithStats,
{
    fn considering_stats(&self, root: &HAST::IdN) -> Self {
        let pred_len = self.hyperast.resolve(root).size();
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
        let mut id_compressed: Vec<HAST::IdN> = vec![];
        while let Some(ele) = stack.pop() {
            let R { curr, idx, lld } = ele;
            let x = self.hyperast.resolve(&curr);
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
        };
        let hyperast = self.hyperast;
        let basic = Decompressible {
            hyperast,
            decomp: basic,
        };
        let kr = basic.compute_kr_bitset();
        let basic = basic.decomp;
        let decomp = SimpleZsTree { basic, kr };
        Decompressible { hyperast, decomp }
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self.as_basic().len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.as_basic().original(id)
    }

    fn root(&self) -> IdD {
        self.as_basic().root()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        self.as_basic().child(x, p)
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        self.as_basic().children(x)
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        self.as_basic().descendants(x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.as_basic().first_descendant(i)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        let r = (self.lld(x) + one() - *x).to_usize().unwrap();
        assert!(r == self.as_basic().descendants_count(x));
        r
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        desc < of && &self.first_descendant(of) <= desc
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt + Debug> Debug
    for Decompressible<HAST, SimpleZsTree<HAST::IdN, IdD>>
where
    HAST::IdN: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePostOrder")
            .field("id_compressed", &self.id_compressed)
            .field("llds", &self.llds)
            .field("kr", &self.kr)
            .finish()
    }
}
