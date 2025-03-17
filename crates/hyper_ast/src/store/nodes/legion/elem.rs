use std::{fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref};

use legion::{
    storage::{Archetype, Component},
    world::{ComponentError, EntityLocation},
};
use num::ToPrimitive;

use crate::{
    filter::{Bloom, BloomResult, BloomSize, BF},
    hashed::{NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    impact::serialize::{CachedHasher, Keyed, MySerialize},
    nodes::{CompressedNode, HashSize, RefContainer},
    store::defaults::LabelIdentifier,
    types::{
        AnyType, Children, HyperType, NodeId, TypeTrait, Typed, TypedNodeId, WithChildren,
        WithMetaData,
    },
};

use super::compo::{self, NoSpacesCS, CS};

pub type NodeIdentifier = legion::Entity;
pub type EntryRef<'a> = legion::world::EntryRef<'a>;
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct HashedNodeRef<'a, T = NodeIdentifier>(pub(super) EntryRef<'a>, PhantomData<T>);

impl crate::types::AAAA for NodeIdentifier {}

impl<'a, T> HashedNodeRef<'a, T> {
    #[doc(hidden)]
    pub fn cast_type<U: NodeId>(self) -> HashedNodeRef<'a, U>
    where
        T: NodeId<IdN = U::IdN>,
    {
        HashedNodeRef(self.0, PhantomData)
    }
    pub(super) fn new(e: EntryRef<'a>) -> Self {
        Self(e, PhantomData)
    }
}
impl<'a, T> From<&'a EntryRef<'a>> for &'a HashedNodeRef<'a, T> {
    fn from(value: &'a EntryRef<'a>) -> Self {
        use ref_cast::RefCast;
        // NOTE it makes compile time layout assertions
        HashedNodeRef::ref_cast(value)
    }
}

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

pub struct HashedNode<Id: TypedNodeId<IdN = NodeIdentifier>> {
    node: CompressedNode<NodeIdentifier, LabelIdentifier, Id::Ty>,
    hashs: SyntaxNodeHashs<u32>,
}

// impl<'a> Symbol<HashedNodeRef<'a>> for legion::Entity {}

// * hashed node impl

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> PartialEq for HashedNode<Id>
where
    Id::IdN: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Eq for HashedNode<Id> where Id::IdN: Eq {}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Hash for HashedNode<Id> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state)
    }
}
// impl<'a, Id: TypedNodeId<IdN = NodeIdentifier, Ty=Type>> crate::types::Typed for HashedNode<Id> {
//     type Type = Id::Ty;

//     fn get_type(&self) -> Type {
//         panic!()
//     }
// }

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Labeled for HashedNode<Id> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        panic!()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        todo!()
        // .or_else(|| {
        //     let a = self.0.get_component::<Box<[Space]>>();
        //     let mut b = String::new();
        //     a.iter()
        //         .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());

        // })
    }
}

// impl<'a,T> crate::types::WithChildren for HashedNode<T> {
//     type ChildIdx = u16;

//     fn child_count(&self) -> Self::ChildIdx {
//         todo!()
//     }

//     fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
//         todo!()
//     }

//     fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
//         todo!()
//     }

//     fn get_children(&self) -> &[Self::TreeId] {
//         todo!()
//     }

//     fn get_children_cpy(&self) -> Vec<Self::TreeId> {
//         todo!()
//     }

//     fn try_get_children(&self) -> Option<&[Self::TreeId]> {
//         todo!()
//     }
// }

// impl<'a,T> crate::types::Tree for HashedNode<T> {
//     fn has_children(&self) -> bool {
//         todo!()
//     }

//     fn has_label(&self) -> bool {
//         todo!()
//     }
// }

// impl Symbol<HashedNode> for legion::Entity {}

// * hashed node reference impl

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> PartialEq for HashedNodeRef<'a, Id> {
    fn eq(&self, other: &Self) -> bool {
        self.0.location().archetype() == other.0.location().archetype()
            && self.0.location().component() == other.0.location().component()
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Eq for HashedNodeRef<'a, Id> {}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Hash for HashedNodeRef<'a, Id> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        crate::types::WithHashs::hash(self, SyntaxNodeHashsKinds::default()).hash(state)
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> Debug for HashedNodeRef<'a, Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HashedNodeRef")
            .field(&self.0.location())
            .finish()
    }
}

