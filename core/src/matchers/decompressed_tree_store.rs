use num_traits::{cast, one, zero, PrimInt};

use crate::{actions::script_generator::CompressedTreePath, tree::tree::{HashKind, NodeStore, Tree, WithChildren, WithHashs}};

pub trait Initializable<IdC: PrimInt, IdD: PrimInt> {
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: NodeStore<T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self;
}

pub trait ShallowDecompressedTreeStore<IdC: PrimInt, IdD: PrimInt>: Initializable<IdC, IdD> {
    fn len(&self) -> usize;
    fn node_count(&self) -> IdD {
        cast(self.len()).unwrap()
    }
    fn original(&self, id: &IdD) -> IdC;
    fn leaf_count(&self) -> IdD;
    fn root(&self) -> IdD;
    fn path(&self,parent:&IdD,descendant:&IdD) -> CompressedTreePath<u32>;
    fn child<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD;
    // fn child_count<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
    //     &self,
    //     store: &S,
    //     x: &IdD,
    // ) -> IdD;
    fn children<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD>;
}

pub trait DecompressedTreeStore<IdC: PrimInt, IdD: PrimInt>: ShallowDecompressedTreeStore<IdC, IdD> {
    fn descendants<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD>;
    fn descendants_count<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize;
    fn first_descendant(&self, i: &IdD) -> IdD;
}

pub trait DecompressedWithParent<IdD: PrimInt> {
    fn has_parent(&self, id: &IdD) -> bool;
    fn parent(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<T: Tree, S: NodeStore<T>>(&self, store: &S, c: &IdD) -> T::ChildIdx;
}

pub trait DecompressedWithSiblings<IdD: PrimInt> {
    fn siblings_count(&self, id: &IdD) -> Option<IdD>;
    fn position_in_parent<T: Tree, S: NodeStore<T>>(&self, store: &S, c: &IdD) -> T::ChildIdx;
}

pub trait BreathFirstContigousSiblings<IdC: PrimInt, IdD: PrimInt>:
    DecompressedTreeStore<IdC, IdD> + DecompressedWithParent<IdD>
{
    fn has_children(&self, id: &IdD) -> bool;
    fn first_child(&self, id: &IdD) -> Option<IdD>;
}

pub trait PostOrder<IdC: PrimInt, IdD: PrimInt>: DecompressedTreeStore<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD;
    fn tree(&self, id: &IdD) -> IdC;
}

/// vec of decompressed nodes layed out in pre order with contigous siblings
pub struct BreathFirst<IdC: PrimInt, IdD: PrimInt> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    id_first_child: Vec<IdD>,
}

impl<IdC: PrimInt, IdD: PrimInt> BreathFirstContigousSiblings<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn has_children(&self, id: &IdD) -> bool {
        self.first_child(id) != None
    }

    fn first_child(&self, id: &IdD) -> Option<IdD> {
        let r = self.id_first_child[id.to_usize().unwrap()];
        if r == num_traits::zero() {
            None
        } else {
            Some(r)
        }
    }
}

impl<IdC: PrimInt, IdD: PrimInt> DecompressedWithParent<IdD> for BreathFirst<IdC, IdD> {
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

    fn position_in_parent<T: Tree, S: NodeStore<T>>(&self, store: &S, c: &IdD) -> T::ChildIdx {
        let p = self.parent(c).unwrap();
        cast(*c - self.first_child(&p).unwrap()).unwrap()
    }
}

impl<IdC: PrimInt, IdD: PrimInt> Initializable<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn new<
        T: Tree<TreeId = IdC>,
        // HK: HashKind, HP: PrimInt,
        S: NodeStore<T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
        let mut leaf_count = zero();
        let mut id_compressed: Vec<IdC> = vec![*root];
        let mut id_parent: Vec<IdD> = vec![num_traits::zero()];
        let mut id_first_child: Vec<IdD> = vec![];
        let mut i: usize = 0;

        while i < id_compressed.len() {
            let node = store.get_node_at_id(&id_compressed[i]);
            let l = node.get_children();
            id_first_child.push(if l.len() > 0 {
                cast(id_compressed.len()).unwrap()
            } else {
                num_traits::zero()
            });
            if l.len() == 0 {
                leaf_count = leaf_count + one();
            }
            id_parent.extend(l.iter().map(|_| cast::<usize, IdD>(i).unwrap()));
            id_compressed.extend_from_slice(l);

            i += 1;
        }

        BreathFirst {
            leaf_count,
            id_compressed,
            id_parent,
            id_first_child,
        }
    }
}

