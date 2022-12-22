use std::{fmt::Debug, ops::Deref};

use bitvec::bitvec;
use num_traits::{cast, one, zero, PrimInt};

use crate::decompressed_tree_store::basic_post_order::BasicPOSlice;
use hyper_ast::types::{Children, IterableChildren, NodeStore, Stored, WithChildren, WithStats};

use super::{
    basic_post_order::BasicPostOrder, simple_post_order::SimplePostOrder, CompletePostOrder,
    DecompressedTreeStore, Initializable, InitializableWithStats, Iter, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for the zs diff algo
/// - post order
/// - key roots
/// Compared to simple and complete post order it does not have parents
pub struct SimpleZsTree<T: Stored, IdD> {
    basic: BasicPostOrder<T, IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)=l(k’)}.
    pub(crate) kr: Box<[IdD]>,
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
        let kr = simple.compute_kr();
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
        self.basic.lld(&(*i - one()))
        // self.basic.lld(i) // TODO Remove bad fixing offset
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        let r = self.id_compressed[id.to_usize().unwrap() - 1].clone();
        assert!(r == self.basic.tree(&(*id - one())));
        r
        // self.basic.tree(id) // TODO Remove bad fixing offset
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderIterable<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    type It = Iter<IdD>;
    fn iter_df_post(&self) -> Iter<IdD> {
        self.basic.iter_df_post()
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrderKeyRoots<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    //     fn kr(&self, x: IdD) -> IdD {
    //         self.kr[x.to_usize().unwrap()]
    //     }
    fn iter_kr(&self) -> std::slice::Iter<'_, IdD> {
        self.kr.iter()
    }
}

impl<'a, T: WithChildren, IdD: PrimInt + Debug> Initializable<'a, T> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn new<S>(store: &'a S, root: &T::TreeId) -> SimpleZsTree<T, IdD>
    where
        S: NodeStore<T::TreeId, R<'a> = T>,
    {
        struct R<IdC, Idx, IdD> {
            curr: IdC,
            idx: Idx,
            lld: IdD,
        }

        let mut leaf_count = 0;
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
                    leaf_count += 1;
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

        let node_count = id_compressed.len();
        let mut kr = vec![num_traits::zero(); leaf_count + 1];
        let mut visited = bitvec![0;node_count];
        let mut k = kr.len() - 1;
        for i in (1..node_count).rev() {
            if !visited[llds[i].to_usize().unwrap()] {
                kr[k] = cast(i + 1).unwrap();
                visited.set(llds[i].to_usize().unwrap(), true);
                if k > 0 {
                    k -= 1;
                }
            }
        }
        id_compressed.shrink_to_fit();
        let id_compressed = id_compressed.into_boxed_slice();
        llds.shrink_to_fit();
        let llds = llds.into_boxed_slice();
        kr.shrink_to_fit();
        kr.reverse();
        kr.pop();
        let kr = kr.into_boxed_slice();

        let basic = BasicPOSlice::<T, IdD> {
            id_compressed: &id_compressed,
            llds: &llds,
            _phantom: std::marker::PhantomData,
        };
        let basic_kr = basic.compute_kr();
        assert_eq!(kr, basic_kr);

        Self {
            basic: BasicPostOrder {
                id_compressed,
                llds,
                _phantom: std::marker::PhantomData,
            },
            kr,
        }
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

        let mut leaf_count = 0;
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
                    leaf_count += 1;
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

        let node_count = id_compressed.len();
        let mut kr = vec![num_traits::zero(); leaf_count + 1];
        let mut visited = bitvec![0;node_count];
        let mut k = kr.len() - 1;
        for i in (1..node_count).rev() {
            if !visited[llds[i].to_usize().unwrap()] {
                kr[k] = cast(i + 1).unwrap();
                visited.set(llds[i].to_usize().unwrap(), true);
                if k > 0 {
                    k -= 1;
                }
            }
        }
        id_compressed.shrink_to_fit();
        let id_compressed = id_compressed.into_boxed_slice();
        llds.shrink_to_fit();
        let llds = llds.into_boxed_slice();
        kr.shrink_to_fit();
        let kr = kr.into_boxed_slice();
        assert_eq!(id_compressed.len(), pred_len);
        assert_eq!(llds.len(), pred_len);
        Self {
            basic: BasicPostOrder {
                id_compressed,
                llds,
                _phantom: std::marker::PhantomData,
            },
            kr,
        }
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
        let r = (self.lld(x) - *x).to_usize().unwrap();
        assert!(r == self.basic.descendants_count(store, x));
        r
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
