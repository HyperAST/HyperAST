use std::{fmt::Debug, hash::Hash, num::NonZeroU64};

use crate::{
    filter::BloomResult,
    hashed::SyntaxNodeHashsKinds,
    impact::serialize::{Keyed, MySerialize},
    nodes::{CompressedNode, HashSize, RefContainer},
    store::defaults::LabelIdentifier,
    types::{HyperType, MySlice, NodeId, Typed, TypedNodeId},
};

pub type NodeIdentifier = NonZeroU64;

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
}

pub struct HashedNodeRef<'a, Id: TypedNodeId<IdN = NodeIdentifier>> {
    id: Id::IdN,
    ty: Id::Ty,
    label: Option<LabelIdentifier>,
    children: &'a [Id::IdN],
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> PartialEq for HashedNodeRef<'a, Id> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Eq for HashedNodeRef<'a, Id> {}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Hash for HashedNodeRef<'a, Id> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Debug for HashedNodeRef<'a, Id>
where
    Id::Ty: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedNodeRef")
            .field("id", &self.id)
            .field("ty", &self.ty)
            .field("label", &self.label)
            .field("children", &self.children)
            .finish()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Typed for HashedNodeRef<'a, Id>
where
    Id::Ty: Copy + Hash + Eq,
{
    type Type = Id::Ty;

    fn get_type(&self) -> Id::Ty {
        self.ty
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithStats for HashedNodeRef<'a, Id> {
    fn size(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize {
        todo!()
    }
}
impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithSerialization
    for HashedNodeRef<'a, Id>
{
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Node for HashedNodeRef<'a, Id> {}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Stored for HashedNodeRef<'a, Id> {
    type TreeId = Id;
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithChildren
    for HashedNodeRef<'a, Id>
{
    type ChildIdx = u16;
    type Children<'b> = MySlice<<Self::TreeId as NodeId>::IdN> where Self: 'b;

    fn child_count(&self) -> Self::ChildIdx {
        todo!()
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        todo!()
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        todo!()
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithHashs for HashedNodeRef<'a, Id> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Tree for HashedNodeRef<'a, Id>
where
    Id::Ty: Copy + Hash + Eq,
{
    fn has_children(&self) -> bool {
        todo!()
    }

    fn has_label(&self) -> bool {
        todo!()
    }
}
impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Labeled for HashedNodeRef<'a, Id> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        todo!()
    }
    fn try_get_label(&self) -> Option<&Self::Label> {
        todo!()
    }
}
impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> RefContainer for HashedNodeRef<'a, Id> {
    type Result = BloomResult;

    fn check<U: MySerialize + Keyed<usize>>(&self, rf: U) -> Self::Result {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id>
where
    Id::Ty: HyperType,
{
    pub fn is_directory(&self) -> bool {
        self.get_type().is_directory()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id> {
    // // pub(crate) fn new(entry: EntryRef<'a>) -> Self {
    // //     Self(entry)
    // // }

    // /// Returns the entity's archetype.
    // pub fn archetype(&self) -> &Archetype {
    //     self.0.archetype()
    // }

    // /// Returns the entity's location.
    // pub fn location(&self) -> EntityLocation {
    //     self.0.location()
    // }

    // /// Returns a reference to one of the entity's components.
    // pub fn into_component<T: Component>(self) -> Result<&'a T, ComponentError> {
    //     self.0.into_component::<T>()
    // }

    // /// Returns a mutable reference to one of the entity's components.
    // ///
    // /// # Safety
    // /// This function bypasses static borrow checking. The caller must ensure that the component reference
    // /// will not be mutably aliased.
    // pub unsafe fn into_component_unchecked<T: Component>(
    //     self,
    // ) -> Result<&'a mut T, ComponentError> {
    //     self.0.into_component_unchecked::<T>()
    // }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<C>(&self) -> Result<&C, String> {
        todo!()
    }

    // /// Returns a mutable reference to one of the entity's components.
    // ///
    // /// # Safety
    // /// This function bypasses static borrow checking. The caller must ensure that the component reference
    // /// will not be mutably aliased.
    // pub unsafe fn get_component_unchecked<T: Component>(&self) -> Result<&mut T, ComponentError> {
    //     self.0.get_component_unchecked::<T>()
    // }

    pub fn into_compressed_node(
        &self,
    ) -> Result<CompressedNode<NodeIdentifier, LabelIdentifier, Id::Ty>, String> {
        todo!()
    }

    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn get_bytes_len(&self, _p_indent_len: u32) -> u32 {
        todo!()
    }

    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn try_get_bytes_len(&self, _p_indent_len: u32) -> Option<u32> {
        todo!()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id> {
    pub fn get_child_by_name(
        &self,
        name: &<HashedNodeRef<'a, Id> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a, Id> as crate::types::Stored>::TreeId> {
        todo!()
    }

    pub fn get_child_idx_by_name(
        &self,
        name: &<HashedNodeRef<'a, Id> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a, Id> as crate::types::WithChildren>::ChildIdx> {
        todo!()
    }

    pub fn try_get_children_name(
        &self,
    ) -> Option<&[<HashedNodeRef<'a, Id> as crate::types::Labeled>::Label]> {
        todo!()
    }
}

pub struct NodeStore {}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    type R<'a> = HashedNodeRef<'a, NodeIdentifier>; // TODO
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        todo!()
    }
}

impl<TIdN: TypedNodeId<IdN = NodeIdentifier>> crate::types::TypedNodeStore<TIdN> for NodeStore {
    type R<'a> = HashedNodeRef<'a, TIdN>; // TODO
    fn resolve(&self, id: &TIdN) -> Self::R<'_> {
        todo!()
    }

    fn try_typed(&self, id: &<TIdN as NodeId>::IdN) -> Option<TIdN> {
        todo!()
    }
}

impl NodeStore {
    pub fn resolve<TIdN: TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: NodeIdentifier,
    ) -> <Self as crate::types::TypedNodeStore<TIdN>>::R<'_> {
        todo!()
    }
}

impl NodeStore {
    pub fn len(&self) -> usize {
        todo!()
    }
}

impl NodeStore {
    pub fn new() -> Self {
        todo!()
    }
}
