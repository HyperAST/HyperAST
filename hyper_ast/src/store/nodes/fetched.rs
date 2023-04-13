use std::{
    fmt::{Debug, Display},
    hash::Hash,
    num::{NonZeroU32, NonZeroU64},
};

use hashbrown::{HashMap, HashSet};
use string_interner::Symbol;

use crate::{
    filter::BloomResult,
    hashed::SyntaxNodeHashsKinds,
    impact::serialize::{Keyed, MySerialize},
    nodes::{CompressedNode, HashSize, RefContainer},
    store::{defaults, labels::LabelStore},
    types::{self, IterableChildren, LabelStore as _, MySlice, Type, Typed},
};

use strum_macros::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[repr(transparent)]
pub struct LabelIdentifier(string_interner::symbol::SymbolU32);

#[cfg(feature = "serialize")]
impl serde::Serialize for LabelIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0.to_usize() as u32)
    }
}
#[cfg(feature = "serialize")]
impl<'de> serde::Deserialize<'de> for LabelIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = LabelIdentifier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer between -2^31 and 2^31")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(LabelIdentifier(
                    string_interner::symbol::SymbolU32::try_from_usize(value as usize).unwrap(),
                ))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(LabelIdentifier(
                    string_interner::symbol::SymbolU32::try_from_usize(value as usize).unwrap(),
                ))
            }
        }
        deserializer.deserialize_u32(Visitor)
    }
}
impl From<crate::store::defaults::LabelIdentifier> for LabelIdentifier {
    fn from(value: crate::store::defaults::LabelIdentifier) -> Self {
        Self(value)
    }
}
impl From<&crate::store::defaults::LabelIdentifier> for LabelIdentifier {
    fn from(value: &crate::store::defaults::LabelIdentifier) -> Self {
        Self(*value)
    }
}
impl From<LabelIdentifier> for u32 {
    fn from(value: LabelIdentifier) -> Self {
        value.0.to_usize() as u32
    }
}

/// the node identifier for remote hyperASTs
///
/// WARN uses a u32 where legion uses u64
///
/// TODO revamp the default node store, changing the backend (for now legion)
/// with an ecs (in memory col oriented db could also fit) with the following qualities:
/// * archetypes (see. legion and hecs)
/// * append only (see. software heritage)
/// * no indirection to entries from entity other than archetypes
/// * deduplicated entries (and entities) based on selected primary components (like a primary key)
/// * OPTIONAL unique hash index for certain nodes (at least members and equivalent)
/// * OPTIONAL lightweight remote view with a subset of metadata
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct NodeIdentifier(NonZeroU32);
// struct Location {
//     arch: u32,
//     offset: u32,
// }

#[cfg(feature = "legion")]
impl From<crate::store::nodes::legion::NodeIdentifier> for NodeIdentifier {
    fn from(id: crate::store::nodes::legion::NodeIdentifier) -> Self {
        let id: u64 = unsafe { std::mem::transmute(id) };
        // WARN cast to smaller container
        let id = id as u32;
        Self(core::num::NonZeroU32::new(id).unwrap())
    }
}

impl NodeIdentifier {
    // pub fn to_bytes(&self) -> [u8; 8] {
    //     self.into()
    // }
    pub fn to_u32(&self) -> u32 {
        self.0.into()
    }
    pub fn from_u32(value: NonZeroU32) -> Self {
        Self(value)
    }
}
// impl From<&NodeIdentifier> for [u8; 8] {
//     fn from(value: &NodeIdentifier) -> Self {
//         let v: u64 = value.0.into();
//         v.to_le_bytes()
//     }
// }
// impl TryFrom<&[u8]> for NodeIdentifier {
//     type Error = ();

//     fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
//         let bytes: [u8; 8] = value.try_into().map_err(|_| ())?;
//         let value = u64::from_le_bytes(bytes);
//         Ok(NodeIdentifier(NonZeroU64::try_from(value).map_err(|_| ())?))
//     }
// }
impl Display for NodeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Hash for NodeIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// #[cfg(not(feature = "fetched_par"))]
pub type FetchedLabels = HashMap<LabelIdentifier, String>;
// #[cfg(feature = "fetched_par")]
// pub type FetchedLabels = HashMap<LabelIdentifier, String>;

pub struct HashedNodeRef<'a> {
    index: u32,
    s_ref: VariantRef<'a>,
}

