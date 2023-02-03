use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use num_traits::{cast, NumCast, PrimInt, ToPrimitive};

use hyper_ast::types::{
    HashKind, LabelStore, Labeled, MySlice, NodeStore, NodeStoreMut, Stored, Typed, WithChildren,
    WithStats,
};

#[allow(dead_code)]
pub struct SimpleTree<K> {
    kind: K,
    label: Option<String>,
    children: Vec<SimpleTree<K>>,
}

impl<K> SimpleTree<K> {
    pub fn new(k: K, l: Option<&str>, c: Vec<SimpleTree<K>>) -> Self {
        Self {
            kind: k,
            label: l.map(|s| s.to_owned()),
            children: c,
        }
    }
}

#[cfg(test)]
fn store<'a>(ls: &mut LS<u16>, ns: &mut NS<Tree>, node: &SimpleTree<u8>) -> u16 {
    fn store_aux<'a>(ls: &mut LS<u16>, ns: &mut NS<Tree>, node: &SimpleTree<u8>) -> Tree {
        let lid = node
            .label
            .as_ref()
            .map(|x| ls.get_or_insert(x.as_str()))
            .unwrap_or(0);
        let mut size = 1;
        let mut height = 0;
        let children = node
            .children
            .iter()
            .map(|x| {
                let t = store_aux(ls, ns, x);
                size += t.size;
                height = height.max(t.height);
                ns.get_or_insert(t)
            })
            .collect();
        height += 1;
        Tree {
            t: node.kind,
            label: lid,
            children,
            size,
            height,
        }
    }
    let t = store_aux(ls, ns, node);
    ns.get_or_insert(t)
}

#[cfg(test)]
pub(crate) fn vpair_to_stores<'a>(
    (src, dst): (SimpleTree<u8>, SimpleTree<u8>),
) -> (LS<u16>, NS<Tree>, u16, u16) {
    let (mut label_store, mut compressed_node_store) = make_stores();
    let src = store(&mut label_store, &mut compressed_node_store, &src);
    let dst = store(&mut label_store, &mut compressed_node_store, &dst);
    (label_store, compressed_node_store, src, dst)
}

impl AsRef<Tree> for &Tree {
    fn as_ref(&self) -> &Tree {
        self
    }
}

pub(crate) struct DisplayTree<'a, 'b, I: num_traits::PrimInt, T: WithChildren> {
    ls: &'a LS<I>,
    ns: &'b NS<T>,
    node: u16,
    depth: usize,
}
impl<'a, 'b> DisplayTree<'a, 'b, u16, Tree> {
    #[allow(dead_code)]
    pub fn new(ls: &'a LS<u16>, ns: &'b NS<Tree>, node: u16) -> Self {
        Self {
            ls,
            ns,
            node,
            depth: 0,
        }
    }
}

impl<'a, 'b> Display for DisplayTree<'a, 'b, u16, Tree>
where
// T: 'a,// + AsTreeRef<TreeRef<'b, T>>,
// T: Typed + WithChildren<TreeId = u16> + Labeled<Label = I> + Eq,
// T::Type: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cs = self.ns.resolve(&self.node);
        writeln!(
            f,
            "{}|-{}:{} \ts{}\th{}",
            " ".repeat(self.depth),
            cs.get_type(),
            self.ls.resolve(cs.get_label()),
            cs.size(),
            cs.height(),
        )?;
        if let Some(cs) = cs.children() {
            let cs: Vec<_> = cs.into();
            for n in cs {
                Display::fmt(
                    &Self {
                        ls: self.ls,
                        ns: self.ns,
                        node: n,
                        depth: self.depth + 1,
                    },
                    f,
                )?;
            }
        }
        Ok(())
    }
}

