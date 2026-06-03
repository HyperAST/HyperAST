use std::{hash::Hash, marker::Send, ops::Deref};

use num::ToPrimitive;

use crate::{
    hashed::{SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::HashSize,
    store::defaults::LabelIdentifier,
    types::*,
};

use super::compo;

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
    type TyErazed = crate::types::AnyType;

    fn unerase(ty: Self::TyErazed) -> Self::Ty {
        ty
    }
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

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> std::hash::Hash for HashedNodeRef<'a, Id> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        WithHashs::hash(self, &Default::default()).hash(state)
    }
}

impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::Typed
    for HashedNodeRef<'a, Id>
where
    Id::Ty: Copy + Hash + Eq + hecs::ComponentRef<'a>,
{
    type Type = Id::Ty;

    fn get_type(&self) -> Id::Ty {
        let t = self.0.get::<&Id::Ty>().unwrap();
        *t
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithStats for HashedNodeRef<'a, Id> {
    fn size(&self) -> usize {
        self.0
            .get::<&compo::Size>()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }

    fn height(&self) -> usize {
        self.0
            .get::<&compo::Height>()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }

    fn line_count(&self) -> usize {
        self.0
            .get::<&compo::LineCount>()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }
}
impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithSerialization
    for HashedNodeRef<'a, Id>
{
    fn try_bytes_len(&self) -> Option<usize> {
        self.0
            .get::<&compo::BytesLen>()
            .and_then(|x| x.0.to_usize())
    }
}

impl<'a, T> HashedNodeRef<'a, T> {
    pub fn size_no_spaces(&self) -> usize {
        self.0
            .get::<&compo::SizeNoSpaces>()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Node for HashedNodeRef<'a, Id> {}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Stored for HashedNodeRef<'a, Id> {
    type TreeId = Id;
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id> {
    pub fn cs(&self) -> Option<crate::types::LendC<'_, Self, u16, NodeIdentifier>> {
        todo!()
        // self.0
        //     .get::<&'a compo::CS<NodeIdentifier>>()
        //     .map(|x| ChildrenSlice(x))
    }
    // pub fn no_spaces(&self) -> Option<&<Self as crate::types::WithChildren>::Children<'_>> {
    //     self.0
    //         .get::<&'a compo::NoSpacesCS<NodeIdentifier>>()
    //         .map(|x| x)
    //         .or_else(|| self.0.get::<&compo::CS<NodeIdentifier>>().map(|x| x))
    //         .map(|x| ChildrenSlice(x))
    // }
}

#[derive(Clone)]
pub struct ChildrenSlice<'a, IdN: Send + Sync + Eq + 'static>(hecs::Ref<'a, compo::CS<IdN>>);

#[derive(Clone)]
pub struct ChildIter<'a, 'b, IdN: Send + Sync + Eq + 'static> {
    index: usize,
    children: &'b ChildrenSlice<'a, IdN>,
}

impl<'a, 'b, IdN: Send + Sync + Eq + 'static> Iterator for ChildIter<'a, 'b, IdN> {
    type Item = &'b IdN;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.children.0 .0.len() {
            let r = &self.children.0 .0[self.index];
            self.index += 1;
            Some(r)
        } else {
            None
        }
    }
}

// impl<'b, T: Send + Sync + Eq + Clone + 'static> IterableChildren<T> for ChildrenSlice<'b, T> {
//     type ChildrenIter<'a>
//         = ChildIter<'b, 'a, T>
//     where
//         T: 'a,
//         Self: 'a;

//     fn iter_children(&self) -> Self::ChildrenIter<'_> {
//         ChildIter {
//             index: 0,
//             children: self,
//         }
//     }

//     fn is_empty(&self) -> bool {
//         self.0 .0.is_empty()
//     }
// }

impl<'b, T: Send + Sync + Eq + 'static> std::ops::Index<u16> for ChildrenSlice<'b, T> {
    type Output = T;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0 .0.deref()[index as usize]
    }
}