// impl<'a> PartialEq for HashedNodeRef<'a> {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl<'a> Eq for HashedNodeRef<'a> {}

// impl<'a> Hash for HashedNodeRef<'a> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         todo!()
//     }
// }

impl<'a> Debug for HashedNodeRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedNodeRef")
            // .field("id", &self.id)
            // .field("ty", &self.ty)
            // .field("label", &self.label)
            // .field("children", &self.children)
            .finish()
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
        match self.s_ref {
            VariantRef::Typed { .. } => None,
            VariantRef::Labeled { .. } => None,
            VariantRef::Children { entities, .. } => {
                Some((&entities.children[self.index as usize][..]).into())
            }
            VariantRef::Both { entities, .. } => {
                Some((&entities.children[self.index as usize][..]).into())
            }
        }
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
        match self.s_ref {
            VariantRef::Typed { .. } => false,
            VariantRef::Labeled { .. } => false,
            VariantRef::Children { .. } => true,
            VariantRef::Both { .. } => true,
        }
    }

    fn has_label(&self) -> bool {
        match self.s_ref {
            VariantRef::Typed { .. } => false,
            VariantRef::Labeled { .. } => true,
            VariantRef::Children { .. } => false,
            VariantRef::Both { .. } => true,
        }
    }
}
impl<'a> crate::types::Labeled for HashedNodeRef<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        match self.s_ref {
            VariantRef::Typed { .. } => panic!(),
            VariantRef::Labeled { entities, .. } => &entities.label[self.index as usize],
            VariantRef::Children { .. } => panic!(),
            VariantRef::Both { entities, .. } => &entities.label[self.index as usize],
        }
    }
}
impl<'a> HashedNodeRef<'a> {
    fn try_get_label(&self) -> Option<&LabelIdentifier> {
        match self.s_ref {
            VariantRef::Typed { .. } => None,
            VariantRef::Labeled { entities, .. } => Some(&entities.label[self.index as usize]),
            VariantRef::Children { .. } => None,
            VariantRef::Both { entities, .. } => Some(&entities.label[self.index as usize]),
        }
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

#[derive(Default)]
pub struct NodeStore {
    // #[cfg(not(feature = "fetched_par"))]
    stockages: HashMap<u32, Variant>,
    // #[cfg(feature = "fetched_par")]
    // stockages: dashmap::DashMap<u32, Variant>,
}
impl Hash for NodeStore {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for r in &self.stockages {
            r.0.hash(state);
            r.1.rev().len().hash(state);
        }
    }
}

macro_rules! variant_store {
    ($id:ty, $rev:ty ; $($( #[$doc:meta] )* $c:ident => { $($d:ident : $e:ty),* $(,)?}),* $(,)?) => {
        mod variants {
            use super::*;

            $(
            $( #[$doc] )*
            #[derive(Clone, Debug, Default)]
            #[cfg_attr(feature = "serialize", derive(serde::Serialize,serde::Deserialize))]
              pub struct $c{
                pub(super) rev: Vec<$rev>,
                $(pub(super) $d: Vec<$e>,)*
            })*
        }
        #[derive(Clone, Debug)]
        #[derive(EnumDiscriminants)]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize,serde::Deserialize))]
        pub enum RawVariant {
            $($c{
                entities: variants::$c,
            },)*
        }
        pub enum Variant {
            $($c{
                index: HashMap<$rev, u32>,
                entities: variants::$c,
            },)*
        }
        pub enum VariantRef<'a> {
            $($c{
                entities: &'a variants::$c,
            },)*
        }
        impl Variant {
            pub fn try_resolve(&self, offset: $rev) -> Option<HashedNodeRef<'_>> {
                Some(match self {$(
                    Variant::$c{entities, index} => HashedNodeRef {
                        index: *index.get(&offset)?,
                        s_ref: VariantRef::$c{entities},
                    },
                )*})
            }
            pub fn index_and_rev_mut(&mut self) -> (&mut HashMap<$rev, u32>, &Vec<$rev>) {
                match self {$(
                    Variant::$c{ index, entities: variants::$c{rev, ..} } => (&mut *index, &*rev),
                )*}
            }
            pub fn rev(&self) -> &Vec<$rev> {
                match self {$(
                    Variant::$c{ entities: variants::$c{rev, ..}, ..} => &rev,
                )*}
            }
            fn extend(&mut self, other: Self) {
                match (self, other) {$(
                    (Variant::$c{ index, entities }, Variant::$c{ entities: other, ..}) => {
                        $(
                            assert_eq!(other.rev.len(), other.$d.len());
                            let mut $d = other.$d.into_iter();
                        )*
                        for k in other.rev.into_iter() {
                            if !index.contains_key(&k) {
                                let i = entities.rev.len();
                                entities.rev.push(k);
                                index.insert(k, i as u32);
                                $(
                                    entities.$d.push($d.next().unwrap());
                                )*
                            }
                        }
                    },
                    )*
                    _=> unreachable!()
                }
            }
            fn extend_raw(&mut self, other: RawVariant) {
                match (self, other) {$(
                    (Variant::$c{ index, entities }, RawVariant::$c{ entities: other, ..}) => {
                        $(
                            assert_eq!(other.rev.len(), other.$d.len());
                            let mut $d = other.$d.into_iter();
                        )*
                        for k in other.rev.into_iter() {
                            if !index.contains_key(&k) {
                                let i = entities.rev.len();
                                entities.rev.push(k);
                                index.insert(k, i as u32);
                                $(
                                    entities.$d.push($d.next().unwrap());
                                )*
                            }
                        }
                    },
                    )*
                    _=> unreachable!()
                }
            }
        }
        impl RawVariant {
            pub fn rev(&self) -> &Vec<$rev> {
                match self {$(
                    RawVariant::$c{ entities: variants::$c{rev, ..}, ..} => &rev,
                )*}
            }
            pub fn kind(&self) -> &Vec<$rev> {
                match self {$(
                    RawVariant::$c{ entities: variants::$c{rev, ..}, ..} => &rev,
                )*}
            }
        }
        impl<'a> crate::types::Typed for HashedNodeRef<'a> {
            type Type = Type;

            fn get_type(&self) -> Type {
                match self.s_ref {$(
                    VariantRef::$c{ entities: variants::$c{kind,..}, ..} => kind[self.index as usize],
                )*}
            }
        }
        impl<'a> crate::types::WithStats for HashedNodeRef<'a> {
            fn size(&self) -> usize {
                match self.s_ref {$(
                    VariantRef::$c{ entities: variants::$c{size,..}, ..} => size[self.index as usize],
                )*}
            }

            fn height(&self) -> usize {
                todo!()
            }
        }
        impl From<RawVariant> for Variant {
            fn from(value: RawVariant) -> Self {
                match value {$(
                    RawVariant::$c{ entities } => {
                        let mut index = HashMap::default();
                        entities.rev.iter().enumerate().for_each(|(i, x)| {
                            index.insert(*x, i as u32);
                        });
                        Variant::$c{ index, entities }
                    },
                )*}
            }
        }
    };
}

variant_store!(NodeIdentifier, NodeIdentifier;
    /// Just a leaf with a type
    Typed => {
        kind: Type,
        size: usize,
    },
    Labeled => {
        kind: Type,
        label: LabelIdentifier,
        size: usize,
    },
    Children => {
        kind: Type,
        children: Vec<NodeIdentifier>,
        size: usize,
    },
    Both => {
        kind: Type,
        children: Vec<NodeIdentifier>,
        label: LabelIdentifier,
        size: usize,
    },
);

#[derive(Default)]
// #[cfg_attr(feature = "serialize", derive(serde::Serialize,serde::Deserialize))]
pub struct DedupPacked {
    label_store: crate::store::labels::LabelStore,
    // TODO maybe use a bloom filter there ? or a specialized u32 hashset
    stockages: HashMap<u32, (HashSet<u32>, RawVariant)>,
}

// impl DedupPacked {
//     pub fn add<T: crate::types::Tree>(&mut self, id: impl Into<NodeIdentifier>, node: T) -> bool {
//         let id: NodeIdentifier = id.into();
//         let loc: Location = id.into();

//         match self.stockages.entry(loc.arch) {
//             hashbrown::hash_map::Entry::Occupied(_) => {
//                 todo!()
//             }
//             hashbrown::hash_map::Entry::Vacant(_) => {
//                 todo!()
//             }
//         }
//     }

//     // pub fn pack(&mut self) -> PackedStore {
//     //     todo!()
//     // }

//     // pub fn unpack(p: PackedStore) -> Self {
//     //     todo!()
//     // }
// }

#[cfg(feature = "serialize")]
impl serde::Serialize for &mut DedupPacked {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
        // serializer.serialize_u32(self.0.to_usize() as u32)
    }
}

// #[derive(Default)]
pub struct SimplePackedBuilder {
    // label_ids: Vec<LabelIdentifier>,
    stockages: HashMap<u32, RawVariant>,
}

impl Default for SimplePackedBuilder {
    fn default() -> Self {
        let mut r = Self {
            stockages: Default::default(),
        };
        let arch = RawVariantDiscriminants::Typed as u32;
        r.stockages.insert(
            arch,
            RawVariant::Typed {
                entities: variants::Typed::default(),
            },
        );
        let arch = RawVariantDiscriminants::Labeled as u32;
        r.stockages.insert(
            arch,
            RawVariant::Labeled {
                entities: variants::Labeled::default(),
            },
        );
        let arch = RawVariantDiscriminants::Children as u32;
        r.stockages.insert(
            arch,
            RawVariant::Children {
                entities: variants::Children::default(),
            },
        );
        let arch = RawVariantDiscriminants::Both as u32;
        r.stockages.insert(
            arch,
            RawVariant::Both {
                entities: variants::Both::default(),
            },
        );
        r
    }
}

impl SimplePackedBuilder {
    pub fn add<T>(&mut self, id: NodeIdentifier, node: T)
    where
        T: crate::types::Tree<Type = crate::types::Type, Label = defaults::LabelIdentifier>
            + crate::types::WithStats,
        T::TreeId: Copy + Into<NodeIdentifier>,
    {
        let kind = node.get_type();
        if let Some(children) = node.children() {
            let children = children.iter_children().map(|x| (*x).into()).collect();
            if node.has_label() {
                let arch = RawVariantDiscriminants::Both as u32;
                match self.stockages.get_mut(&arch).unwrap() {
                    RawVariant::Both {
                        entities: a @ variants::Both { .. },
                    } => {
                        a.rev.push(id);
                        a.kind.push(kind);

                        a.children.push(children);
                        a.label.push(node.get_label().into());

                        a.size.push(node.size());
                    }
                    _ => unreachable!(),
                }
            } else {
                let arch = RawVariantDiscriminants::Children as u32;
                match self.stockages.get_mut(&arch).unwrap() {
                    RawVariant::Children {
                        entities: a @ variants::Children { .. },
                    } => {
                        a.rev.push(id);
                        a.kind.push(kind);

                        a.children.push(children);

                        a.size.push(node.size());
                    }
                    _ => unreachable!(),
                }
            }
        } else if node.has_label() {
            let arch = RawVariantDiscriminants::Labeled as u32;
            match self.stockages.get_mut(&arch).unwrap() {
                RawVariant::Labeled {
                    entities: a @ variants::Labeled { .. },
                } => {
                    a.rev.push(id);
                    a.kind.push(kind);

                    a.label.push(node.get_label().into());

                    a.size.push(node.size());
                }
                _ => unreachable!(),
            }
        } else {
            let arch = RawVariantDiscriminants::Typed as u32;
            match self.stockages.get_mut(&arch).unwrap() {
                RawVariant::Typed {
                    entities: a @ variants::Typed { .. },
                } => {
                    a.rev.push(id);
                    a.kind.push(kind);

                    a.size.push(node.size());
                }
                _ => unreachable!(),
            }
        };
    }
    #[cfg(feature = "single-indirection")]
    pub fn add<T>(&mut self, id: NodeIdentifier, node: T)
    where
        T: crate::types::Tree<Type = crate::types::Type, Label = defaults::LabelIdentifier>,
        T::TreeId: Copy + Into<NodeIdentifier>,
    {
        let loc: Location = id.into();

        match self.stockages.entry(loc.arch) {
            hashbrown::hash_map::Entry::Occupied(mut occ) => {
                let kind = node.get_type();
                match occ.get_mut() {
                    RawVariant::Typed { entities } => {
                        entities.kind.push(kind);
                        entities.rev.push(loc.offset);
                    }
                    RawVariant::Labeled { entities } => {
                        entities.kind.push(kind);
                        entities.rev.push(loc.offset);
                        let label = node.get_label();
                        entities.label.push(label.into());
                    }
                    RawVariant::Children { entities } => {
                        entities.kind.push(kind);
                        entities.rev.push(loc.offset);
                        let children = node.children().unwrap();
                        entities
                            .children
                            .push(children.iter_children().map(|x| (*x).into()).collect());
                    }
                    RawVariant::Both { entities } => {
                        entities.kind.push(kind);
                        entities.rev.push(loc.offset);
                        let label = node.get_label();
                        entities.label.push(label.into());
                        let children = node.children().unwrap();
                        entities
                            .children
                            .push(children.iter_children().map(|x| (*x).into()).collect());
                    }
                }
            }
            hashbrown::hash_map::Entry::Vacant(vac) => {
                let kind = node.get_type();
                let ent = if let Some(children) = node.children() {
                    let children = children.iter_children().map(|x| (*x).into()).collect();
                    if node.has_label() {
                        RawVariant::Both {
                            entities: variants::Both {
                                rev: vec![loc.offset],
                                kind: vec![kind],
                                children: vec![children],
                                label: vec![node.get_label().into()],
                            },
                        }
                    } else {
                        RawVariant::Children {
                            entities: variants::Children {
                                rev: vec![loc.offset],
                                kind: vec![kind],
                                children: vec![children],
                            },
                        }
                    }
                } else if node.has_label() {
                    RawVariant::Labeled {
                        entities: variants::Labeled {
                            rev: vec![loc.offset],
                            kind: vec![kind],
                            label: vec![node.get_label().into()],
                        },
                    }
                } else {
                    RawVariant::Typed {
                        entities: variants::Typed {
                            rev: vec![loc.offset],
                            kind: vec![kind],
                        },
                    }
                };
                vac.insert(ent);
            }
        }
    }

