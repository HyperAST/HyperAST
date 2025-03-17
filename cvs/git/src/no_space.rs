use std::ops::Deref;

use hyperast::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, AnyType, Children, NodeId, SimpleHyperAST, TypedNodeId, AAAA},
};

use crate::SimpleStores;

// pub trait NoSpaceNodeStoreContainer: types::HyperASTAsso {
//     type NST<'store>: types::Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx> where Self: 'store;

//     type NSNS<'store>: types::NodeStore<Self::IdN, R<'store> = Self::NST<'store>>
//     where
//         Self: 'store;
//     fn no_spaces_node_store<'a>(&'a self) -> Self::NSNS<'a>;
// }

pub trait AsNoSpace {
    type R;
    fn as_nospaces(&self) -> &Self::R;
}

pub trait AsNoSpace2 {
    type R;
    fn as_nospaces(&self) -> Self::R;
}

// impl<'a, T: types::Stored, TS: 'a, NS: 'a, LS: 'a> AsNoSpace2
//     for &'a hyperast::store::SimpleStores<TS, NS, LS>
// {
//     type R = SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, TS, NoSpaceNodeStore<NS>, LS>;

//     fn as_nospaces(&self) -> &Self::R {
//         unsafe { std::mem::transmute(self) }
//     }
// }