impl<'a, Id> Deref for HashedNodeRef<'a, Id> {
    type Target = EntryRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id> {
    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn get_bytes_len(&self, _p_indent_len: u32) -> u32
    where
        Id::Ty: 'static + TypeTrait + Send + Sync + Debug,
    {
        // use crate::types::Typed;
        // if self.get_type().is_spaces() {
        //     self.0
        //         .get_component::<compo::BytesLen>()
        //         .expect(&format!(
        //             "node with type {:?} don't have a len",
        //             self.get_type()
        //         ))
        //         .0
        //     // self.get_component::<Box<[Space]>>()
        //     //     .expect("spaces node should have spaces")
        //     //     .iter()
        //     //     .map(|x| {
        //     //         if x == &Space::ParentIndentation {
        //     //             p_indent_len
        //     //         } else {
        //     //             1
        //     //         }
        //     //     })
        //     //     .sum()
        // } else {
        //     self.0
        //         .get_component::<compo::BytesLen>()
        //         .expect(&format!(
        //             "node with type {:?} don't have a len",
        //             self.get_type()
        //         ))
        //         .0
        // }
        // .map_or_else(|_| self
        //     .get_type().to_string().len() as u32,|x|x.0)
        self.0.get_component::<compo::BytesLen>().unwrap().0
    }
}

impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id> {
    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn try_get_bytes_len(&self, _p_indent_len: u32) -> Option<u32>
    where
        Id::Ty: HyperType + Copy + Send + Sync,
    {
        // use crate::types::Typed;
        if self.get_type().is_spaces() {
            self.0.get_component::<compo::BytesLen>().map(|x| x.0).ok()
            // let s = self.get_component::<Box<[Space]>>().ok()?;
            // let s = s
            //     .iter()
            //     .map(|x| {
            //         if x == &Space::ParentIndentation {
            //             p_indent_len
            //         } else {
            //             1
            //         }
            //     })
            //     .sum();
            // Some(s)
        } else {
            self.0.get_component::<compo::BytesLen>().map(|x| x.0).ok()
        }
        // .map_or_else(|_| self
        //     .get_type().to_string().len() as u32,|x|x.0)
    }

    pub fn is_directory(&self) -> bool
    where
        Id: HyperType + Copy + Send + Sync,
    {
        self.get_type().is_directory()
    }
}

impl<'a, T, C: Component> WithMetaData<C> for HashedNodeRef<'a, T> {
    fn get_metadata(&self) -> Option<&C> {
        self.0.get_component::<C>().ok()
    }
}

impl<'a, T> HashedNodeRef<'a, T> {
    // pub(crate) fn new(entry: EntryRef<'a>) -> Self {
    //     Self(entry)
    // }

    /// Returns the entity's archetype.
    pub fn archetype(&self) -> &Archetype {
        self.0.archetype()
    }

    /// Returns the entity's location.
    pub fn location(&self) -> EntityLocation {
        self.0.location()
    }

