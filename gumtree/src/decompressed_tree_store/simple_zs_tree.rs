use std::fmt::{Debug, Display};

use bitvec::bitvec;
use num_traits::{cast, one, zero, PrimInt};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{
    Children, IterableChildren, LabelStore, NodeStore, Stored, Tree, WithChildren,
    WithSerialization,
};

use super::{
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    DecompressedTreeStore, Initializable, Iter, PostOrder, PostOrderIterable, PostOrderKeyRoots,
    ShallowDecompressedTreeStore,
};

/// made for the zs diff algo
/// - post order
/// - key roots
#[derive(Debug)]
pub struct SimpleZsTree<T: Stored, IdD> {
    leaf_count: IdD,
    id_compressed: Vec<T::TreeId>,
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)=l(k’)}.
    kr: Vec<IdD>,
    _phantom: std::marker::PhantomData<*const T>,
}

impl<'d, T: Stored, IdD: PrimInt> SimpleZsTree<T, IdD> {
    pub fn leaf_count(&self) -> IdD {
        self.leaf_count
    }
}

impl<'d, T: WithChildren, IdD: PrimInt> SimpleZsTree<T, IdD> {
    fn size<'a, S>(store: &'a S, x: &T::TreeId) -> usize
    where
        S: NodeStore<T::TreeId, R<'a> = T>,
    {
        let tmp = store.resolve(x);
        let Some(cs) = tmp.children() else {
            return 1;
        };

        let mut z = 0;
        for x in cs.iter_children() {
            z += Self::size(store, x);
        }
        z + 1
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrder<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[(*id).to_usize().unwrap() - 1].clone()
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> PostOrderIterable<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    type It = Iter<IdD>;
    fn iter_df_post(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'d, T: 'd + WithChildren, IdD: PrimInt> PostOrderKeyRoots<'d, T, IdD> for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.to_usize().unwrap()]
    }
}

impl<'a, T: WithChildren, IdD: PrimInt> Initializable<'a, T> for SimpleZsTree<T, IdD>
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
        let leaf_count = cast(leaf_count).unwrap();
        id_compressed.shrink_to_fit();
        llds.shrink_to_fit();
        kr.shrink_to_fit();
        Self {
            leaf_count,
            id_compressed,
            llds,
            kr,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> ShallowDecompressedTreeStore<'d, T, IdD>
    for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[(*id).to_usize().unwrap()].clone()
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
            let cs = node.children().unwrap();
            // TODO use lld to decide if it is a leaf, should make things faster and cleaner
            if cs.is_empty() {
                panic!("no children in this tree")
            } else {
                let mut z = 0;
                // TODO going in reverse it should be possible to use the llds to get left sibling
                // also possible to use parents to move faster to the right
                // but it should be simple and fst to get the size of subtree in metadata
                // NOTE #descendants x == size x + 1
                let cs = cs.before(*d + one());
                for x in cs.iter_children() {
                    z += Self::size(store, x); // TODO check if we can make it significantly faster using metadata
                }
                r = self.lld(&r) + cast(z).unwrap() - one();
            }
        }
        r
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let a = self.original(x);
        let node = store.resolve(&a);
        let cs = node.children().unwrap();
        let mut r = vec![];
        let mut c = self.lld(x);
        for x in cs.iter_children() {
            c = c + cast(Self::size(store, &x)).unwrap() - one();
            r.push(c);
        }
        r
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> CompressedTreePath<Idx> {
        todo!()
    }
}

impl<'d, T: WithChildren + 'd, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD>
    for SimpleZsTree<T, IdD>
where
    T::TreeId: Clone,
{
    fn descendants<'b, S>(&self, _store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>, // S: 'b + NodeStore<IdC>,
                                            // S::R<'b>: WithChildren<TreeId = IdC>,
                                            // <S::R<'b> as WithChildren>::ChildIdx: PrimInt,
                                            // for<'c> <S::R<'b> as WithChildren>::Children<'c>:
                                            //     types::Children<<S::R<'b> as WithChildren>::ChildIdx, IdC>,
    {
        (self.lld(x).to_usize().unwrap()..x.to_usize().unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.lld(i)
    }

    fn descendants_count<'b, S>(&self, _store: &'b S, x: &IdD) -> usize
    where
        S: NodeStore<T::TreeId, R<'b> = T>, // S: 'b + NodeStore<IdC>,
                                            // S::R<'b>: WithChildren<TreeId = IdC>,
                                            // <S::R<'b> as WithChildren>::ChildIdx: PrimInt,
                                            // for<'c> <S::R<'b> as WithChildren>::Children<'c>:
                                            //     types::Children<<S::R<'b> as WithChildren>::ChildIdx, IdC>,
    {
        (self.lld(x) - *x).to_usize().unwrap()
    }
}

pub struct DisplaySimpleZsTree<'a, T: Stored, IdD: PrimInt, S, LS>
where
    LS: LabelStore<str>,
{
    pub inner: &'a SimpleZsTree<T, IdD>,
    pub node_store: &'a S,
    pub label_store: &'a LS,
}

impl<'a, T: Tree + WithSerialization, IdD: PrimInt, S, LS> Display
    for DisplaySimpleZsTree<'a, T, IdD, S, LS>
where
    S: NodeStore<T::TreeId, R<'a> = T>,
    T::TreeId: Clone + Eq + Debug,
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