impl<'a, T: types::Stored, TS: 'a, NS: 'a, LS: 'a> AsNoSpace
    for &'a hyperast::types::SimpleHyperAST<T, TS, NS, LS>
{
    type R = SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, TS, NoSpaceNodeStore<NS>, LS>;

    fn as_nospaces(&self) -> &Self::R {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: types::Stored, TS, NS, LS> AsNoSpace for hyperast::types::SimpleHyperAST<T, TS, NS, LS> {
    type R = SimpleHyperAST<NoSpaceNode<T>, TS, NoSpaceNodeStore<NS>, LS>;

    fn as_nospaces(&self) -> &Self::R {
        unsafe { std::mem::transmute(self) }
    }
}

impl<TS, NS, LS> AsNoSpace for hyperast::store::SimpleStores<TS, NS, LS> {
    type R = hyperast::store::SimpleStores<TS, NoSpaceNodeStore<NS>, LS>;

    fn as_nospaces(&self) -> &Self::R {
        unsafe { std::mem::transmute(self) }
    }
}

pub trait IntoNoSpaceGAT {
    type R<'a>
    where
        Self: 'a;
    fn as_nospaces(&self) -> Self::R<'_>;
    fn as_nospaces2<'a>(&'a self) -> Self::R<'a>;
}

impl<'a, T: types::Stored, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceGAT
    for &'a hyperast::types::SimpleHyperAST<T, TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b>
        = SimpleHyperAST<NoSpaceWrapper<'b, T::TreeId>, &'b TS, NoSpaceNodeStoreWrapper<'b>, &'b LS>
    where
        Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }

    fn as_nospaces2(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'a, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceGAT for &'a hyperast::store::SimpleStores<TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b>
        = SimpleHyperAST<
        NoSpaceWrapper<'b, NodeIdentifier>,
        &'b TS,
        NoSpaceNodeStoreWrapper<'b>,
        &'b LS,
    >
    where
        Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    fn as_nospaces2(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait NoSpaceMarker {}

impl<T: NoSpaceMarker, TS, NS: NoSpaceMarker, LS> NoSpaceMarker for SimpleHyperAST<T, TS, NS, LS> {}
impl<TS, NS: NoSpaceMarker, LS> NoSpaceMarker for hyperast::store::SimpleStores<TS, NS, LS> {}

impl<'a, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceGAT for hyperast::store::SimpleStores<TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b>
        =
        SimpleHyperAST<NoSpaceWrapper<'b, NodeIdentifier>, TS, NoSpaceNodeStoreWrapper<'b>, &'b LS>
    where
        Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    fn as_nospaces2(&self) -> Self::R<'_> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
// impl<TS> NoSpaceNodeStoreContainer for hyperast::store::SimpleStores<TS>
// where
//     TS: for<'s> hyperast::types::TypeStore<hyperast::store::nodes::legion::HashedNodeRef<'s>>,
// {
//     type NST<'store> = NoSpaceWrapper<'store, Self::IdN> where Self: 'store;
//     type NSNS<'store> = NoSpaceNodeStoreWrapper<'store> where Self: 'store;

//     fn no_spaces_node_store<'b>(&'b self) -> Self::NSNS<'b> {
//         Into::<NoSpaceNodeStoreWrapper>::into(&self.node_store)
//     }
// }

pub trait IntoNoSpaceLife<'a> {
    type R<'b>
    where
        Self: 'b,
        Self: 'a;
    fn as_nospaces(&'a self) -> Self::R<'a>;
}

impl<'a, T: types::Stored, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceLife<'a>
    for &hyperast::types::SimpleHyperAST<T, TS, NS, LS>
where
    NoSpaceNodeStoreWrapper<'a>: From<&'a NS>,
{
    type R<'b>
        = SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, TS, NoSpaceNodeStoreWrapper<'a>, &'a LS>
    where
        Self: 'b,
        Self: 'a;

    fn as_nospaces(&'a self) -> Self::R<'a> {
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub fn as_nospaces<'a, TS>(
    stores: &'a hyperast::store::SimpleStores<TS>,
) -> SimpleHyperAST<
    NoSpaceWrapper<'static, NodeIdentifier>,
    TS,
    NoSpaceNodeStoreWrapper<'a>,
    &'a hyperast::store::labels::LabelStore,
> {
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    let node_store = node_store.into();
    SimpleHyperAST {
        node_store,
        label_store,
        _phantom: std::marker::PhantomData,
    }
}

pub fn as_nospaces2<'a, TS>(
    stores: &'a hyperast::store::SimpleStores<TS>,
) -> hyperast::store::SimpleStores<
    TS,
    NoSpaceNodeStoreWrapper<'a>,
    &'a hyperast::store::labels::LabelStore,
> {
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    let node_store = node_store.into();
    hyperast::store::SimpleStores {
        node_store,
        label_store,
        type_store: std::marker::PhantomData,
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NoSpaceNodeStoreWrapper<'a> {
    pub s: &'a NodeStore,
}
impl<'a> NoSpaceMarker for NoSpaceNodeStoreWrapper<'a> {}

#[repr(transparent)]
pub struct NoSpaceNodeStore<NS> {
    pub s: NS,
}

impl<'a> From<&'a NodeStore> for NoSpaceNodeStoreWrapper<'a> {
    fn from(value: &'a NodeStore) -> Self {
        NoSpaceNodeStoreWrapper { s: value }
    }
}

impl<NS> From<NS> for NoSpaceNodeStore<NS> {
    fn from(s: NS) -> Self {
        Self { s }
    }
}

impl<NS> From<&NS> for &NoSpaceNodeStore<NS> {
    fn from(s: &NS) -> Self {
        unsafe { std::mem::transmute(s) }
    }
}

// impl<'a, T> From<&'a NodeStore> for NoSpaceNodeStoreWrapper<'a, T> {
//     fn from(value: &'a NodeStore) -> Self {
//         NoSpaceNodeStoreWrapper {
//             s: value,
//             phantom: PhantomData,
//         }
//     }
// }

#[repr(transparent)]
pub struct NoSpaceWrapper<'a, T> {
    pub(crate) inner: HashedNodeRef<'a, T>,
}

impl<'a, T> NoSpaceMarker for NoSpaceWrapper<'a, T> {}

#[repr(transparent)]
pub struct NoSpaceNode<N> {
    inner: N,
}

impl<'a, T> AsRef<HashedNodeRef<'a, T>> for NoSpaceWrapper<'a, T> {
    fn as_ref(&self) -> &HashedNodeRef<'a, T> {
        &self.inner
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct MIdN<IdN>(pub IdN);

impl<IdN> Deref for MIdN<IdN> {
    type Target = IdN;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<IdN: Clone + Eq + AAAA> NodeId for MIdN<IdN> {
    type IdN = IdN;

    fn as_id(&self) -> &Self::IdN {
        &self.0
    }

    unsafe fn from_id(id: Self::IdN) -> Self {
        Self(id)
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        std::mem::transmute(id)
    }
}

// impl<IdN: Clone + Eq + NodeId> TypedNodeId for MIdN<IdN> {
//     type Ty = AnyType;
//     type TyErazed = TType;
//     fn unerase(ty: Self::TyErazed) -> Self::Ty {
//         ty.e()
//     }
// }

impl<'a, T> types::WithStats for NoSpaceWrapper<'a, T> {
    fn size(&self) -> usize {
        self.inner.size_no_spaces()
    }

    fn height(&self) -> usize {
        self.inner.height()
    }

    fn line_count(&self) -> usize {
        self.inner.line_count()
    }
}

impl<'a, T> types::WithSerialization for NoSpaceWrapper<'a, T> {
    /// WARN return the len with spaces ? YES
    fn try_bytes_len(&self) -> Option<usize> {
        self.inner.try_bytes_len()
    }
}

impl<'a, T> types::Labeled for NoSpaceWrapper<'a, T> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        self.inner.get_label_unchecked()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.inner.try_get_label()
    }
}

impl<'a, T> types::Node for NoSpaceWrapper<'a, T> {}

impl<'a, T: types::NodeId> types::Stored for NoSpaceWrapper<'a, T> {
    type TreeId = T::IdN;
}

// // NOTE: use of the deref polymorphism trick
// impl<'a, T: 'static + TypedNodeId<IdN = NodeIdentifier>> types::Typed for &NoSpaceWrapper<'a, T> {
//     type Type = <T as TypedNodeId>::Ty;

//     fn get_type(&self) -> Self::Type {
//         self.inner.get_type()
//     }
// }

// impl<'a> NoSpaceWrapper<'a> {
//     fn cs(&self) -> Option<&NoSpaceSlice<<Self as types::Stored>::TreeId>> {
//         self.inner.cs().map(|x|x.into()).ok()
//     }
// }

// impl<'a, T> types::Labeled for &NoSpaceWrapper<'a, T> {
//     type Label = LabelIdentifier;

//     fn get_label_unchecked(&self) -> &LabelIdentifier {
//         self.inner.get_label_unchecked()
//     }

//     fn try_get_label(&self) -> Option<&Self::Label> {
//         self.inner.try_get_label()
//     }
// }

// impl<'a, T> types::Node for &NoSpaceWrapper<'a, T> {}

// impl<'a, T> types::Stored for &NoSpaceWrapper<'a, T> {
//     type TreeId = NodeIdentifier;
// }
// impl<'a, T> types::WithChildren for &NoSpaceWrapper<'a, T> {
//     type ChildIdx = u16;
//     type Children<'b> = MySlice<Self::TreeId> where Self: 'b;

//     fn child_count(&self) -> u16 {
//         self.inner.no_spaces().map_or(0, |x| x.child_count())
//     }

//     fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
//         self.inner
//             .no_spaces()
//             .ok()
//             .and_then(|x| x.get(*idx).copied())
//     }

//     fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
//         self.inner
//             .no_spaces()
//             .ok()
//             .and_then(|x| x.rev(*idx).copied())
//     }

//     fn children(&self) -> Option<LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>> {
//         self.inner.no_spaces().ok()
//     }
// }

// impl<'a, T: TypedNodeId<IdN = NodeIdentifier> + 'static> types::Tree for &NoSpaceWrapper<'a, T> {
//     fn has_children(&self) -> bool {
//         self.inner.has_children()
//     }

//     fn has_label(&self) -> bool {
//         self.inner.has_label()
//     }
// }

impl<'a, T: types::NodeId> types::CLending<'a, u16, T::IdN> for NoSpaceWrapper<'_, T> {
    type Children = types::ChildrenSlice<'a, T::IdN>;
}

