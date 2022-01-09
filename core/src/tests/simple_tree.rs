use std::{
    cell::{Ref, RefCell},
    marker::PhantomData, borrow::Borrow,
};

use num_traits::{cast, PrimInt};

use crate::tree::tree::{HashKind, LabelStore, Labeled, NodeStore, WithChildren, NodeStoreMut};

pub(crate) struct ST<K> {
    kind: K,
    label: Option<String>,
    children: Vec<ST<K>>,
}

impl<K> ST<K> {
    pub fn new(k: K) -> Self {
        Self {
            kind: k,
            label: None,
            children: vec![],
        }
    }
    pub fn new_l(k: K, l: &str) -> Self {
        Self {
            kind: k,
            label: Some(l.to_owned()),
            children: vec![],
        }
    }
    pub fn new_l_c(k: K, l: &str, c: Vec<ST<K>>) -> Self {
        Self {
            kind: k,
            label: Some(l.to_owned()),
            children: c,
        }
    }
    pub fn new_c(k: K, c: Vec<ST<K>>) -> Self {
        Self {
            kind: k,
            label: None,
            children: c,
        }
    }
}

fn store(ls: &mut LS<u16>, ns: &mut NS<Tree>, node: &ST<u8>) -> u16 {
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

pub(crate) fn vpair_to_stores<'a>((src, dst): (ST<u8>, ST<u8>)) -> (LS<u16>, NS<Tree>, u16, u16) {
    let (mut label_store, mut compressed_node_store) = make_stores();
    let src = store(&mut label_store, &mut compressed_node_store, &src);
    let dst = store(&mut label_store, &mut compressed_node_store, &dst);
    (label_store, compressed_node_store, src, dst)
}

fn make_stores<'a>() -> (LS<u16>, NS<Tree>) {
    let label_store = LS::<u16> {
        // v: RefCell::new(vec![b"".to_vec()]),
        v: Default::default(),
        phantom: PhantomData,
    };
    let compressed_node_store = NS::<Tree> {
        v: vec![],
    };
    (label_store, compressed_node_store)
}

#[derive(PartialEq, Eq)]
pub(crate) struct Tree {
    pub(crate) t: u8,
    pub(crate) label: u16,
    pub(crate) children: Vec<u16>,
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

type A = Tree;
impl WithChildren for Tree {
    type ChildIdx = u8;

    fn child_count(&self) -> Self::ChildIdx {
        self.children.len() as u8
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.children[*idx as usize]
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

impl<T: WithChildren + Labeled> NS<T>
where
    T::Label: PrimInt,
{
    pub(crate) fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        ls: &LS<T::Label>,
    ) -> std::fmt::Result {
        self.v.iter().enumerate().for_each(|(i, x)| {
            write!(
                f,
                "[{}]: {}\n",
                i,
                std::str::from_utf8(&ls.resolve(&x.get_label())).unwrap()
            )
            .unwrap()
        });
        write!(f, "")
        // f.debug_struct("NS").field("v", &self.v).finish()
    }
}

impl<'a, T: 'a + WithChildren + Eq> NodeStore<'a, T::TreeId, &'a T> for NS<T>
where
    T::TreeId: PrimInt,
{

    fn resolve(&'a self, id: &T::TreeId) -> &'a T {
        &self.v[cast::<T::TreeId, usize>(*id).unwrap()]
    }
}

impl<'a, T: 'a + WithChildren + Eq> NodeStoreMut<'a, T, &'a T> for NS<T>
where
    T::TreeId: PrimInt,
{
}

impl<'a, T: 'a + WithChildren + Eq> NS<T>
where
    T::TreeId: PrimInt,
{
    fn get_or_insert(&mut self, node: T) -> T::TreeId {
        if let Some(i) = self.v
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
        // let a:Ref<'a,_> = self.v.borrow();
        // Ref::map(a, |x| {
        //     &x[cast::<Self::I, usize>(*id).unwrap()]
        // }).deref()
    }
}

// pub(crate) fn get_child<A: ZsStore<u16, u16> + DecompressedTreeStore<u16, u16>>(
//     store: &NS<Tree>,
//     arena: &A,
//     x: u16,
//     p: &[u16],
// ) -> u16 {
//     let mut r = x;
//     for d in p {
//         let a = arena.original(r);
//         let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
//         if cs.len() > 0 {
//             let mut z = 0;
//             for x in cs[0..(*d as usize) + 1].to_owned() {
//                 z += size(store, x);
//             }
//             r = arena.lld(r) + z - 1;
//         } else {
//             panic!("no children in this tree")
//         }
//     }
//     r
// }

// pub(crate) fn get_child2<A: BreathFirstContigousSiblings<u16, u16>>(
//     store: &NS<Tree>,
//     arena: &A,
//     x: u16,
//     p: &[u16],
// ) -> u16 {
//     let mut r = x;
//     for d in p {
//         let a = arena.original(r);
//         let cs: Vec<_> = store.get_node_at_id(&a).get_children().to_owned();
//         if cs.len() > 0 {
//             r = arena.first_child(r).unwrap() + *d;
//         } else {
//             panic!("no children in this tree")
//         }
//     }
//     r
// }