impl<'b, T: Send + Sync + Eq + 'static> Iterator for ChildrenSlice<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}


impl<'b, T: Send + Sync + Eq + Clone + 'static> Childrn< T> for ChildrenSlice<'b, T> {
    fn len(&self) -> usize {
        self.0 .0.deref().len()
    }
    fn is_empty(&self) -> bool {
        unimplemented!("cannot be implemented on ChildrenSlice")
    }
}
impl<'b, T: Send + Sync + Eq + Clone + 'static> Children<u16, T> for ChildrenSlice<'b, T> {
    fn child_count(&self) -> u16 {
        self.0 .0.deref().len().to_u16().unwrap()
    }

    fn get(&self, i: u16) -> Option<&T> {
        self.0 .0.deref().get(usize::from(i))
    }

    fn rev(&self, idx: u16) -> Option<&T> {
        let c: u16 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, _i: u16) -> Self {
        unimplemented!("cannot be implemented on ChildrenSlice")
        // (&self.0.0.deref()[i.into()..]).into()
    }

    fn before(&self, _i: u16) -> Self {
        unimplemented!("cannot be implemented on ChildrenSlice")
        // (&self.0.0.deref()[..i.into()]).into()
    }

    fn between(&self, _start: u16, _end: u16) -> Self {
        unimplemented!("cannot be implemented on ChildrenSlice")
        // (&self.0.0.deref()[start.into()..end.into()]).into()
    }

    fn inclusive(&self, _start: u16, _end: u16) -> Self {
        unimplemented!("cannot be implemented on ChildrenSlice")
        // (&self.0.0.deref()[start.into()..=end.into()]).into()
    }

    fn iter_children(&self) -> Self {
        unimplemented!("cannot be implemented on ChildrenSlice")
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::CLending<'a, u16, NodeIdentifier>
    for HashedNodeRef<'_, Id>
{
    type Children = crate::types::ChildrenSlice<'a, NodeIdentifier>;
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithChildren
    for HashedNodeRef<'a, Id>
{
    type ChildIdx = u16;
    // type Children<'b> = ChildrenSlice<'b,<Self::TreeId as NodeId>::IdN> where Self: 'b;

    fn child_count(&self) -> Self::ChildIdx {
        self.cs()
            .map_or(0, |x| {
                let c: u16 = x.child_count();
                c
            })
            .to_u16()
            .expect("too much children")
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        todo!()
        // self.cs()
        //     .unwrap_or_else(|| {
        //         log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
        //         panic!()
        //     })
        //     .0
        //      .0
        //     .get(idx.to_usize().unwrap())
        //     .map(|x| *x)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        let v = self.cs()?;
        let c: Self::ChildIdx = v.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        v.get(c).cloned()
    }

    fn children(&self) -> Option<LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>> {
        todo!() // perdu
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::WithHashs for HashedNodeRef<'a, Id> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        use crate::hashed::NodeHashs;
        self.0
            .get::<&SyntaxNodeHashs<Self::HP>>()
            .unwrap()
            .deref()
            .hash(kind)
    }
}

impl<'a, Id> crate::types::ErasedHolder for HashedNodeRef<'a, Id> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        todo!()
    }
}

impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::Tree
    for HashedNodeRef<'a, Id>
where
    Id::Ty: Copy + Hash + Eq,
{
    fn has_children(&self) -> bool {
        self.cs().map(|x| !crate::types::Childrn::is_empty(&x)).unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get::<&LabelIdentifier>().is_some()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Labeled for HashedNodeRef<'a, Id> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        todo!("API changes needed")
        // TODO it shows issues with the current API, i.e. should use GAT
        // self.0
        //     .get::<&LabelIdentifier>()
        //     .expect("check with self.has_label()")
        //     .deref()
    }
    fn try_get_label(&self) -> Option<&Self::Label> {
        todo!("API changes needed")
        // self.0.get::<&LabelIdentifier>().as_deref()
    }
}