    /// Returns a reference to one of the entity's components.
    pub fn into_component<C: Component>(self) -> Result<&'a C, ComponentError> {
        self.0.into_component::<C>()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn into_component_unchecked<C: Component>(
        self,
    ) -> Result<&'a mut C, ComponentError> {
        self.0.into_component_unchecked::<C>()
    }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<C: Component>(&self) -> Result<&C, ComponentError> {
        self.0.get_component::<C>()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn get_component_unchecked<C: Component>(&self) -> Result<&mut C, ComponentError> {
        self.0.get_component_unchecked::<C>()
    }
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, T> {
    pub fn get_child_by_name(
        &self,
        name: &<HashedNodeRef<'a, T> as crate::types::Labeled>::Label,
    ) -> Option<NodeIdentifier> {
        let labels = self
            .0
            .get_component::<CS<<HashedNodeRef<'a, T> as crate::types::Labeled>::Label>>()
            .ok()?;
        let idx = labels.0.iter().position(|x| x == name);
        idx.map(|idx| self.child(&idx.to_u16().unwrap()).unwrap())
    }

    pub fn get_child_idx_by_name(
        &self,
        name: &<HashedNodeRef<'a, T> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a, T> as crate::types::WithChildren>::ChildIdx> {
        let labels = self
            .0
            .get_component::<CS<<HashedNodeRef<'a, T> as crate::types::Labeled>::Label>>()
            .ok()?;
        labels
            .0
            .iter()
            .position(|x| x == name)
            .map(|x| x.to_u16().unwrap())
    }

    pub fn try_get_children_name(
        &self,
    ) -> Option<&[<HashedNodeRef<'a, T> as crate::types::Labeled>::Label]> {
        self.0
            .get_component::<CS<<HashedNodeRef<'a, T> as crate::types::Labeled>::Label>>()
            .ok()
            .map(|x| &*x.0)
    }
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, Id>
where
    Id::Ty: 'static + Sync + Send + TypeTrait,
{
    pub fn into_compressed_node(
        &self,
    ) -> Result<CompressedNode<legion::Entity, LabelIdentifier, Id::Ty>, ComponentError> {
        // if let Ok(spaces) = self.0.get_component::<Box<[Space]>>() {
        //     return Ok(CompressedNode::Spaces(spaces.clone()));
        // }
        let kind = self.0.get_component::<Id::Ty>()?;
        if kind.is_spaces() {
            let spaces = self.0.get_component::<LabelIdentifier>().unwrap();
            return Ok(CompressedNode::Spaces(spaces.clone()));
        }
        let a = self.0.get_component::<LabelIdentifier>();
        let label: Option<LabelIdentifier> = a.ok().map(|x| x.clone());
        let children = self.children().map(|mut x| x.map(|x| x.clone()).collect());
        // .0.get_component::<CS<legion::Entity>>();
        // let children = children.ok().map(|x| x.0.clone());
        Ok(CompressedNode::new(
            *kind,
            label,
            children.unwrap_or_default(),
        ))
    }
}

impl<'a, T> AsRef<HashedNodeRef<'a, T>> for HashedNodeRef<'a, T> {
    fn as_ref(&self) -> &HashedNodeRef<'a, T> {
        self
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
        match self.0.get_component::<Id::TyErazed>() {
            Ok(t) => Id::unerase(t.clone()),
            e => Id::unerase(e.unwrap().clone()),
            // Err(ComponentError::NotFound {..}) => {
            //     let type_type = self.0.archetype().layout().component_types()[0];
            //     self.0.
            //     todo!()
            // }
        }
    }
    fn try_get_type(&self) -> Option<Self::Type> {
        self.0.get_component::<Id::Ty>().ok().copied()
    }
}
impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::Typed
    for &HashedNodeRef<'a, Id>
{
    type Type = AnyType;

    fn get_type(&self) -> AnyType {
        match self.0.get_component::<Id::Ty>() {
            Ok(t) => {
                let t: &'static dyn HyperType = t.as_static();
                t.into()
            }
            Err(e @ ComponentError::NotFound { .. }) => {
                todo!("{:?}", e)
            }
            e => {
                todo!("{:?}", e)
            }
        }
    }
}

impl<'a, T> crate::types::WithStats for HashedNodeRef<'a, T> {
    fn size(&self) -> usize {
        self.0
            .get_component::<compo::Size>()
            .ok()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }

    fn height(&self) -> usize {
        self.0
            .get_component::<compo::Height>()
            .ok()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }

    fn line_count(&self) -> usize {
        self.0
            .get_component::<compo::LineCount>()
            .ok()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(0)
    }
}

impl<'a, T> HashedNodeRef<'a, T> {
    pub fn size_no_spaces(&self) -> usize {
        self.0
            .get_component::<compo::SizeNoSpaces>()
            .ok()
            .and_then(|x| x.0.to_usize())
            .unwrap_or(1)
    }
}

impl<'a, T> crate::types::WithSerialization for HashedNodeRef<'a, T> {
    fn try_bytes_len(&self) -> Option<usize> {
        self.0
            .get_component::<compo::BytesLen>()
            .ok()
            .map(|x| x.0.to_usize().unwrap())
    }
}

impl<'a, T> crate::types::Labeled for HashedNodeRef<'a, T> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        self.0
            .get_component::<LabelIdentifier>()
            .expect("check with self.has_label()")
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.0.get_component::<LabelIdentifier>().ok()
        // .or_else(|| {
        //     let a = self.0.get_component::<Box<[Space]>>();
        //     let mut b = String::new();
        //     a.iter()
        //         .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());