impl<'a, 'b> Debug for DisplayTree<'a, 'b, u16, Tree> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cs = self.ns.resolve(&self.node);
        writeln!(
            f,
            "{}|-{}:{}    \t({})\ts{}\th{}",
            " ".repeat(self.depth),
            cs.get_type(),
            self.ls.resolve(cs.get_label()),
            self.node,
            cs.size(),
            cs.height(),
        )?;
        if let Some(cs) = cs.children() {
            let cs: Vec<_> = cs.into();
            for n in cs {
                Debug::fmt(
                    &Self {
                        ls: self.ls,
                        ns: self.ns,
                        node: n,
                        depth: self.depth + 1,
                    },
                    f,
                )?;
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn make_stores<'a>() -> (LS<u16>, NS<Tree>) {
    let label_store = LS::<u16> {
        // v: RefCell::new(vec![b"".to_vec()]),
        v: Default::default(),
        phantom: PhantomData,
    };
    let compressed_node_store = NS::<Tree> { v: vec![] };
    (label_store, compressed_node_store)
}

#[derive(PartialEq, Eq)]
pub struct Tree {
    pub(crate) t: u8,
    pub(crate) label: u16,
    pub(crate) children: Vec<u16>,
    pub(crate) size: u16,
    pub(crate) height: u16,
}

// impl<'a> ApplicableActions<Tree,TreeRef<'a,Tree>> for ActionsVec<SimpleAction<Tree>> {
//     fn build(
//         t: <Tree as Typed>::Type,
//         l: <Tree as Labeled>::Label,
//         cs: Vec<<Tree as Stored>::TreeId>,
//     ) -> Tree {
//         Tree {
//             t,
//             label: l,
//             children: cs,
//         }
//     }
// }
// impl<'a> hyper_ast::types::NodeStoreExt<'a, Tree, TreeRef<'a, Tree>> for NS<Tree> {
//     fn build_then_insert(
//         &mut self,
//         t: <TreeRef<'a, Tree> as hyper_ast::types::Typed>::Type,
//         l: <TreeRef<'a, Tree> as hyper_ast::types::Labeled>::Label,
//         cs: Vec<<Tree as Stored>::TreeId>,
//     ) -> <Tree as Stored>::TreeId {
//         let node = Tree {
//             t,
//             label: l,
//             children: cs,
//         };
//         self.get_or_insert(node)
//     }
// }

impl hyper_ast::types::NodeStoreExt<Tree> for NS<Tree> {
    fn build_then_insert(
        &mut self,
        _i: <Tree as hyper_ast::types::Stored>::TreeId,
        t: <Tree as hyper_ast::types::Typed>::Type,
        _l: Option<<Tree as hyper_ast::types::Labeled>::Label>,
        cs: Vec<<Tree as Stored>::TreeId>,
    ) -> <Tree as Stored>::TreeId {
        let node = Tree {
            t,
            label: 0,
            children: cs,
            size: 0,
            height: 0,
        };
        self.get_or_insert(node)
    }
}

impl hyper_ast::types::Typed for Tree {
    type Type = u8;

    fn get_type(&self) -> Self::Type {
        self.t
    }
}

impl hyper_ast::types::WithSerialization for Tree {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}

impl<T> hyper_ast::types::WithSerialization for TreeRef<'_, T> {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}
impl<T> Clone for TreeRef<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: hyper_ast::types::Typed> hyper_ast::types::Typed for TreeRef<'_, T> {
    type Type = T::Type;

    fn get_type(&self) -> Self::Type {
        self.0.get_type()
    }
}
impl hyper_ast::types::Labeled for Tree {
    type Label = u16;

    fn get_label(&self) -> &Self::Label {
        &self.label
    }
}
impl<T: hyper_ast::types::Labeled> hyper_ast::types::Labeled for TreeRef<'_, T> {
    type Label = T::Label;

    fn get_label(&self) -> &Self::Label {
        self.0.get_label()
    }
}
impl hyper_ast::types::Node for Tree {}
impl<T: hyper_ast::types::Node> hyper_ast::types::Node for TreeRef<'_, T> {}
impl hyper_ast::types::Tree for Tree {
    fn has_children(&self) -> bool {
        self.children.len() > 0
    }

