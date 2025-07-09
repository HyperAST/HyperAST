use crate::types::{
    HashKind, HyperType, LabelStore, Labeled, NodeId, NodeStore, NodeStoreMut, Stored, Typed,
    WithChildren, WithStats,
};
use crate::{store::nodes::compo, types};
use num::{NumCast, PrimInt, ToPrimitive, cast};
use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub struct SimpleTree<K, DD = ()> {
    kind: K,
    label: Option<String>,
    children: Vec<SimpleTree<K, DD>>,
    derived_data: DD,
}

impl<K, DD: Default> SimpleTree<K, DD> {
    pub fn new(k: K, l: Option<&str>, c: Vec<SimpleTree<K, DD>>) -> Self {
        Self {
            kind: k,
            label: l.map(|s| s.to_owned()),
            children: c,
            derived_data: Default::default(),
        }
    }
}

impl<K, DD> SimpleTree<K, DD> {
    pub fn with_dd(k: K, l: Option<&str>, c: Vec<SimpleTree<K, DD>>, derived_data: DD) -> Self {
        Self {
            kind: k,
            label: l.map(|s| s.to_owned()),
            children: c,
            derived_data,
        }
    }
}

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

use crate::store::SimpleStores;
pub fn vpair_to_stores<'a>(
    (src, dst): (SimpleTree<u8>, SimpleTree<u8>),
) -> (SimpleStores<TStore, NS<Tree>, LS<u16>>, u16, u16) {
    let (mut label_store, mut compressed_node_store) = make_stores();
    let src = store(&mut label_store, &mut compressed_node_store, &src);
    let dst = store(&mut label_store, &mut compressed_node_store, &dst);
    let stores = SimpleStores {
        type_store: std::marker::PhantomData::<TStore>,
        node_store: compressed_node_store,
        label_store,
    };
    (stores, src, dst)
}

impl AsRef<Tree> for &Tree {
    fn as_ref(&self) -> &Tree {
        self
    }
}

pub struct DisplayTree<'a, 'b, I: num::PrimInt, T: WithChildren> {
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

impl<'a, 'b> Display for DisplayTree<'a, 'b, u16, Tree> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cs = self.ns.resolve(&self.node);
        writeln!(
            f,
            "{}|-{}:{} \ts{}\th{}",
            " ".repeat(self.depth),
            cs.get_type(),
            self.ls.resolve(cs.get_label_unchecked()),
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
        write!(f, "{}|-{}", " ".repeat(self.depth), cs.get_type(),)?;
        if let Some(label) = cs.try_get_label() {
            write!(f, ":{}", self.ls.resolve(label),)?;
        }
        writeln!(f, "    \t({})\ts{}\th{}", self.node, cs.size(), cs.height(),)?;
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
        v: Default::default(),
        phantom: PhantomData,
    };
    let compressed_node_store = NS::<Tree> { v: vec![] };
    (label_store, compressed_node_store)
}

#[derive(PartialEq, Eq)]
pub struct Tree {
    pub t: u8,
    pub label: u16,
    pub children: Vec<u16>,
    pub size: u16,
    pub height: u16,
}

impl types::NodeStoreExt<Tree> for NS<Tree> {
    fn build_then_insert(
        &mut self,
        _i: <Tree as types::Stored>::TreeId,
        t: <Tree as types::Typed>::Type,
        _l: Option<<Tree as types::Labeled>::Label>,
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

impl types::Typed for Tree {
    type Type = u8;

    fn get_type(&self) -> Self::Type {
        self.t
    }
}

impl types::WithSerialization for Tree {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}

impl<T> types::WithSerialization for TreeRef<'_, T> {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}
impl<T> Clone for TreeRef<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: types::Typed> types::Typed for TreeRef<'_, T> {
    type Type = T::Type;

    fn get_type(&self) -> Self::Type {
        self.0.get_type()
    }
}
impl types::Labeled for Tree {
    type Label = u16;

