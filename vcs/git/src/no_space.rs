use std::ops::Deref;

use hyperast::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, AAAA, Children, NodeId},
};

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

// TODO use ref_cast
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
        unsafe { std::mem::transmute(s) } // TODO use ref_cast
    }
}

#[repr(transparent)]
pub struct NoSpaceWrapper<'a, T>(HashedNodeRef<'a, T>);

#[repr(transparent)]
pub struct NoSpaceNode<N> {
    inner: N,
}

impl<'a, T> AsRef<HashedNodeRef<'a, T>> for NoSpaceWrapper<'a, T> {
    fn as_ref(&self) -> &HashedNodeRef<'a, T> {
        &self.0
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
        unsafe { std::mem::transmute(id) }
    }
}

impl<'a, T> types::WithStats for NoSpaceWrapper<'a, T> {
    fn size(&self) -> usize {
        self.0.size_no_spaces()
    }

    fn height(&self) -> usize {
        self.0.height()
    }

    fn line_count(&self) -> usize {
        self.0.line_count()
    }
}

impl<'a, T> types::WithSerialization for NoSpaceWrapper<'a, T> {
    /// WARN return the len with spaces ? YES
    fn try_bytes_len(&self) -> Option<usize> {
        self.0.try_bytes_len()
    }
}

impl<'a, T> types::Labeled for NoSpaceWrapper<'a, T> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        self.0.get_label_unchecked()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.0.try_get_label()
    }
}

impl<'a, T> types::Node for NoSpaceWrapper<'a, T> {}

impl<'a, T: types::NodeId> types::Stored for NoSpaceWrapper<'a, T> {
    type TreeId = T::IdN;
}

impl<'a, T: types::NodeId> types::CLending<'a, u16, T::IdN> for NoSpaceWrapper<'_, T> {
    type Children = types::ChildrenSlice<'a, T::IdN>;
}

impl<'a, T: types::NodeId<IdN = NodeIdentifier>> types::WithChildren for NoSpaceWrapper<'a, T> {
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.0.no_spaces().map_or(0, |x| x.child_count())
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.0.no_spaces().ok().and_then(|x| x.get(*idx).copied())
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        let v = self.0.no_spaces().ok()?;
        let c: Self::ChildIdx = v.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        v.get(c).cloned()
    }

    fn children(
        &self,
    ) -> Option<hyperast::types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>>
    {
        self.0.no_spaces().ok()
    }
}

impl<'a, T> types::WithHashs for NoSpaceWrapper<'a, T> {
    type HK = hyperast::hashed::SyntaxNodeHashsKinds;
    type HP = hyperast::nodes::HashSize;

    fn hash<'b>(&'b self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP {
        self.0.hash(kind)
    }
}

impl<'a> hyperast::types::ErasedHolder for NoSpaceWrapper<'a, MIdN<NodeIdentifier>> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        self.0.unerase_ref(tid)
    }

    unsafe fn unerase_ref_unchecked<T: hyperast::types::Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        unsafe { self.0.unerase_ref_unchecked(tid) }
    }
}

impl<'a> hyperast::store::nodes::ErasedHolder for NoSpaceWrapper<'a, NodeIdentifier> {
    unsafe fn unerase_ref_unchecked<T: 'static + hyperast::store::nodes::Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        unsafe { self.0.unerase_ref_unchecked(tid) }
    }

    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        self.0.unerase_ref(tid)
    }
}

impl<'a> types::Tree for NoSpaceWrapper<'a, NodeIdentifier> {
    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_label(&self) -> bool {
        self.0.has_label()
    }
}

impl<'store> types::NStore for NoSpaceNodeStoreWrapper<'store> {
    type IdN = NodeIdentifier;
    type Idx = u16;
}

impl<'a, 'store> types::lending::NLending<'a, NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type N = NoSpaceWrapper<'a, NodeIdentifier>;
}

impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    fn resolve(&self, id: &NodeIdentifier) -> types::LendN<'_, Self, NodeIdentifier> {
        NoSpaceWrapper(self.s.resolve(*id))
    }
}

impl<'store> types::NStore for &NoSpaceNodeStoreWrapper<'store> {
    type IdN = NodeIdentifier;
    type Idx = u16;
}

impl<'a, 'store> types::lending::NLending<'a, NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    type N = NoSpaceWrapper<'a, NodeIdentifier>;
}

impl<'store> types::NodeStore<NodeIdentifier> for &NoSpaceNodeStoreWrapper<'store> {
    fn resolve(&self, id: &NodeIdentifier) -> types::LendN<'_, Self, NodeIdentifier> {
        NoSpaceWrapper(self.s.resolve(*id))
    }
}

impl<'store> NoSpaceNodeStoreWrapper<'store> {
    pub fn resolve(&self, id: NodeIdentifier) -> NoSpaceWrapper<'store, NodeIdentifier> {
        NoSpaceWrapper(types::NodeStore::resolve(self.s, &id))
    }
}