    fn has_label(&self) -> bool {
        self.label != 0
    }
}

impl<T: hyper_ast::types::Tree> hyper_ast::types::Tree for TreeRef<'_, T>
where
    T::TreeId: Clone,
{
    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_label(&self) -> bool {
        self.0.has_label()
    }
}

impl hyper_ast::types::Stored for Tree {
    type TreeId = u16;
}
impl<T: hyper_ast::types::Stored> hyper_ast::types::Stored for TreeRef<'_, T> {
    type TreeId = T::TreeId;
}

impl WithChildren for Tree {
    type ChildIdx = u8;

    type Children<'a> = MySlice<Self::TreeId>;

    fn child_count(&self) -> Self::ChildIdx {
        self.children.len() as u8
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.children.get(idx.to_usize().unwrap()).map(|x| *x)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        let idx = num_traits::CheckedSub::checked_sub(&self.child_count(), &(*idx + 1))?;
        self.children.get(idx.to_usize().unwrap()).copied()
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        Some(self.children.as_slice().into())
    }
}

impl<T: WithChildren> WithChildren for TreeRef<'_, T>
where
    T::TreeId: Clone,
{
    type ChildIdx = T::ChildIdx;

    type Children<'a> = T::Children<'a> where Self: 'a;

    fn child_count(&self) -> Self::ChildIdx {
        self.0.child_count()
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.0.child(idx)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.0.child_rev(idx)
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        self.0.children()
    }
}

impl WithStats for Tree {
    fn size(&self) -> usize {
        self.size.to_usize().unwrap()
    }

    fn height(&self) -> usize {
        self.height.to_usize().unwrap()
    }
}

impl<T: Stored + WithStats> WithStats for TreeRef<'_, T>
where
    T::TreeId: Clone,
{
    fn size(&self) -> usize {
        self.0.size()
    }

    fn height(&self) -> usize {
        self.0.height()
    }
}

pub enum H {
    S,
    L,
}

impl HashKind for H {
    fn structural() -> Self {
        H::S
    }

    fn label() -> Self {
        H::L
    }
}

impl hyper_ast::types::WithHashs for Tree {
    type HK = H;
    type HP = u8;
    fn hash(&self, _kind: &H) -> u8 {
        0
    }
}

impl<T: hyper_ast::types::WithHashs> hyper_ast::types::WithHashs for TreeRef<'_, T> {
    type HK = T::HK;
    type HP = T::HP;
    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.0.hash(kind)
    }
}

pub(crate) struct NS<T> {
    v: Vec<T>,
}

// impl<T: WithChildren + Labeled> NS<T>
// where
//     T::Label: PrimInt,
// {
//     pub(crate) fn fmt(
//         &self,
//         f: &mut std::fmt::Formatter<'_>,
//         ls: &LS<T::Label>,
//     ) -> std::fmt::Result {
//         self.v.iter().enumerate().for_each(|(i, x)| {
//             write!(
//                 f,
//                 "[{}]: {}\n",
//                 i,
//                 std::str::from_utf8(&ls.resolve(&x.get_label())).unwrap()
//             )
//             .unwrap()
//         });
//         write!(f, "")
//         // f.debug_struct("NS").field("v", &self.v).finish()
//     }
// }

// impl<'a, T: AsTreeRef<R>, R: 'a + WithChildren + Eq> NodeStore<'a, R::TreeId, R> for NS<T>
// where
//     R::TreeId: PrimInt,
// {
//     fn resolve(&self, id: &R::TreeId) -> R {
//         self.v[cast::<R::TreeId, usize>(*id).unwrap()].as_tree_ref()
//     }
// }