    fn get_label_unchecked(&self) -> &Self::Label {
        &self.label
    }

    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        (self.label != 0).then_some(self.get_label_unchecked())
    }
}
impl<T: types::Labeled> types::Labeled for TreeRef<'_, T> {
    type Label = T::Label;

    fn get_label_unchecked(&self) -> &Self::Label {
        self.0.get_label_unchecked()
    }

    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        self.0.try_get_label()
    }
}
impl types::Node for Tree {}
impl<T: types::Node> types::Node for TreeRef<'_, T> {}
impl types::Tree for Tree {
    fn has_children(&self) -> bool {
        self.children.len() > 0
    }

    fn has_label(&self) -> bool {
        self.label != 0
    }
}

impl types::ErasedHolder for Tree {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<Ty>() {
            let t = &self.t;
            let t = unsafe { std::mem::transmute(t) };
            Some(t)
        } else {
            None
        }
    }
}

impl<'a, T: types::Tree> types::ErasedHolder for TreeRef<'_, T> {
    fn unerase_ref<TT: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&TT> {
        self.0.unerase_ref(tid)
    }
}

impl<T: types::Tree> types::Tree for TreeRef<'_, T>
where
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_label(&self) -> bool {
        self.0.has_label()
    }
}

impl types::Stored for Tree {
    type TreeId = u16;
}
impl<T: types::Stored> types::Stored for TreeRef<'_, T> {
    type TreeId = T::TreeId;
}

impl<'a> types::CLending<'a, u8, u16> for Tree {
    type Children = types::ChildrenSlice<'a, u16>;
}

impl<'a> types::CLending<'a, u16, u16> for Tree {
    type Children = types::ChildrenSlice<'a, u16>;
}

impl WithChildren for Tree {
    type ChildIdx = u8;

    fn child_count(&self) -> Self::ChildIdx {
        self.children.len() as u8
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.children.get(idx.to_usize().unwrap()).map(|x| *x)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        let idx = num::CheckedSub::checked_sub(&self.child_count(), &(*idx + 1))?;
        self.children.get(idx.to_usize().unwrap()).copied()
    }

    fn children(&self) -> Option<types::LendC<'_, Self, u8, u16>> {
        Some(self.children.as_slice().into())
    }
}

impl<'a, T: WithChildren> types::CLending<'a, T::ChildIdx, T::TreeId> for TreeRef<'_, T>
where
    T: types::CLending<'a, T::ChildIdx, T::TreeId>,
{
    type Children = <T as types::CLending<'a, T::ChildIdx, T::TreeId>>::Children;
}

impl<T: WithChildren> WithChildren for TreeRef<'_, T>
where
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
{
    type ChildIdx = T::ChildIdx;

    // type Children<'a>
    //     = T::Children<'a>
    // where
    //     Self: 'a;

    fn child_count(&self) -> Self::ChildIdx {
        self.0.child_count()
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.0.child(idx)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.0.child_rev(idx)
    }

    fn children(
        &self,
    ) -> Option<types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>> {
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

    fn line_count(&self) -> usize {
        todo!()
    }
}

impl types::WithMetaData<compo::StmtCount> for Tree {
    fn get_metadata(&self) -> Option<&compo::StmtCount> {
        if (42..52).contains(&self.t) {
            return Some(&compo::StmtCount(1));
        }
        (52..62).contains(&self.t).then_some(&compo::StmtCount(2))
    }
}

impl types::WithMetaData<compo::MemberImportCount> for Tree {
    fn get_metadata(&self) -> Option<&compo::MemberImportCount> {
        if (100..120).contains(&self.t) {
            return Some(&compo::MemberImportCount(1));
        }
        (100..140)
            .contains(&self.t)
            .then_some(&compo::MemberImportCount(2))
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

    fn line_count(&self) -> usize {
        self.0.line_count()
    }
}

impl<T: Stored + types::WithMetaData<compo::StmtCount>> types::WithMetaData<compo::StmtCount>
    for TreeRef<'_, T>
{
    fn get_metadata(&self) -> Option<&compo::StmtCount> {
        self.0.get_metadata()
    }
}

impl<T: Stored + types::WithMetaData<compo::MemberImportCount>>
    types::WithMetaData<compo::MemberImportCount> for TreeRef<'_, T>
{
    fn get_metadata(&self) -> Option<&compo::MemberImportCount> {
        self.0.get_metadata()
    }
}

#[derive(Clone, Copy)]
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

impl std::ops::Deref for H {
    type Target = Self;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl types::WithHashs for Tree {
    type HK = H;
    type HP = u8;
    fn hash(&self, _kind: impl std::ops::Deref<Target = Self::HK>) -> u8 {
        0
    }
}

impl<T: types::WithHashs> types::WithHashs for TreeRef<'_, T> {
    type HK = T::HK;
    type HP = T::HP;
    fn hash(&self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP {
        self.0.hash(kind)
    }
}

pub struct NS<T> {
    v: Vec<T>,
}

impl<T: 'static + types::Tree> types::NStore for NS<T> {
    type IdN = <T as types::Stored>::TreeId;
    type Idx = T::ChildIdx;
}

impl<'a, T: types::Tree> types::NLending<'a, T::TreeId> for NS<T>
where
    T::TreeId: ToPrimitive,
{
    type N = TreeRef<'a, T>;
}

impl<T: types::Tree> NodeStore<T::TreeId> for NS<T>
where
    T::TreeId: ToPrimitive,
{
    fn resolve(&self, id: &T::TreeId) -> TreeRef<'_, T> {
        TreeRef(&self.v[id.to_usize().unwrap()])
    }
}

#[derive(PartialEq, Eq)]
pub struct TreeRef<'a, T>(&'a T);