impl<'a, T: types::NodeId<IdN = NodeIdentifier>> types::WithChildren for NoSpaceWrapper<'a, T> {
    type ChildIdx = u16;
    // type Children<'b>
    //     = MySlice<Self::TreeId>
    // where
    //     Self: 'b;

    fn child_count(&self) -> u16 {
        self.inner.no_spaces().map_or(0, |x| x.child_count())
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.inner
            .no_spaces()
            .ok()
            .and_then(|x| x.get(*idx).copied())
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        let v = self.inner.no_spaces().ok()?;
        let c: Self::ChildIdx = v.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        v.get(c).cloned()
    }

    fn children(
        &self,
    ) -> Option<hyperast::types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>>
    {
        self.inner.no_spaces().ok()
    }
}

impl<'a, T> types::WithHashs for NoSpaceWrapper<'a, T> {
    type HK = hyperast::hashed::SyntaxNodeHashsKinds;
    type HP = hyperast::nodes::HashSize;

    fn hash<'b>(&'b self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP {
        self.inner.hash(kind)
    }
}

impl<'a> hyperast::types::ErasedHolder for NoSpaceWrapper<'a, MIdN<NodeIdentifier>> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        self.inner.unerase_ref(tid)
    }

    unsafe fn unerase_ref_unchecked<T: 'static + types::Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        self.inner.unerase_ref_unchecked(tid)
    }
}

// impl<'a> types::Tree for NoSpaceWrapper<'a, MIdN<NodeIdentifier>> {
//     fn has_children(&self) -> bool {
//         self.inner.has_children()
//     }

//     fn has_label(&self) -> bool {
//         self.inner.has_label()
//     }
// }

impl<'a> hyperast::types::ErasedHolder for NoSpaceWrapper<'a, NodeIdentifier> {
    unsafe fn unerase_ref_unchecked<T: 'static + hyperast::types::Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        self.inner.unerase_ref_unchecked(tid)
    }

    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        self.inner.unerase_ref(tid)
    }
}