        // })
    }
}

impl<'a, T> crate::types::Node for HashedNodeRef<'a, T> {}

impl<'a, T: crate::types::NodeId> crate::types::Stored for HashedNodeRef<'a, T> {
    type TreeId = T;
}

impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Node for HashedNode<Id> {}
impl<'a, Id: TypedNodeId<IdN = NodeIdentifier>> crate::types::Stored for HashedNode<Id> {
    type TreeId = Id::IdN;
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, T> {
    pub fn cs(&self) -> Result<crate::types::LendC<'_, Self, u16, NodeIdentifier>, ComponentError> {
        // let scount = self.0.get_component::<CSStaticCount>().ok();
        // if let Some(CSStaticCount(scount)) = scount {
        // if *scount == 1 {
        //     self.0
        //         .get_component::<CS0<NodeIdentifier, 1>>()
        //         .map(|x| x.into())
        //     } else if *scount == 2 {
        //         self.0
        //             .get_component::<CS0<NodeIdentifier, 2>>()
        //             .map(|x| x.into())
        //     } else
        // if *scount == 3 {
        //     self.0
        //         .get_component::<CS0<NodeIdentifier, 3>>()
        //         .map(|x| x.into())
        // } else {
        //     panic!()
        // }
        // } else {
        let r = self
            .0
            .get_component::<CS<NodeIdentifier>>()
            .map(|x| (*x.0).into())
            .or_else(|_| {
                self.0
                    .get_component::<compo::CS0<NodeIdentifier, 1>>()
                    .map(|x| (&x.0).into())
            })
            .or_else(|_| {
                self.0
                    .get_component::<compo::CS0<NodeIdentifier, 2>>()
                    .map(|x| (&x.0).into())
            });
        r
        // }
    }
    pub fn no_spaces(&self) -> Result<crate::types::LendC<'_, Self, u16, T::IdN>, ComponentError> {
        self.0
            .get_component::<NoSpacesCS<NodeIdentifier>>()
            .map(|x| (*x.0).into())
            .or_else(|_| {
                self.0
                    .get_component::<compo::NoSpacesCS0<NodeIdentifier, 1>>()
                    .map(|x| (&x.0).into())
            })
            .or_else(|_| {
                self.0
                    .get_component::<compo::NoSpacesCS0<NodeIdentifier, 2>>()
                    .map(|x| (&x.0).into())
            })
            .or_else(|_| self.cs())
    }
}