// impl<'a, T:for<'b> AsTreeRef2<'b, R>, R: 'a + WithChildren + Eq> NodeStore2<R::TreeId, T, R> for NS<T>
// where
//     R::TreeId: PrimInt,
// {
//     fn resolve(&self, id: R::TreeId) -> R {
//         self.v[cast::<R::TreeId, usize>(id).unwrap()].as_tree_ref()
//     }
// }

// impl NodeStore2<u16> for NS<Tree> {
//     type R<'a> = TreeRef<'a, Tree>;
//     fn resolve(&self, id: &u16) -> TreeRef<'_, Tree> {
//         TreeRef(&self.v[id.to_usize().unwrap()])
//     }
// }

// impl<'a, T:'a+ hyper_ast::types::Tree> NodeStore2<'a, T::TreeId> for NS<T>
// where T::TreeId : ToPrimitive {
//     type R = TreeRef<'a, T>;

//     fn resolve(&self, id: &T::TreeId) -> TreeRef<'_, T> {
//         TreeRef(&self.v[id.to_usize().unwrap()])
//     }
// }

impl<T: hyper_ast::types::Tree> NodeStore<T::TreeId> for NS<T>
where
    T::TreeId: ToPrimitive,
{
    type R<'a>  = TreeRef<'a, T> where T: 'a;

    fn resolve(&self, id: &T::TreeId) -> TreeRef<'_, T> {
        TreeRef(&self.v[id.to_usize().unwrap()])
    }
}

// impl<T: hyper_ast::types::Tree> NodeStore3<T::TreeId> for NS<T>
// where T::TreeId : ToPrimitive {
//     type R = dyn for<'any> GenericItem<'any, Item = TreeRef<'any, T>>;

//     fn resolve(&self, id: &T::TreeId) -> TreeRef<'_, T> {
//         TreeRef(&self.v[id.to_usize().unwrap()])
//     }
// }

#[derive(PartialEq, Eq)]
pub(crate) struct TreeRef<'a, T>(&'a T);

// impl<'a> AsTreeRef<TreeRef<'a, Tree>> for Tree {
//     fn as_tree_ref(&self) -> TreeRef<'a,Tree> {
//         TreeRef(self)
//     }
// }
// impl<'a, T> AsTreeRef<TreeRef<'a, T>> for TreeRef<'a, T> {
//     fn as_tree_ref(&self) -> TreeRef<T> {
//         TreeRef(&self.0)
//     }
// }

// impl<'a> AsTreeRef<TreeRef<'a, SimpleTree<u8>>> for SimpleTree<u8> {
//     fn as_tree_ref(&self) -> TreeRef<SimpleTree<u8>> {
//         TreeRef(self)
//     }
// }

// impl<'a, T: AsTreeRef<R> + WithChildren + Eq, R: 'a + WithChildren<TreeId = T::TreeId> + Eq>
//     NodeStoreMut<'a, T, R> for NS<T>
// where
//     <T as Stored>::TreeId: PrimInt,
// {
//     fn get_or_insert(&mut self, node: T) -> <T as Stored>::TreeId {
//         let p = self.v.iter().position(|x| node.eq(x));
//         if let Some(p) = p {
//             self.v[p] = node;
//             cast::<usize, R::TreeId>(p).unwrap()
//         } else {
//             self.v.push(node);
//             cast::<usize, R::TreeId>(self.v.len() - 1).unwrap()
//         }
//     }
// }

// impl NodeStoreMut2<Tree> for NS<Tree> {
//     fn get_or_insert(&mut self, node: Tree) -> u16 {
//         let p = self.v.iter().position(|x| node.eq(x));
//         if let Some(p) = p {
//             self.v[p] = node;
//             cast::<usize, u16>(p).unwrap()
//         } else {
//             self.v.push(node);
//             cast::<usize, u16>(self.v.len() - 1).unwrap()
//         }
//     }
// }

