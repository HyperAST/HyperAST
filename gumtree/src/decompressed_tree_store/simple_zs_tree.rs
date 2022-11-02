use num_traits::{cast, one, zero, PrimInt};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{NodeStore, WithChildren};

use super::{
    size, DecompressedTreeStore, Initializable, Iter, PostOrder, PostOrderIterable,
    PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for the zs diff algo
/// - post order
/// - key roots
#[derive(Debug)]
pub struct SimpleZsTree<IdC, IdD> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)=l(k’)}.
    kr: Vec<IdD>,
}

impl<'d, IdC: Clone, IdD: PrimInt> PostOrder<'d, IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).to_usize().unwrap() - 1].clone()
    }
}

impl<'d, IdC: Clone, IdD: PrimInt> PostOrderIterable<'d, IdC, IdD> for SimpleZsTree<IdC, IdD> {
    type It = Iter<IdD>;
    fn iter_df_post(&self) -> Iter<IdD> {
        Iter {
            current: zero(),
            len: (cast(self.id_compressed.len())).unwrap(),
        }
    }
}

impl<'d, IdC: Clone, IdD: PrimInt> PostOrderKeyRoots<'d, IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.to_usize().unwrap()]
    }
}

impl<'d, IdC: Clone, IdD: PrimInt> Initializable<'d, IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn new<
        S,
    >(
        store: &'d S,
        root: &IdC,
    ) -> SimpleZsTree<IdC, IdD>
    where
        S: 'd + NodeStore<IdC>,
        S::R<'d>: WithChildren<TreeId = IdC>,
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
        let mut id_compressed: Vec<IdC> = vec![];
        loop {
            if let Some(R { curr, idx, lld }) = stack.pop() {
                let x = store.resolve(&curr);
                let l = x.try_get_children().map_or(zero(), |x|cast(x.len()).unwrap());

                if l == zero() {
                    // leaf
                    let lld = cast(id_compressed.len()).unwrap();
                    if let Some(tmp) = stack.last_mut() {
                        if tmp.idx == one() {
                            tmp.lld = lld;
                        }
                    }
                    llds.push(lld);
                    id_compressed.push(curr.clone());
                    leaf_count += 1;
                } else if idx < l {
                    //
                    let child = x.get_child(&idx).clone();
                    stack.push(R {
                        curr,
                        idx: idx + one(),
                        lld: lld,
                    });
                    stack.push(R {
                        curr: child,
                        idx: zero(),
                        lld: zero(),
                    });
                } else {
                    if let Some(tmp) = stack.last_mut() {
                        if tmp.idx == one() {
                            tmp.lld = lld;
                        }
                    }
                    id_compressed.push(curr.clone());
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
        }
    }
}

impl<'d, IdC: Clone, IdD: PrimInt> ShallowDecompressedTreeStore<'d, IdC, IdD>
    for SimpleZsTree<IdC, IdD>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).to_usize().unwrap()].clone()
    }

    fn leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
    }

    fn root(&self) -> IdD {
        cast::<_, IdD>(self.len()).unwrap() - one() // todo test changing it
    }

    fn child<'b,S>(&self, store: &'b S, x: &IdD, p: &[<S::R<'b> as WithChildren>::ChildIdx]) -> IdD
    where
        S: NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.resolve(&a).get_children().to_owned();
            if cs.len() > 0 {
                let mut z = 0;
                for x in cs[0..cast::<_, usize>(*d).unwrap() + 1].to_owned() {
                    z += size(store, &x);
                }
                r = self.lld(&r) + cast(z).unwrap() - one();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<'b,S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        let a = self.original(x);
        let cs: Vec<_> = store.resolve(&a).get_children().to_owned();
        let mut r = vec![];
        let mut c = self.lld(x);
        for x in cs.to_owned() {
            c = c + cast(size(store, &x)).unwrap() - one();
            r.push(c);
        }
        r
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> CompressedTreePath<Idx> {
        todo!()
    }
}
impl<'d, IdC: Clone, IdD: PrimInt> DecompressedTreeStore<'d, IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn descendants<'b,S>(&self, _store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        (cast::<_, usize>(self.lld(x)).unwrap()..cast::<_, usize>(*x).unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).to_usize().unwrap()]
    }

    fn descendants_count<'b,S>(&self, _store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        cast::<_, usize>(self.lld(x) - *x).unwrap()
    }
}