impl<T: types::Tree + Eq> NodeStoreMut<T> for NS<T>
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

pub struct LS<I> {
    // v: RefCell<Vec<crate::types::OwnedLabel>>,
    v: Vec<types::OwnedLabel>,
    phantom: PhantomData<*const I>,
}

impl<'a, I> types::LStore for LS<I> {
    type I = I;
}

impl<'a, I: PrimInt> LabelStore<types::SlicedLabel> for LS<I> {
    type I = I;
    fn get_or_insert<T: Borrow<types::SlicedLabel>>(&mut self, node: T) -> Self::I {
        let a = &mut self.v;
        let b = a
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x.eq(node.borrow()) { Some(i) } else { None })
            .to_owned();
        if let Some(i) = b {
            cast(i + 1).unwrap()
        } else {
            let l = a.len();
            a.push(node.borrow().to_owned());
            cast(l + 1).unwrap()
        }
    }

    fn get<T: Borrow<types::SlicedLabel>>(&self, node: T) -> Option<Self::I> {
        let a = &self.v;
        let b = a
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x.eq(node.borrow()) { Some(i) } else { None })
            .to_owned();
        b.map(|i| cast(i + 1).unwrap())
    }

    fn resolve(&self, id: &Self::I) -> &types::SlicedLabel {
        &self.v[cast::<Self::I, usize>(*id).unwrap() - 1]
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

pub struct TStore;

#[derive(Clone, Copy, std::hash::Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))] // todo only for bevy
pub struct Ty(u8);

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl HyperType for Ty {
    fn as_shared(&self) -> types::Shared {
        todo!()
    }

    fn as_abstract(&self) -> types::Abstracts {
        if (42..62).contains(&self.0) {
            types::Abstract::Statement.into()
        } else {
            Default::default()
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        todo!()
    }

    fn as_static_str(&self) -> &'static str {
        todo!()
    }

    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + Sized,
    {
        match other.as_any().downcast_ref::<Self>() {
            Some(other_concrete) => self == other_concrete,
            _ => false,
        }
    }

    fn is_file(&self) -> bool {
        false
    }

    fn is_directory(&self) -> bool {
        false
    }

    fn is_spaces(&self) -> bool {
        todo!()
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn is_hidden(&self) -> bool {
        todo!()
    }

    fn is_named(&self) -> bool {
        todo!()
    }

    fn is_supertype(&self) -> bool {
        todo!()
    }

    fn get_lang(&self) -> types::LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn lang_ref(&self) -> types::LangWrapper<types::AnyType> {
        todo!()
    }
}

impl types::TypeStore for TStore {
    type Ty = self::Ty;
}