impl<T: hyper_ast::types::Tree + Eq> NodeStoreMut<T> for NS<T>
where
    T::TreeId: ToPrimitive + NumCast,
{
    fn get_or_insert(&mut self, node: T) -> T::TreeId {
        let p = self.v.iter().position(|x| node.eq(x));
        if let Some(p) = p {
            self.v[p] = node;
            cast::<usize, T::TreeId>(p).unwrap()
        } else {
            self.v.push(node);
            cast::<usize, T::TreeId>(self.v.len() - 1).unwrap()
        }
    }
}

impl<'a, T: 'a + WithChildren + Eq> NS<T>
where
    T::TreeId: PrimInt,
{
    fn get_or_insert(&mut self, node: T) -> T::TreeId {
        if let Some(i) = self
            .v
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x == &node { Some(i) } else { None })
        {
            cast(i).unwrap()
        } else {
            let l = self.v.len();
            self.v.push(node);
            cast(l).unwrap()
        }
    }
}

// pub(crate) struct NS<T: WithChildren> {
//     v: Vec<Rc<T>>,
// }

// impl<T: WithChildren + Eq> NS<T> where T::TreeId:PrimInt {
//     fn get_or_insert(&mut self, node: T) -> T::TreeId {
//         let mut a = self.v;
//         if let Some(i) = a
//             .iter()
//             .enumerate()
//             .find_map(|(i, x)| if x.as_ref() == &node { Some(i) } else { None })
//         {
//             cast(i).unwrap()
//         } else {
//             let l = a.len();
//             a.push(Rc::new(node));
//             cast(l).unwrap()
//         }
//     }

//     fn resolve<'b>(&'b self, id: &T::TreeId) -> &'b T {
//         // Ref::map((&self.v).borrow(), |x| {
//         //     &x[cast::<T::TreeId, usize>(*id).unwrap()]
//         // })
//         &self.v[cast::<T::TreeId, usize>(*id).unwrap()]
//     }
// }

pub(crate) struct LS<I: PrimInt> {
    // v: RefCell<Vec<hyper_ast::types::OwnedLabel>>,
    v: Vec<hyper_ast::types::OwnedLabel>,
    phantom: PhantomData<*const I>,
}

impl<'a, I: PrimInt> LabelStore<hyper_ast::types::SlicedLabel> for LS<I> {
    type I = I;
    fn get_or_insert<T: Borrow<hyper_ast::types::SlicedLabel>>(&mut self, node: T) -> Self::I {
        let a = &mut self.v;
        let b = a
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x.eq(node.borrow()) { Some(i) } else { None })
            .to_owned();
        if let Some(i) = b {
            cast(i).unwrap()
        } else {
            let l = a.len();
            a.push(node.borrow().to_owned());
            cast(l).unwrap()
        }
    }

    fn get<T: Borrow<hyper_ast::types::SlicedLabel>>(&self, node: T) -> Option<Self::I> {
        let a = &self.v;
        let b = a
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x.eq(node.borrow()) { Some(i) } else { None })
            .to_owned();
        b.map(|i| cast(i).unwrap())
    }

    fn resolve(&self, id: &Self::I) -> &hyper_ast::types::SlicedLabel {
        &self.v[cast::<Self::I, usize>(*id).unwrap()]
    }
}

#[allow(unused_macros)]
macro_rules! tree {
    ( $k:expr ) => {
        SimpleTree::new($k, None, vec![])
    };
    ( $k:expr, $l:expr) => {
        SimpleTree::new($k, Some($l), vec![])
    };
    ( $k:expr, $l:expr; [$($x:expr),+ $(,)?]) => {
        SimpleTree::new($k, Some($l), vec![$($x),+])
    };
    ( $k:expr; [$($x:expr),+ $(,)?]) => {
        SimpleTree::new($k, None, vec![$($x),+])
    };
}

#[allow(unused_imports)]
pub(crate) use tree;
