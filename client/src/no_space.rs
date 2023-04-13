use hyper_ast::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::{legion::NodeStore, HashedNodeRef},
    },
    types::{self, Children, MySlice, SimpleHyperAST},
};

pub(crate) fn as_nospaces<'a>(
    stores: &'a hyper_ast::store::SimpleStores,
) -> SimpleHyperAST<
    NoSpaceWrapper<'a>,
    NoSpaceNodeStoreWrapper<'a>,
    &'a hyper_ast::store::labels::LabelStore,
> {
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    let node_store = NoSpaceNodeStoreWrapper { s: node_store };
    SimpleHyperAST {
        node_store,
        label_store,
        _phantom: std::marker::PhantomData,
    }
}

#[repr(transparent)]
pub(crate) struct NoSpaceNodeStoreWrapper<'a> {
    pub(crate) s: &'a NodeStore,
}

#[repr(transparent)]
pub(crate) struct NoSpaceWrapper<'a> {
    inner: HashedNodeRef<'a>,
}

impl<'a> types::Typed for NoSpaceWrapper<'a> {
    type Type = types::Type;

    fn get_type(&self) -> types::Type {
        self.inner.get_type()
    }
}

impl<'a> types::WithStats for NoSpaceWrapper<'a> {
    fn size(&self) -> usize {
        self.inner.size_no_spaces()
    }

    fn height(&self) -> usize {
        self.inner.height()
    }
}

impl<'a> types::WithSerialization for NoSpaceWrapper<'a> {
    /// WARN return the len with spaces ? YES
    fn try_bytes_len(&self) -> Option<usize> {
        self.inner.try_bytes_len()
    }
}

impl<'a> types::Labeled for NoSpaceWrapper<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.inner.get_label()
    }
}

impl<'a> types::Node for NoSpaceWrapper<'a> {}

impl<'a> types::Stored for NoSpaceWrapper<'a> {
    type TreeId = NodeIdentifier;
}

// impl<'a> NoSpaceWrapper<'a> {
//     fn cs(&self) -> Option<&NoSpaceSlice<<Self as types::Stored>::TreeId>> {
//         self.inner.cs().map(|x|x.into()).ok()
//     }
// }

impl<'a> types::WithChildren for NoSpaceWrapper<'a> {
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

impl<'a> types::WithHashs for NoSpaceWrapper<'a> {
    type HK = hyper_ast::hashed::SyntaxNodeHashsKinds;
    type HP = hyper_ast::nodes::HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.inner.hash(kind)
    }
}

impl<'a> types::Tree for NoSpaceWrapper<'a> {
    fn has_children(&self) -> bool {
        self.inner.has_children()
    }

    fn has_label(&self) -> bool {
        self.inner.has_label()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.inner.try_get_label()
    }
}

impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type R<'a> = NoSpaceWrapper<'a> where Self: 'a;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        NoSpaceWrapper {
            inner: types::NodeStore::resolve(self.s, id),
        }
    }
}


impl<'store> NoSpaceNodeStoreWrapper<'store> {
    pub fn resolve(&self, id: NodeIdentifier) -> <Self as hyper_ast::types::NodeStore<NodeIdentifier>>::R<'_> {
        NoSpaceWrapper {
            inner: types::NodeStore::resolve(self.s, &id),
        }
    }
}