impl<'a, T: crate::types::NodeId> crate::types::CLending<'a, u16, T::IdN> for HashedNodeRef<'_, T> {
    type Children = crate::types::ChildrenSlice<'a, T::IdN>;
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> crate::types::WithChildren
    for HashedNodeRef<'a, T>
{
    type ChildIdx = u16;
    // type Children<'b>
    //     = MySlice<Self::TreeId>
    // where
    //     Self: 'b;

    fn child_count(&self) -> u16 {
        self.cs()
            .map_or(0, |x| {
                let c: u16 = x.child_count();
                c
            })
            .to_u16()
            .expect("too much children")
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<NodeIdentifier> {
        self.cs().ok()?
            // .unwrap_or_else(|x| {
            //     log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
            //     panic!("{}", x)
            // })
            .0
            .get(idx.to_usize().unwrap())
            .map(|x| *x)
        // .unwrap_or_else(|| {
        //     log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
        //     panic!()
        // })
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<NodeIdentifier> {
        let v = self.cs().ok()?;
        // .unwrap_or_else(|x| {
        //     log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
        //     panic!("{}", x)
        // });
        // v.0.get(v.len() - 1 - num::cast::<_, usize>(*idx).unwrap()).cloned()
        let c: Self::ChildIdx = v.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        v.get(c).cloned()
    }

    // unsafe fn children_unchecked<'b>(&'b self) -> &'b [Self::TreeId] {
    //     let cs = self.cs().unwrap_or_else(|x| {
    //         log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
    //         panic!("{}", x)
    //     });
    //     cs
    // }

    // fn get_children_cpy<'b>(&'b self) -> Vec<Self::TreeId> {
    //     let cs = self.cs().unwrap_or_else(|x| {
    //         log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
    //         panic!("{}", x)
    //     });
    //     cs.to_vec()
    // }

    fn children(
        &self,
    ) -> Option<crate::types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>> {
        self.cs().ok()
    }
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> crate::types::WithRoles
    for HashedNodeRef<'a, T>
{
    fn role_at<Role: 'static + Copy + std::marker::Sync + std::marker::Send>(
        &self,
        at: Self::ChildIdx,
    ) -> Option<Role> {
        let ro = self.0.get_component::<compo::RoleOffsets>().ok()?;
        let r = self.0.get_component::<Box<[Role]>>().ok()?;
        let mut i = 0;
        for &ro in ro.0.as_ref() {
            if ro as u16 > at {
                return None;
            } else if ro as u16 == at {
                return Some(r[i]);
            }
            i += 1;
        }
        None
    }
}

impl<'a, T> crate::types::WithPrecompQueries for HashedNodeRef<'a, T> {
    fn wont_match_given_precomputed_queries(&self, needed: u16) -> bool {
        if needed == num::zero() {
            return false;
        }
        let Ok(v) = self.get_component::<compo::Precomp<u16>>() else {
            return self.get_component::<compo::PrecompFlag>().is_ok();
        };
        v.0 & needed != needed
    }
}

impl<'a, T> crate::types::WithHashs for HashedNodeRef<'a, T> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP {
        self.0
            .get_component::<SyntaxNodeHashs<Self::HP>>()
            .unwrap()
            .hash(&kind)
    }
}

impl<'a, Id> crate::types::ErasedHolder for HashedNodeRef<'a, Id> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            self.get_component().ok()
        } else {
            None
        }
    }
}

impl<'a, Id: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::Tree
    for HashedNodeRef<'a, Id>
{
    fn has_children(&self) -> bool {
        self.cs()
            .map(|x| !crate::types::Childrn::is_empty(&x))
            .unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get_component::<LabelIdentifier>().is_ok()
    }
}

impl<'a, T> HashedNodeRef<'a, T> {}

impl<'a, T> RefContainer for HashedNodeRef<'a, T> {
    type Result = BloomResult;

    fn check<U: MySerialize + Keyed<usize>>(&self, rf: U) -> Self::Result {
        use crate::filter::BF as _;

        let Ok(e) = self.0.get_component::<BloomSize>() else {
            return BloomResult::MaybeContain;
        };
        macro_rules! check {
        ( $($t:ty),* ) => {
            match *e {
                BloomSize::Much => {
                    log::trace!("[Too Much]");
                    BloomResult::MaybeContain
                },
                BloomSize::None => BloomResult::DoNotContain,
                $( <$t>::SIZE => {
                    let x = CachedHasher::<usize,<$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::once(rf);
                    let x = x.into_iter().map(|x|<$t>::check_raw(self.0.get_component::<$t>().unwrap(), x));

                    for x in x {
                        if let BloomResult::MaybeContain = x {
                            return BloomResult::MaybeContain
                        }
                    }
                    BloomResult::DoNotContain
                }),*
            }
        };
    }
        check![
            Bloom<&'static [u8], u16>,
            Bloom<&'static [u8], u32>,
            Bloom<&'static [u8], u64>,
            Bloom<&'static [u8], [u64; 2]>,
            Bloom<&'static [u8], [u64; 4]>,
            Bloom<&'static [u8], [u64; 8]>,
            Bloom<&'static [u8], [u64; 16]>,
            Bloom<&'static [u8], [u64; 32]>,
            Bloom<&'static [u8], [u64; 64]>
        ]
    }
}
