use std::{marker::PhantomData, num::NonZeroU64};

use crate::types::{NodeId, TypedNodeId};

mod boxing;
mod compo;
mod elem;
pub use elem::HashedNodeRef;

pub type NodeIdentifier = NonZeroU64;

impl crate::types::AAAA for NodeIdentifier {}
impl NodeId for NodeIdentifier {
    type IdN = Self;
    fn as_id(&self) -> &Self::IdN {
        self
    }

    unsafe fn from_id(id: Self::IdN) -> Self {
        id
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        id
    }
}

impl TypedNodeId for NodeIdentifier {
    type Ty = crate::types::AnyType;
    type TyErazed = crate::types::AnyType;

    fn unerase(ty: Self::TyErazed) -> Self::Ty {
        ty
    }
}

pub struct NodeStore {
    nodes: std::collections::HashMap<NodeIdentifier, boxing::ErasedMap>,
}

pub struct TMarker<IdN>(std::marker::PhantomData<IdN>);

impl<IdN> Default for TMarker<IdN> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a, IdN: 'a + crate::types::NodeId + TypedNodeId<IdN = NodeIdentifier>>
    crate::types::NLending<'a, IdN> for TMarker<IdN>
{
    type N = HashedNodeRef<'a, IdN>;
}

impl<IdN> crate::types::Node for TMarker<IdN> {}

impl<IdN: crate::types::NodeId> crate::types::Stored for TMarker<IdN> {
    type TreeId = IdN;
}

impl<'a> crate::types::lending::NLending<'a, NodeIdentifier> for NodeStore {
    type N = HashedNodeRef<'a, NodeIdentifier>;
}

impl crate::types::lending::NodeStore<NodeIdentifier> for NodeStore {
    fn resolve(
        &self,
        id: &NodeIdentifier,
    ) -> <Self as crate::types::lending::NLending<'_, NodeIdentifier>>::N {
        HashedNodeRef(self.nodes.get(id).unwrap(), PhantomData)
    }

    // type NMarker = TMarker<NodeIdentifier>;
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TyNodeStore<TIdN>
    for NodeStore
{
    type R<'a> = HashedNodeRef<'a, TIdN>; // TODO
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TypedNodeStore<TIdN>
    for NodeStore
{
    fn resolve(&self, id: &TIdN) -> Self::R<'_> {
        let r = self.nodes.get(id.as_id()).unwrap();
        let r: HashedNodeRef<<TIdN as NodeId>::IdN> = HashedNodeRef(r, PhantomData);
        assert!(r.get_component::<TIdN::Ty>().is_ok());
        HashedNodeRef(r.0, PhantomData)
    }

    fn try_typed(&self, id: &<TIdN as NodeId>::IdN) -> Option<TIdN> {
        let r = self.nodes.get(&id.as_id())?;
        let r: HashedNodeRef<<TIdN as NodeId>::IdN> = HashedNodeRef(r, PhantomData);
        if r.get_component::<TIdN::Ty>().is_err() {
            return None;
        }
        Some(unsafe { TIdN::from_id(id.clone()) })
    }
}

impl NodeStore {
    pub fn resolve<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: NodeIdentifier,
    ) -> <Self as crate::types::TyNodeStore<TIdN>>::R<'_> {
        let r = self.nodes.get(&id.as_id()).unwrap();
        let r: HashedNodeRef<<TIdN as NodeId>::IdN> = HashedNodeRef(r, PhantomData);
        assert!(r.get_component::<TIdN::Ty>().is_ok());
        HashedNodeRef(r.0, PhantomData)
    }
}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}
