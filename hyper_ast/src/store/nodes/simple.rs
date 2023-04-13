use std::{fmt::Debug, hash::Hash, num::NonZeroU64};

use crate::{
    hashed::SyntaxNodeHashsKinds,
    nodes::{CompressedNode, HashSize, RefContainer},
    store::defaults::LabelIdentifier,
    types::{MySlice, Type, Typed}, filter::BloomResult, impact::serialize::{MySerialize, Keyed},
};

pub type NodeIdentifier = NonZeroU64;

pub struct HashedNodeRef<'a> {
    id: NodeIdentifier,
    ty: Type,
    label: Option<LabelIdentifier>,
    children: &'a [NodeIdentifier],
}

impl<'a> PartialEq for HashedNodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> Eq for HashedNodeRef<'a> {}

impl<'a> Hash for HashedNodeRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl<'a> Debug for HashedNodeRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedNodeRef")
            .field("id", &self.id)
            .field("ty", &self.ty)
            .field("label", &self.label)
            .field("children", &self.children)
            .finish()
    }
}

impl<'a> crate::types::Typed for HashedNodeRef<'a> {
    type Type = Type;

    fn get_type(&self) -> Type {
        self.ty
    }
}

impl<'a> crate::types::WithStats for HashedNodeRef<'a> {
    fn size(&self) -> usize {
        todo!()
    }

    fn height(&self) -> usize {
        todo!()
    }
}
impl<'a> crate::types::WithSerialization for HashedNodeRef<'a> {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}

impl<'a> crate::types::Node for HashedNodeRef<'a> {}

impl<'a> crate::types::Stored for HashedNodeRef<'a> {
    type TreeId = NodeIdentifier;
}

impl<'a> crate::types::WithChildren for HashedNodeRef<'a> {
    type ChildIdx = u16;
    type Children<'b> = MySlice<Self::TreeId> where Self: 'b;

    fn child_count(&self) -> Self::ChildIdx {
        todo!()
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        todo!()
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        todo!()
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        todo!()
    }
}

impl<'a> crate::types::WithHashs for HashedNodeRef<'a> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        todo!()
    }
}

impl<'a> crate::types::Tree for HashedNodeRef<'a> {
    fn has_children(&self) -> bool {
        todo!()
    }

    fn has_label(&self) -> bool {
        todo!()
    }
}
impl<'a> crate::types::Labeled for HashedNodeRef<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        todo!()
    }
}
impl<'a> RefContainer for HashedNodeRef<'a> {
    type Result = BloomResult;

    fn check<U: MySerialize + Keyed<usize>>(&self, rf: U) -> Self::Result {
        todo!()
    }
}

impl<'a> HashedNodeRef<'a> {
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
    pub fn get_component<T>(&self) -> Result<&T, String> {
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
    ) -> Result<CompressedNode<NodeIdentifier, LabelIdentifier>, String> {
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

    pub fn is_directory(&self) -> bool {
        self.get_type().is_directory()
    }

    pub fn get_child_by_name(
        &self,
        name: &<HashedNodeRef<'a> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a> as crate::types::Stored>::TreeId> {
        todo!()
    }

    pub fn get_child_idx_by_name(
        &self,
        name: &<HashedNodeRef<'a> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a> as crate::types::WithChildren>::ChildIdx> {
        todo!()
    }

    pub fn try_get_children_name(
        &self,
    ) -> Option<&[<HashedNodeRef<'a> as crate::types::Labeled>::Label]> {
        todo!()
    }
}

pub struct NodeStore {}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    type R<'a> = HashedNodeRef<'a>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        todo!()
    }
}

impl NodeStore {
    pub fn resolve(
        &self,
        id: NodeIdentifier,
    ) -> <Self as crate::types::NodeStore<NodeIdentifier>>::R<'_> {
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
