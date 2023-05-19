use std::ops::Deref;

use crate::{MultiType, TStore};
use hyper_ast::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, Children, MySlice, NodeId, SimpleHyperAST, TypedNodeId},
};

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
pub struct NoSpaceNodeStoreWrapper<'a> {
    pub s: &'a NodeStore,
}

#[repr(transparent)]
pub struct NoSpaceNodeStoreWrapperT<'a> {
    pub s: &'a NodeStore,
}

impl<'a> From<&'a NodeStore> for NoSpaceNodeStoreWrapper<'a> {
    fn from(value: &'a NodeStore) -> Self {
        NoSpaceNodeStoreWrapper { s: value }
    }
}

impl<'a> From<NoSpaceNodeStoreWrapper<'a>> for NoSpaceNodeStoreWrapperT<'a> {
    fn from(value: NoSpaceNodeStoreWrapper<'a>) -> Self {
        NoSpaceNodeStoreWrapperT { s: value.s }
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

// impl<'a> NoSpaceWrapper<'a> {
//     fn cs(&self) -> Option<&NoSpaceSlice<<Self as types::Stored>::TreeId>> {
//         self.inner.cs().map(|x|x.into()).ok()
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

impl<'store> types::NodeStore<MIdN<NodeIdentifier>> for NoSpaceNodeStoreWrapperT<'store> {
    type R<'a> = NoSpaceWrapper<'a, MIdN<NodeIdentifier>> where Self: 'a;
    fn resolve(&self, id: &MIdN<NodeIdentifier>) -> Self::R<'_> {
        NoSpaceWrapper {
            inner: unsafe { self.s._resolve(id.as_id()) },
        }
    }
}

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
