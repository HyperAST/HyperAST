use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use num_traits::{cast, PrimInt};

use crate::{tree::tree::{
    HashKind, LabelStore, Labeled, NodeStore, NodeStoreMut, Typed, WithChildren,
}, actions::{action_vec::{ApplicableActions, ActionsVec}, script_generator2::SimpleAction}};

pub(crate) struct SimpleTree<K> {
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

fn store(ls: &mut LS<u16>, ns: &mut NS<Tree>, node: &SimpleTree<u8>) -> u16 {
    let lid = node
        .label
        .as_ref()
        .map(|x| ls.get_or_insert(x.as_str().as_bytes()))
        .unwrap_or(0);
    let t = Tree {
        t: node.kind,
        label: lid,
        children: node.children.iter().map(|x| store(ls, ns, x)).collect(),
    };
    ns.get_or_insert(t)
}

pub(crate) fn vpair_to_stores<'a>(
    (src, dst): (SimpleTree<u8>, SimpleTree<u8>),
) -> (LS<u16>, NS<Tree>, u16, u16) {
    let (mut label_store, mut compressed_node_store) = make_stores();
    let src = store(&mut label_store, &mut compressed_node_store, &src);
    let dst = store(&mut label_store, &mut compressed_node_store, &dst);
    (label_store, compressed_node_store, src, dst)
}

pub(crate) struct DisplayTree<'a, 'b, I: num_traits::PrimInt, T: WithChildren> {
    ls: &'a LS<I>,
    ns: &'b NS<T>,
    node: u16,
    depth: usize,
}

impl<'a, 'b, I: num_traits::PrimInt, T: WithChildren> DisplayTree<'a, 'b, I, T> {
    pub fn new(ls: &'a LS<I>, ns: &'b NS<T>, node: u16) -> Self {
        Self {
            ls,
            ns,
            node,
            depth: 0,
        }
    }
}

impl<'a, 'b, I: num_traits::PrimInt, T> Display for DisplayTree<'a, 'b, I, T>
where
    T: Typed + WithChildren<TreeId = u16> + Labeled<Label = I> + Eq,
    T::Type: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cs = self.ns.resolve(&self.node);
        writeln!(
            f,
            "{}|-{}:{}",
            " ".repeat(self.depth),
            cs.get_type(),
            std::str::from_utf8(self.ls.resolve(cs.get_label())).unwrap()
        )?;
        let cs = cs.get_children().to_vec();
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
        Ok(())
    }
}
impl<'a, 'b, I: num_traits::PrimInt, T> Debug for DisplayTree<'a, 'b, I, T>
where
    T: Typed + WithChildren<TreeId = u16> + Labeled<Label = I> + Eq,
    T::Type: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cs = self.ns.resolve(&self.node);
        writeln!(
            f,
            "{}|-{}:{}    \t({})",
            " ".repeat(self.depth),
            cs.get_type(),
            std::str::from_utf8(self.ls.resolve(cs.get_label())).unwrap(),
            self.node,
        )?;
        let cs = cs.get_children().to_vec();
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
        Ok(())
    }
}

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
pub(crate) struct Tree {
    pub(crate) t: u8,
    pub(crate) label: u16,
    pub(crate) children: Vec<u16>,
}


impl<'a> ApplicableActions<'a, Tree> for ActionsVec<SimpleAction<Tree>> {
    fn build(
        t: <Tree as Typed>::Type,
        l: <Tree as Labeled>::Label,
        cs: Vec<<Tree as Stored>::TreeId>,
    ) -> Tree {
        Tree {
            t,
            label: l,
            children: cs,
        }
    }
}


impl crate::tree::tree::Typed for Tree {
    type Type = u8;

    fn get_type(&self) -> Self::Type {
        self.t
    }
}
impl crate::tree::tree::Labeled for Tree {
    type Label = u16;

    fn get_label(&self) -> &Self::Label {
        &self.label
    }
}
impl crate::tree::tree::Node for Tree {}
impl crate::tree::tree::Tree for Tree {
    fn has_children(&self) -> bool {
        self.children.len() > 0
    }

    fn has_label(&self) -> bool {
        self.label != 0
    }
}

impl crate::tree::tree::Stored for Tree {
    type TreeId = u16;
}

impl WithChildren for Tree {
    type ChildIdx = u8;

    fn child_count(&self) -> Self::ChildIdx {
        self.children.len() as u8
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.children[*idx as usize]
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.children[self.children.len() - (*idx as usize) - 1]
    }

    fn get_children(&self) -> &[Self::TreeId] {
        &self.children
    }
}

pub(crate) enum H {
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

impl crate::tree::tree::WithHashs for Tree {
    type HK = H;
    type HP = u8;
    fn hash(&self, _kind: &H) -> u8 {
        0
    }
}

pub(crate) struct NS<T: WithChildren> {
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

impl<'a, T: 'a + WithChildren + Eq> NodeStore<'a, T::TreeId, &'a T> for NS<T>
where
    T::TreeId: PrimInt,
{
    fn resolve(&'a self, id: &T::TreeId) -> &'a T {
        &self.v[cast::<T::TreeId, usize>(*id).unwrap()]
    }
}

impl<'a, T: 'a + WithChildren + Eq> NodeStoreMut<'a, T, &'a T> for NS<T> where T::TreeId: PrimInt {
    fn get_or_insert(&mut self, node: T) -> <T as super::tree::Stored>::TreeId {
        let p = self.v.iter().position(|x| {
            node.eq(x)
        });
        if let Some(p) = p {
            self.v[p] = node;
            cast::<usize, T::TreeId>(p).unwrap()
        } else {
            self.v.push(node);
            cast::<usize, T::TreeId>(self.v.len()-1).unwrap()
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
    // v: RefCell<Vec<crate::tree::tree::OwnedLabel>>,
    v: Vec<crate::tree::tree::OwnedLabel>,
    phantom: PhantomData<*const I>,
}

impl<'a, I: PrimInt> LabelStore<crate::tree::tree::SlicedLabel> for LS<I> {
    type I = I;
    fn get_or_insert<T: Borrow<crate::tree::tree::SlicedLabel>>(&mut self, node: T) -> Self::I {
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

    fn resolve(&self, id: &Self::I) -> &crate::tree::tree::SlicedLabel {
        &self.v[cast::<Self::I, usize>(*id).unwrap()]
    }
}

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
pub(crate) use tree;

use super::tree::Stored;