impl<'a> types::Tree for NoSpaceWrapper<'a, NodeIdentifier> {
    fn has_children(&self) -> bool {
        self.inner.has_children()
    }

    fn has_label(&self) -> bool {
        self.inner.has_label()
    }
}

// impl<'store, T: TypedNodeId<IdN = NodeIdentifier>> types::NodeStore<T>
//     for NoSpaceNodeStoreWrapper<'store, T>
// {
//     type R<'a> = NoSpaceWrapper<'a,T> where Self: 'a;
//     fn resolve(&self, id: &T) -> Self::R<'_> {
//         NoSpaceWrapper {
//             inner: unsafe { self.s._resolve(id.as_id()) },
//         }
//     }
// }

impl<'store> types::NStore for NoSpaceNodeStoreWrapper<'store> {
    type IdN = NodeIdentifier;

    type Idx = u16;
}

// impl<'store> types::NodStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
//     type R<'a> = NoSpaceWrapper<'a, NodeIdentifier>;
// }

impl<'a, 'store> types::lending::NLending<'a, NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type N = NoSpaceWrapper<'a, NodeIdentifier>;
}

impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    fn resolve(&self, id: &NodeIdentifier) -> types::LendN<'_, Self, NodeIdentifier> {
        NoSpaceWrapper {
            inner: unsafe { self.s._resolve(id.as_id()) },
        }
    }
}

impl<'store> types::inner_ref::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type Ref = NoSpaceWrapper<'static, NodeIdentifier>;

    fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
        let t = &NoSpaceWrapper {
            inner: unsafe { self.s._resolve::<NodeIdentifier>(id.as_id()) },
        };
        // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
        let t = unsafe { std::mem::transmute(t) };
        f(t)
    }

    fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
        let t = &NoSpaceWrapper {
            inner: unsafe { self.s._resolve::<NodeIdentifier>(id.as_id()) },
        };
        // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
        let t = unsafe { std::mem::transmute(t) };
        f(t)
    }

    fn multi<R, const N: usize>(
        &self,
        id: &[NodeIdentifier; N],
        f: impl Fn(&[Self::Ref; N]) -> R,
    ) -> R {
        todo!()
    }
}

impl<'store> types::inner_ref::NodeStore<NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    type Ref = NoSpaceWrapper<'static, NodeIdentifier>;

    fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
        (*self).scoped(id, f)
    }

    fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
        (*self).scoped_mut(id, f)
    }

    fn multi<R, const N: usize>(
        &self,
        id: &[NodeIdentifier; N],
        f: impl Fn(&[Self::Ref; N]) -> R,
    ) -> R {
        (*self).multi(id, f)
    }
}

impl<'store> types::NStore for &NoSpaceNodeStoreWrapper<'store> {
    type IdN = NodeIdentifier;

    type Idx = u16;
}

// impl<'store> types::NodStore<NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
//     type R<'a> = NoSpaceWrapper<'a, NodeIdentifier>;
// }

impl<'a, 'store> types::lending::NLending<'a, NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    type N = NoSpaceWrapper<'a, NodeIdentifier>;
}

impl<'store> types::NodeStore<NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    // type NMarker = TMarker<NodeIdentifier>;
    fn resolve(&self, id: &NodeIdentifier) -> types::LendN<'_, Self, NodeIdentifier> {
        NoSpaceWrapper {
            inner: unsafe { self.s._resolve(id.as_id()) },
        }
    }
}

// impl<NS> types::NodeStore<MIdN<NodeIdentifier>> for NoSpaceNodeStore<NS> {
//     type R<'a> = NoSpaceWrapper<'a, MIdN<NodeIdentifier>> where Self: 'a;
//     fn resolve(&self, id: &MIdN<NodeIdentifier>) -> Self::R<'_> {
//         NoSpaceWrapper {
//             inner: unsafe { self.s._resolve(id.as_id()) },
//         }
//     }
// }

// impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store, NodeIdentifier> {
//     type R<'a> = NoSpaceWrapper<'a, NodeIdentifier> where Self: 'a;
//     fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
//         NoSpaceWrapper {
//             inner: unsafe { self.s._resolve(id.as_id()) },
//         }
//     }
// }

// impl<'store> &NoSpaceNodeStoreWrapper<'store, MIdN<NodeIdentifier>>> {
//     fn from(value: &NoSpaceNodeStoreWrapper<'store, MIdN<NodeIdentifier>>) -> Self {
//         tr
//     }
// }

impl<'store> NoSpaceNodeStoreWrapper<'store> {
    pub fn resolve(&self, id: NodeIdentifier) -> NoSpaceWrapper<'store, NodeIdentifier> {
        NoSpaceWrapper {
            inner: types::NodeStore::resolve(self.s, &id),
        }
    }
}
