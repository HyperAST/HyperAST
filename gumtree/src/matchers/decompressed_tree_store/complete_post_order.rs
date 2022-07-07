use num_traits::{cast, one, zero, PrimInt};

use crate::tree::{
    tree::{NodeStore, Stored, Tree, WithChildren},
    tree_path::CompressedTreePath,
};

use super::{
    size, DecompressedTreeStore, DecompressedWithParent, Initializable, Iter, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<IdC, IdD: PrimInt + Into<usize>> {
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
    pub fn iter(
        &self,
    ) -> impl Iterator<Item=&IdC> {
        self.id_compressed
            .iter()
    }
}

impl<IdC: Clone, IdD: PrimInt + Into<usize>> DecompressedWithParent<IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        let r = self.id_parent[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.parent(id) != None
    }

    fn position_in_parent<T: WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        _store: &S,
        c: &IdD,
    ) -> T::ChildIdx {
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

impl<IdC: Clone, IdD: PrimInt + Into<usize>> PostOrder<IdC, IdD> for CompletePostOrder<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).into() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into() - 1].clone()
    }
}

impl<IdC: Clone, IdD: PrimInt + Into<usize>> PostOrderIterable<IdC, IdD>
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

impl<IdC: Clone, IdD: PrimInt + Into<usize>> PostOrderKeyRoots<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.into()]
    }
}
impl<IdC: Clone, IdD: PrimInt + Into<usize>> Initializable<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
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
                let x = store.resolve(&curr);
                let l = x.child_count();

                if l == zero() {
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
                } else if idx < l {
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

impl<IdC: Clone, IdD: PrimInt + Into<usize>> ShallowDecompressedTreeStore<IdC, IdD>
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
        cast::<_, IdD>(self.len()).unwrap() - one()
    }

    fn child<T: Stored<TreeId = IdC> + WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.resolve(&a).get_children().to_owned();
            if cs.len() > 0 {
                let mut z = 0;
                for x in cs[0..cast::<_, usize>(*d).unwrap() + 1].to_owned() {
                    z += size(store, &x);
                }
                r = self.first_descendant(&r) + cast(z).unwrap() - one();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<
        T: Stored<TreeId = IdC> + WithChildren,
        S: for<'a> NodeStore<'a, T::TreeId, &'a T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        let a = self.original(x);
        let cs: Vec<_> = store.resolve(&a).get_children().to_owned();
        if cs.len() == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs.len()];
        let mut c = *x - one(); // = self.first_descendant(x);
        let mut i = cs.len() - 1;
        let mut it = (0..cs.len()).rev();
        r[i] = c;
        while i > 0 {
            i -= 1;
            c = c - cast(size(store, &cs[it.next().unwrap()])).unwrap();
            r[i] = c;
        }
        r
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<Idx> {
        let mut idxs: Vec<Idx> = vec![];
        let mut curr = *descendant;
        loop {
            if let Some(p) = self.parent(&curr) {
                let idx = {
                    let lld: usize = cast(self.llds[cast::<_, usize>(p).unwrap()]).unwrap();
                    // TODO use other llds to skip nodes for count
                    cast(
                        self.id_parent[lld..cast(curr).unwrap()]
                            .iter()
                            .filter(|x| **x == p)
                            .count(),
                    )
                    .unwrap()
                };

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

impl<IdC: Clone, IdD: PrimInt + Into<usize>> DecompressedTreeStore<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn descendants<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        _store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        (cast::<_, usize>(self.first_descendant(x)).unwrap()..cast::<_, usize>(*x).unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).into()]
    }

    fn descendants_count<T: Tree<TreeId = IdC>, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(
        &self,
        _store: &S,
        x: &IdD,
    ) -> usize {
        cast::<_, usize>(*x - self.first_descendant(x) + one()).unwrap()
    }
}