impl<IdC: PrimInt, IdD: PrimInt> ShallowDecompressedTreeStore<IdC, IdD> for BreathFirst<IdC, IdD> {
    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[id.to_usize().unwrap()]
    }

    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn leaf_count(&self) -> IdD {
        self.leaf_count
    }

    fn root(&self) -> IdD {
        zero()
    }

    fn child<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
            if cs.len() > 0 {
                r = self.first_child(&r).unwrap() + cast(*d).unwrap();
            } else {
                panic!("no children in this tree")
            }
        }
        r
    }

    fn children<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        let node = store.get_node_at_id(&self.original(x));
        let l: usize = cast(node.child_count()).unwrap();
        let s: usize = cast(*x).unwrap();
        let r = s + 1..s + l;
        r.map(|x| cast::<usize, IdD>(x).unwrap())
            .collect::<Vec<_>>()
            .to_owned()
    }


    fn path(&self,parent:&IdD,descendant:&IdD) -> CompressedTreePath<u32> {
        todo!()
    }
}

impl<IdC: PrimInt, IdD: PrimInt> DecompressedTreeStore<IdC, IdD> for BreathFirst<IdC, IdD> {

    fn descendants<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        // todo possible opti by also making descendants contigous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let node = store.get_node_at_id(&self.original(&id[i]));
            let l: usize = cast(node.child_count()).unwrap();
            let s: usize = cast(id[i]).unwrap();
            let r = s + 1..s + l;
            id.extend(r.map(|x| cast::<usize, IdD>(x).unwrap()));
            i += 1;
        }
        id
    }

    fn descendants_count<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize {
        // todo possible opti by also making descendants contigous in arena
        let mut id: Vec<IdD> = vec![*x];
        let mut i: usize = cast(*x).unwrap();

        while i < id.len() {
            let node = store.get_node_at_id(&self.original(&id[i]));
            let l: usize = cast(node.child_count()).unwrap();
            let s: usize = cast(id[i]).unwrap();
            let r = s + 1..s + l;
            id.extend(r.map(|x| cast::<usize, IdD>(x).unwrap()));
            i += 1;
        }
        id.len()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        todo!()
    }
}

pub trait PostOrderKeyRoots<IdC: PrimInt, IdD: PrimInt + Into<usize>>: PostOrder<IdC, IdD> {
    fn kr(&self, x: IdD) -> IdD;
}

