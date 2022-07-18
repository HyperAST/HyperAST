use std::fmt::Debug;

use num_traits::{cast, one, zero, PrimInt, ToPrimitive};

use crate::tree::tree_path::CompressedTreePath;
use hyper_ast::types::{NodeStore, Stored, Tree, WithChildren};

use super::{
    size, DecompressedTreeStore, DecompressedWithParent, Initializable, Iter, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<IdC, IdD: PrimInt> {
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

impl<'d, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> DecompressedWithParent<'d, IdC, IdD>
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

    fn position_in_parent<S>(&self, _store: &S, c: &IdD) -> <S::R<'d> as WithChildren>::ChildIdx
    where
        S: NodeStore<IdC>,
        S::R<'d>: WithChildren<TreeId = IdC>,
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
}

impl<'d, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> PostOrder<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).into() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into() - 1].clone()
    }
}

impl<'d, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> PostOrderIterable<'d, IdC, IdD>
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

impl<'d, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> PostOrderKeyRoots<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.into()]
    }
}
impl<'d, IdC: Clone, IdD: PrimInt + Into<usize>> Initializable<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
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
                    let curr_idx = cast(id_compressed.len()).unwrap();
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
                        id_parent[cast::<_, usize>(x).unwrap()] = curr_idx;
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
            if !visited[llds[i].into()] {
                kr[k] = cast(i + 1).unwrap();
                visited[llds[i].into()] = true;
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

impl<'a, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> ShallowDecompressedTreeStore<'a, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into()].clone()
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
        let cs: Vec<_> = store
            .resolve(&a)
            .try_get_children()
            .map_or(vec![], |x| x.to_owned());
        // println!(
        //     "cs={:?}",
        //     cs.iter().collect::<Vec<_>>()
        // );
        if cs.len() == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs.len()];
        let mut c = *x - one(); // = self.first_descendant(x);
        let mut i = cs.len() - 1;
        let mut it = (0..cs.len()).rev();
        r[i] = c;
        while i > 0 {
            let y = it.next().unwrap();
            // println!(
            //     "i={:?} c={:?} size={:?} r={:?}", i, c.to_usize().unwrap(), size(store, &cs[y]),
            //     r.iter().map(|x| x.to_usize().unwrap()).collect::<Vec<_>>()
            // );
            i -= 1;
            c = c - cast(size(store, &cs[y])).unwrap();
            r[i] = c;
        }
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

impl<'d, IdC: Clone + Debug, IdD: PrimInt + Into<usize>> DecompressedTreeStore<'d, IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn descendants<'b, S>(&self, _store: &S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        (cast::<_, usize>(self.first_descendant(x)).unwrap()..cast::<_, usize>(*x).unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).into()]
    }

    fn descendants_count<'b, S>(&self, _store: &S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        cast::<_, usize>(*x - self.first_descendant(x) + one()).unwrap()
    }
}
