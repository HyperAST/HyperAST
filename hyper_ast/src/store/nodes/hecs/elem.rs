use crate::types::*;

pub type NodeIdentifier = hecs::Entity;

#[repr(transparent)]
pub struct HashedNodeRef<'a, T = NodeIdentifier>(
    pub(super) hecs::EntityRef<'a>,
    std::marker::PhantomData<T>,
);

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
    type Ty = AnyType;
}

impl<'a, T> HashedNodeRef<'a, T> {
    pub(super) fn new(e: hecs::EntityRef<'a>) -> Self {
        Self(e, std::marker::PhantomData)
    }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<C: hecs::ComponentRef<'a>>(self) -> Option<C::Ref> {
        self.0.get::<C>()
    }
}

mod impl_traits_noderef {
    use super::*;
    use crate::{
        hashed::{SyntaxNodeHashs, SyntaxNodeHashsKinds},
        nodes::HashSize,
        types::TypedNodeId,
    };

    impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> std::hash::Hash for HashedNodeRef<'a, Id> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            WithHashs::hash(self, &Default::default()).hash(state)
        }
    }

    impl<'a, T> WithHashs for HashedNodeRef<'a, T> {
        type HK = SyntaxNodeHashsKinds;
        type HP = HashSize;

        fn hash(&self, kind: &Self::HK) -> Self::HP {
            use crate::hashed::NodeHashs;
            self.0
                .get::<&SyntaxNodeHashs<Self::HP>>()
                .unwrap()
                .hash(kind)
        }
    }

    impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::Typed
        for HashedNodeRef<'a, Id>
    {
        type Type = Id::Ty;

        fn get_type(&self) -> Id::Ty
        where
            Id::Ty: Copy + Send + Sync,
        {
            *self.0.get::<&Id::Ty>().unwrap()
        }
    }
}