pub struct SimpleZsTree<IdC: PrimInt, IdD: PrimInt + Into<usize>> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)= l(k’)}.
    kr: Vec<IdD>,
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> PostOrder<IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).into() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into() - 1]
    }
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> PostOrderKeyRoots<IdC, IdD>
    for SimpleZsTree<IdC, IdD>
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.into()]
    }
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> Initializable<IdC, IdD> for SimpleZsTree<IdC, IdD> {
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: NodeStore<T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
        struct R<IdC, Idx, IdD> {
            curr: IdC,
            idx: Idx,
            lld: IdD,
        }

        let mut leaf_count = 0;
        let mut stack = vec![R {
            curr: *root,
            idx: zero(),
            lld: zero(),
        }];
        let mut llds: Vec<IdD> = vec![];
        let mut id_compressed = vec![];
        loop {
            if let Some(R { curr, idx, lld }) = stack.pop() {
                let x = store.get_node_at_id(&curr);
                let l = x.child_count();

                if l == zero() {
                    // leaf
                    let lld = cast(id_compressed.len()).unwrap();
                    if let Some(tmp) = stack.last_mut() {
                        if tmp.idx == one() {
                            tmp.lld = lld;
                        }
                    }
                    llds.push(lld);
                    id_compressed.push(curr);
                    leaf_count += 1;
                } else if idx < l {
                    //
                    let child = x.get_child(&idx);
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
                    id_compressed.push(curr);
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
        }
    }
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ShallowDecompressedTreeStore<IdC, IdD>
    for SimpleZsTree<IdC, IdD>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into()]
    }

    fn leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
    }

    fn root(&self) -> IdD {
        self.node_count() - one() // todo test changing it
    }

    fn child<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
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

    fn children<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        let a = self.original(x);
        let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
        let mut r = vec![];
        let mut c = self.lld(x);
        for x in cs.to_owned() {
            c = c + cast(size(store, &x)).unwrap() - one();
            r.push(c);
        }
        r
    }

    fn path(&self,parent:&IdD,descendant:&IdD) -> CompressedTreePath<u32> {
        todo!()
    }
}
impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> DecompressedTreeStore<IdC, IdD>
    for SimpleZsTree<IdC, IdD>
{

    fn descendants<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        (cast::<_, usize>(self.lld(x)).unwrap()..cast::<_, usize>(*x).unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).into()]
    }

    fn descendants_count<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize {
        cast::<_, usize>(self.lld(x) - *x).unwrap()
    }
}

fn size<T: WithChildren, NS: NodeStore<T>>(store: &NS, x: &T::TreeId) -> usize {
    let cs = store.get_node_at_id(&x).get_children().to_owned();

    let mut z = 0;
    for x in cs {
        z += size(store, &x);
    }
    z + 1
}

pub struct CompletePostOrder<IdC: PrimInt, IdD: PrimInt + Into<usize>> {
    leaf_count: IdD,
    id_compressed: Vec<IdC>,
    id_parent: Vec<IdD>,
    pub(crate) llds: Vec<IdD>,
    /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)= l(k’)}.
    kr: Vec<IdD>,
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> DecompressedWithParent<IdD>
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

    fn position_in_parent<T: Tree, S: NodeStore<T>>(&self, store: &S, c: &IdD) -> T::ChildIdx {
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

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> PostOrder<IdC, IdD> for CompletePostOrder<IdC, IdD> {
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[(*i).into() - 1] + num_traits::one()
    }

    fn tree(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into() - 1]
    }
}

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> PostOrderKeyRoots<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn kr(&self, x: IdD) -> IdD {
        self.kr[x.into()]
    }
}
impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> Initializable<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: NodeStore<T>,
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
            curr: *root,
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
                let x = store.get_node_at_id(&curr);
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

impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ShallowDecompressedTreeStore<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        self.id_compressed[(*id).into()]
    }

    fn leaf_count(&self) -> IdD {
        cast(self.kr.len()).unwrap()
    }

    fn root(&self) -> IdD {
        self.node_count() - one()
    }

    fn child<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        let mut r = *x;
        for d in p {
            let a = self.original(&r);
            let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
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

    fn children<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        let a = self.original(x);
        let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
        if cs.len() == 0 {
            return vec![];
        }
        let mut r = vec![zero(); cs.len()];
        let mut c = *x - one(); // = self.first_desendant(x);
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

    fn path(&self,parent:&IdD,descendant:&IdD) -> CompressedTreePath<u32> {
        todo!()
    }
}

    impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> DecompressedTreeStore<IdC, IdD>
    for CompletePostOrder<IdC, IdD>
{
    fn descendants<T: Tree<TreeId = IdC>, S: NodeStore<T>>(&self, store: &S, x: &IdD) -> Vec<IdD> {
        (cast::<_, usize>(self.first_descendant(x)).unwrap()..cast::<_, usize>(*x).unwrap())
            .map(|x| cast(x).unwrap())
            .collect()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.llds[(*i).into()]
    }

    fn descendants_count<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize {
        cast::<_, usize>(*x - self.first_descendant(x) + one()).unwrap()
    }
}