    pub fn build(self, // ls: &crate::store::labels::LabelStore
    ) -> SimplePacked {
        let mut res = SimplePacked::default();
        // res.labels = self
        //     .label_ids
        //     .iter()
        //     .map(|x| ls.resolve(&x.0).to_string())
        //     .collect();
        // res.label_ids = self.label_ids;
        res.storages_arch.reserve_exact(self.stockages.len());
        res.storages_variants.reserve_exact(self.stockages.len());
        for (arch, variant) in self.stockages {
            res.storages_arch.push(arch);
            res.storages_variants.push(variant);
        }
        res
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SimplePacked {
    // label_ids: Vec<LabelIdentifier>,
    // labels: Vec<String>,
    storages_arch: Vec<u32>,
    storages_variants: Vec<RawVariant>,
}

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
    pub fn try_resolve(
        &self,
        id: NodeIdentifier,
    ) -> Option<<Self as crate::types::NodeStore<NodeIdentifier>>::R<'_>> {
        // let loc: Location = id.into();
        // self.stockages
        //     .get(&loc.arch)
        //     .and_then(|s| s.try_resolve(loc.offset))
        for v in self.stockages.values() {
            let r = v.try_resolve(id);
            if r.is_some() {
                return r;
            }
        }
        None
    }
}

fn i64_to_i32x2(n: u64) -> [u32; 2] {
    let n = n.to_le_bytes();
    [
        u32::from_le_bytes(n[..4].try_into().unwrap()),
        u32::from_le_bytes(n[4..].try_into().unwrap()),
    ]
}

impl NodeStore {
    pub fn len(&self) -> usize {
        let mut total = 0;
        for a in &self.stockages {
            total += a.1.rev().len()
        }
        total
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            stockages: Default::default(),
        }
    }

    pub fn extend(&mut self, raw: SimplePacked) //-> (Vec<LabelIdentifier>, Vec<String>)
    {
        for (arch, entities) in raw.storages_arch.into_iter().zip(raw.storages_variants) {
            self._extend_from_raw(arch, entities)
        }
        // (raw.label_ids, raw.labels)
    }

    fn _extend(&mut self, arch: u32, mut entities: Variant) {
        match self.stockages.entry(arch) {
            hashbrown::hash_map::Entry::Occupied(mut occ) => {
                occ.get_mut().extend(entities);
            }
            hashbrown::hash_map::Entry::Vacant(vac) => {
                let (index, rev) = entities.index_and_rev_mut();
                if index.is_empty() {
                    rev.iter().enumerate().for_each(|(i, x)| {
                        index.insert(*x, i as u32);
                    })
                }
                vac.insert(entities);
            }
        }
    }

    fn _extend_from_raw(&mut self, arch: u32, entities: RawVariant) {
        match self.stockages.entry(arch) {
            hashbrown::hash_map::Entry::Occupied(mut occ) => {
                occ.get_mut().extend_raw(entities);
            }
            hashbrown::hash_map::Entry::Vacant(vac) => {
                vac.insert(entities.into());
            }
        }
    }
}
