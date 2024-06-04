use std::ops::Deref;

use crate::{MultiType, TStore};
use hyper_ast::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, Children, MySlice, NodeId, SimpleHyperAST, TypedNodeId},
};

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
//     for &'a hyper_ast::store::SimpleStores<TS, NS, LS>
// {
//     type R = SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, TS, NoSpaceNodeStore<NS>, LS>;

//     fn as_nospaces(&self) -> &Self::R {
//         unsafe { std::mem::transmute(self) }
//     }
// }

impl<'a, T: types::Stored, TS: 'a, NS: 'a, LS: 'a> AsNoSpace
    for &'a hyper_ast::types::SimpleHyperAST<T, TS, NS, LS>
{
    type R = SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, TS, NoSpaceNodeStore<NS>, LS>;

    fn as_nospaces(&self) -> &Self::R {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: types::Stored, TS, NS, LS> AsNoSpace for hyper_ast::types::SimpleHyperAST<T, TS, NS, LS> {
    type R = SimpleHyperAST<NoSpaceNode<T>, TS, NoSpaceNodeStore<NS>, LS>;

    fn as_nospaces(&self) -> &Self::R {
        unsafe { std::mem::transmute(self) }
    }
}

impl<TS, NS, LS> AsNoSpace for hyper_ast::store::SimpleStores<TS, NS, LS> {
    type R = hyper_ast::store::SimpleStores<TS, NoSpaceNodeStore<NS>, LS>;

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
    for &'a hyper_ast::types::SimpleHyperAST<T, TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b> = SimpleHyperAST<
        NoSpaceWrapper<'b, T::TreeId>,
        &'b TS,
        NoSpaceNodeStoreWrapper<'b>,
        &'b LS,
    > where Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }

    fn as_nospaces2(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'a, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceGAT for &'a hyper_ast::store::SimpleStores<TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b> = SimpleHyperAST<
        NoSpaceWrapper<'b, NodeIdentifier>,
        &'b TS,
        NoSpaceNodeStoreWrapper<'b>,
        &'b LS,
    > where Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    fn as_nospaces2(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait NoSpaceMarker {}

impl<T: NoSpaceMarker, TS, NS: NoSpaceMarker, LS> NoSpaceMarker for SimpleHyperAST<T, TS, NS, LS> {}
impl<TS, NS: NoSpaceMarker, LS> NoSpaceMarker for hyper_ast::store::SimpleStores<TS, NS, LS> {}

impl<'a, TS: 'a, NS: 'a, LS: 'a> IntoNoSpaceGAT for hyper_ast::store::SimpleStores<TS, NS, LS>
where
    for<'b> NoSpaceNodeStoreWrapper<'b>: From<&'b NS>,
{
    type R<'b> = SimpleHyperAST<
        NoSpaceWrapper<'b, NodeIdentifier>,
        &'b TS,
        NoSpaceNodeStoreWrapper<'b>,
        &'b LS,
    > where Self: 'b;

    fn as_nospaces(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    fn as_nospaces2(&self) -> Self::R<'_> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
// impl<TS> NoSpaceNodeStoreContainer for hyper_ast::store::SimpleStores<TS>
// where
//     TS: for<'s> hyper_ast::types::TypeStore<hyper_ast::store::nodes::legion::HashedNodeRef<'s>>,
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
    for &hyper_ast::types::SimpleHyperAST<T, TS, NS, LS>
where
    NoSpaceNodeStoreWrapper<'a>: From<&'a NS>,
{
    type R<'b> =
        SimpleHyperAST<NoSpaceWrapper<'a, T::TreeId>, &'a TS, NoSpaceNodeStoreWrapper<'a>, &'a LS> where Self: 'b, Self: 'a;

    fn as_nospaces(&'a self) -> Self::R<'a> {
        let type_store = &self.type_store;
        let label_store = &self.label_store;
        let node_store = &self.node_store;
        let node_store = node_store.into();
        SimpleHyperAST {
            type_store,
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

// impl<'a, TS: 'a> IntoNoSpace<'a> for hyper_ast::store::SimpleStores<TS> {
//     type R = SimpleHyperAST<
//         NoSpaceWrapper<'a, NodeIdentifier>,
//         &'a TS,
//         NoSpaceNodeStoreWrapper<'a>,
//         &'a hyper_ast::store::labels::LabelStore,
//     >;

//     fn as_nospaces(&'a self) -> Self::R {
//         let type_store = &self.type_store;
//         let label_store = &self.label_store;
//         let node_store = &self.node_store;
//         let node_store = node_store.into();
//         SimpleHyperAST {
//             type_store,
//             node_store,
//             label_store,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl<'a> IntoNoSpace<'a> for &hyper_ast::store::SimpleStores<TStore> {
//     type R = SimpleHyperAST<
//         NoSpaceWrapper<'a, NodeIdentifier>,
//         &'a TStore,
//         NoSpaceNodeStoreWrapper<'a>,
//         &'a hyper_ast::store::labels::LabelStore,
//     >;

//     fn as_nospaces(&'a self) -> Self::R {
//         let type_store = &self.type_store;
//         let label_store = &self.label_store;
//         let node_store = &self.node_store;
//         let node_store = node_store.into();
//         SimpleHyperAST {
//             type_store,
//             node_store,
//             label_store,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// pub fn as_nospaces<'a, TS>(
//     stores: &'a hyper_ast::store::SimpleStores<TS>,
// ) -> SimpleHyperAST<
//     NoSpaceWrapper<'a, NodeIdentifier>,
//     &'a TS,
//     NoSpaceNodeStoreWrapper<'a>,
//     &'a hyper_ast::store::labels::LabelStore,
// > {
//     let type_store = &stores.type_store;
//     let label_store = &stores.label_store;
//     let node_store = &stores.node_store;
//     let node_store = node_store.into();
//     SimpleHyperAST {
//         type_store,
//         node_store,
//         label_store,
//         _phantom: std::marker::PhantomData,
//     }
// }

pub fn as_nospaces<'a>(
    stores: &'a hyper_ast::store::SimpleStores<TStore>,
) -> SimpleHyperAST<
    NoSpaceWrapper<'a, NodeIdentifier>,
    &'a TStore,
    NoSpaceNodeStoreWrapper<'a>,
    &'a hyper_ast::store::labels::LabelStore,
> {
    let type_store = &stores.type_store;
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    let node_store = node_store.into();
    SimpleHyperAST {
        type_store,
        node_store,
        label_store,
        _phantom: std::marker::PhantomData,
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
    inner: HashedNodeRef<'a, T>,
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

impl<IdN: Clone + Eq + NodeId> NodeId for MIdN<IdN> {
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

impl<IdN: Clone + Eq + NodeId> TypedNodeId for MIdN<IdN> {
    type Ty = MultiType;
}

impl<'a> types::Typed for NoSpaceWrapper<'a, MIdN<NodeIdentifier>> {
    type Type = MultiType;

    fn get_type(&self) -> MultiType {
        // self.inner.get_type()

        if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_java::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_java::types::Java as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Java(*t)
        } else if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_cpp::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Cpp(*t)
        } else if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_xml::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_xml::types::Xml as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Xml(*t)
        } else {
            panic!()
        }
    }
}

impl<'a> types::Typed for NoSpaceWrapper<'a, NodeIdentifier> {
    type Type = MultiType;

    fn get_type(&self) -> MultiType {
        if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_java::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_java::types::Java as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Java(*t)
        } else if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_cpp::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Cpp(*t)
        } else if let Ok(t) = self
            .inner
            .get_component::<hyper_ast_gen_ts_xml::types::Type>()
        {
            let t = *t as u16;
            let t = <hyper_ast_gen_ts_xml::types::Xml as hyper_ast::types::Lang<_>>::make(t);
            MultiType::Xml(*t)
        } else {
            panic!()
        }
    }
}

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

impl<'a, T> types::Stored for NoSpaceWrapper<'a, T> {
    type TreeId = NodeIdentifier;
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

//     fn children(&self) -> Option<&Self::Children<'_>> {
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

impl<'a, T> types::WithChildren for NoSpaceWrapper<'a, T> {
    type ChildIdx = u16;
    type Children<'b> = MySlice<Self::TreeId> where Self: 'b;

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
        self.inner
            .no_spaces()
            .ok()
            .and_then(|x| x.rev(*idx).copied())
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        self.inner.no_spaces().ok()
    }
}

impl<'a, T> types::WithHashs for NoSpaceWrapper<'a, T> {
    type HK = hyper_ast::hashed::SyntaxNodeHashsKinds;
    type HP = hyper_ast::nodes::HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.inner.hash(kind)
    }
}

impl<'a> types::Tree for NoSpaceWrapper<'a, MIdN<NodeIdentifier>> {
    fn has_children(&self) -> bool {
        self.inner.has_children()
    }

    fn has_label(&self) -> bool {
        self.inner.has_label()
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

impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type R<'a> = NoSpaceWrapper<'a, NodeIdentifier> where Self: 'a;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        NoSpaceWrapper {
            inner: unsafe { self.s._resolve(id.as_id()) },
        }
    }
}

impl<'store> types::NodeStore<NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    type R<'a> = NoSpaceWrapper<'a, NodeIdentifier> where Self: 'a;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
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
