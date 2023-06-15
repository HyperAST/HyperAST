use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU64},
};

use string_interner::{DefaultHashBuilder, Symbol};

use crate::{
    filter::BloomResult,
    hashed::SyntaxNodeHashsKinds,
    impact::serialize::{Keyed, MySerialize},
    nodes::{CompressedNode, HashSize, RefContainer},
    store::{defaults, labels::LabelStore},
    types::{
        AnyType, IterableChildren, LabelStore as _, MySlice, NodeId, TypeIndex, TypeStore,
        TypeTrait, Typed, TypedNodeId,
    },
};

use strum_macros::*;
#[cfg(feature = "native")]
type HashMap<K, V> = hashbrown::HashMap<K, V, DefaultHashBuilder>;
#[cfg(feature = "native")]
type HashSet<K> = hashbrown::HashSet<K, DefaultHashBuilder>;
#[cfg(not(feature = "native"))]
type HashMap<K, V> = hashbrown::HashMap<K, V, std::collections::hash_map::RandomState>;
#[cfg(not(feature = "native"))]
type HashSet<K> = hashbrown::HashSet<K, std::collections::hash_map::RandomState>;

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

pub struct FetchedLabels(HashMap<LabelIdentifier, String>);
// #[cfg(feature = "fetched_par")]
// pub type FetchedLabels = HashMap<LabelIdentifier, String>;

impl crate::types::LabelStore<str> for FetchedLabels {
    type I = LabelIdentifier;

    fn get_or_insert<T: std::borrow::Borrow<str>>(&mut self, node: T) -> Self::I {
        todo!()
    }

    fn get<T: std::borrow::Borrow<str>>(&self, node: T) -> Option<Self::I> {
        todo!()
    }

    fn resolve(&self, id: &Self::I) -> &str {
        self.0.get(id).unwrap()
    }
}

impl FetchedLabels {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn try_resolve(&self, id: &<Self as crate::types::LabelStore<str>>::I) -> Option<&str> {
        self.0.get(id).map(|x| x.as_str())
    }
    pub fn insert(&mut self, k: LabelIdentifier, v: String) -> Option<String> {
        self.0.insert(k, v)
    }
}

#[cfg(feature = "native")]
impl Default for FetchedLabels {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(not(feature = "native"))]
impl Default for FetchedLabels {
    fn default() -> Self {
        use std::collections::hash_map::RandomState;
        let inner = HashMap::with_hasher(RandomState::default()); //generate_with(42, 142, 542, 9342));
        Self(inner)
    }
}

pub struct HashedNodeRef<'a, T> {
    index: u32,
    s_ref: VariantRef<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> HashedNodeRef<'a, T> {
    #[doc(hidden)]
    pub fn cast_type<U>(self) -> HashedNodeRef<'a, U> {
        HashedNodeRef {
            index: self.index,
            s_ref: self.s_ref,
            phantom: PhantomData,
        }
    }
}

// impl<'a,T> PartialEq for HashedNodeRef<'a,T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl<'a,T> Eq for HashedNodeRef<'a,T> {}

// impl<'a,T> Hash for HashedNodeRef<'a,T> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         todo!()
//     }
// }

impl<'a, T> Debug for HashedNodeRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedNodeRef")
            // .field("id", &self.id)
            // .field("ty", &self.ty)
            // .field("label", &self.label)
            // .field("children", &self.children)
            .finish()
    }
}
impl<'a, T> crate::types::WithSerialization for HashedNodeRef<'a, T> {
    fn try_bytes_len(&self) -> Option<usize> {
        todo!()
    }
}

impl<'a, T> crate::types::Node for HashedNodeRef<'a, T> {}

impl<'a, T> crate::types::Stored for HashedNodeRef<'a, T> {
    type TreeId = NodeIdentifier;
}

impl<'a, T> crate::types::WithChildren for HashedNodeRef<'a, T> {
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

impl<'a, T> crate::types::WithHashs for HashedNodeRef<'a, T> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        todo!()
    }
}

impl<'a, T: TypedNodeId> crate::types::Tree for HashedNodeRef<'a, T>
where
    T::Ty: TypeTrait,
{
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
impl<'a, T> crate::types::Labeled for HashedNodeRef<'a, T> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        match self.s_ref {
            VariantRef::Typed { .. } => panic!(),
            VariantRef::Labeled { entities, .. } => &entities.label[self.index as usize],
            VariantRef::Children { .. } => panic!(),
            VariantRef::Both { entities, .. } => &entities.label[self.index as usize],
        }
    }
    fn try_get_label(&self) -> Option<&LabelIdentifier> {
        match self.s_ref {
            VariantRef::Typed { .. } => None,
            VariantRef::Labeled { entities, .. } => Some(&entities.label[self.index as usize]),
            VariantRef::Children { .. } => None,
            VariantRef::Both { entities, .. } => Some(&entities.label[self.index as usize]),
        }
    }
}
impl<'a, T> RefContainer for HashedNodeRef<'a, T> {
    type Result = BloomResult;

    fn check<U: MySerialize + Keyed<usize>>(&self, rf: U) -> Self::Result {
        todo!()
    }
}

impl<'a, T> HashedNodeRef<'a, T> {
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
    ) -> Result<CompressedNode<NodeIdentifier, LabelIdentifier, T>, String> {
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

    pub fn is_directory(&self) -> bool
    where
        T: TypedNodeId,
        T::Ty: TypeTrait,
    {
        use crate::types::HyperType;
        self.get_type().is_directory()
    }

    pub fn get_child_by_name(
        &self,
        name: &<HashedNodeRef<'a, T> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a, T> as crate::types::Stored>::TreeId> {
        todo!()
    }

    pub fn get_child_idx_by_name(
        &self,
        name: &<HashedNodeRef<'a, T> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a, T> as crate::types::WithChildren>::ChildIdx> {
        todo!()
    }

    pub fn try_get_children_name(
        &self,
    ) -> Option<&[<HashedNodeRef<'a, T> as crate::types::Labeled>::Label]> {
        todo!()
    }
}

#[derive(Default, Debug)]
pub struct NodeStore {
    // #[cfg(not(feature = "fetched_par"))]
    // stockages: HashMap<u32, Variant>,
    index: HashMap<NodeIdentifier, (u32, u32)>,
    vindex: HashMap<Arch<String>, u32>,
    variants: Vec<Variant>,
    // #[cfg(feature = "fetched_par")]
    // stockages: dashmap::DashMap<u32, Variant>,
}
impl Hash for NodeStore {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // for r in &self.stockages {
        //     r.0.hash(state);
        //     r.1.rev().len().hash(state);
        // }
        self.index.keys().for_each(|x| x.hash(state))
    }
}
type StaticStr = &'static str;
lazy_static::lazy_static! {
    static ref LANGS: std::sync::Mutex<Vec<StaticStr>> = {
        Default::default()
    };
}
#[cfg(feature = "serialize")]
fn deserialize_static_str<'de, D>(deserializer: D) -> Result<StaticStr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    <&str>::deserialize(deserializer).map(move |d| {
        let mut langs = LANGS.lock().unwrap();
        for x in langs.iter() {
            if *x == d {
                return *x;
            }
        }
        let d: &'static str = Box::leak(d.into());
        langs.push(d);
        d
    })
}

macro_rules! variant_store {
    ($id:ty, $rev:ty ; $($( #[$doc:meta] )* $c:ident => { $($d:ident : $e:ty),* $(,)?}),* $(,)?) => {
        mod variants {
            use super::*;

            $(
            $( #[$doc] )*
            #[derive(Clone, Debug)]
            #[cfg_attr(feature = "serialize", derive(serde::Serialize,serde::Deserialize))]
            pub struct $c{
                #[cfg_attr(feature = "serialize", serde(deserialize_with = "deserialize_static_str"))]
                pub(super) lang: StaticStr,
                pub(super) rev: Vec<$rev>,
                $(pub(super) $d: Vec<$e>,)*
            }
            impl $c{
                pub(super) fn lang(lang: &'static str) -> Self {
                    Self {
                        lang,
                        rev: Default::default(),
                        $($d: Default::default(),)*

                    }
                }
            }
            )*
        }
        #[derive(Clone, Debug)]
        #[derive(EnumDiscriminants)]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize,serde::Deserialize))]
        #[cfg_attr(feature = "serialize", strum_discriminants(derive(serde::Serialize,serde::Deserialize)))]
        pub enum RawVariant {
            $($c{
                entities: variants::$c,
            },)*
        }
        #[derive(Debug)]
        pub enum Variant {
            $($c{
                // index: HashMap<$rev, u32>,
                entities: variants::$c,
            },)*
        }
        pub enum VariantRef<'a> {
            $($c{
                entities: &'a variants::$c,
            },)*
        }
        impl Variant {
            // pub fn try_resolve<T>(&self, offset: $rev) -> Option<HashedNodeRef<'_, T>> {
            //     Some(match self {$(
            //         Variant::$c{entities, index} => HashedNodeRef {
            //             index: *index.get(&offset)?,
            //             s_ref: VariantRef::$c{entities},
            //             phantom: PhantomData,
            //         },
            //     )*})
            // }
            pub fn get<T>(&self, index: u32) -> HashedNodeRef<'_, T> {
                match self {$(
                    Variant::$c{entities} => HashedNodeRef {
                        index,
                        s_ref: VariantRef::$c{entities},
                        phantom: PhantomData,
                    },
                )*}
            }
            // pub fn index_and_rev_mut(&mut self) -> (&mut HashMap<$rev, u32>, &Vec<$rev>) {
            //     match self {$(
            //         Variant::$c{ index, entities: variants::$c{rev, ..} } => (&mut *index, &*rev),
            //     )*}
            // }
            pub fn rev(&self) -> &Vec<$rev> {
                match self {$(
                    Variant::$c{ entities: variants::$c{rev, ..}, ..} => &rev,
                )*}
            }
            // fn extend(&mut self, other: Self) {
            //     match (self, other) {$(
            //         (Variant::$c{ index, entities }, Variant::$c{ entities: other, ..}) => {
            //             $(
            //                 assert_eq!(other.rev.len(), other.$d.len());
            //                 let mut $d = other.$d.into_iter();
            //             )*
            //             for k in other.rev.into_iter() {
            //                 if !index.contains_key(&k) {
            //                     let i = entities.rev.len();
            //                     entities.rev.push(k);
            //                     index.insert(k, i as u32);
            //                     $(
            //                         entities.$d.push($d.next().unwrap());
            //                     )*
            //                 }
            //             }
            //         },
            //         )*
            //         _=> unreachable!()
            //     }
            // }
            fn extend_raw(&mut self, other: RawVariant) {
                match (self, other) {$(
                    // (Variant::$c{ index, entities }, RawVariant::$c{ entities: other, ..}) => {
                    //     $(
                    //         assert_eq!(other.rev.len(), other.$d.len());
                    //         let mut $d = other.$d.into_iter();
                    //     )*
                    //     for k in other.rev.into_iter() {
                    //         if !index.contains_key(&k) {
                    //             let i = entities.rev.len();
                    //             entities.rev.push(k);
                    //             index.insert(k, i as u32);
                    //             $(
                    //                 entities.$d.push($d.next().unwrap());
                    //             )*
                    //         }
                    //     }
                    // },
                    // )*
                    (Variant::$c{ entities }, RawVariant::$c{ entities: other, ..}) => {
                        // $(
                        //     assert_eq!(other.rev.len(), other.$d.len());
                        //     let mut $d = other.$d.into_iter();
                        // )*
                        entities.rev.extend(other.rev);
                        $(
                            entities.$d.extend(other.$d);
                        )*
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
        impl<'a,T: TypedNodeId> crate::types::Typed for HashedNodeRef<'a,T> where T::Ty: TypeTrait {
            type Type = T::Ty;

            fn get_type(&self) -> T::Ty {
                match self.s_ref {$(
                    VariantRef::$c{ entities: variants::$c{lang, kind,..}, ..} => {
                        // assert_eq!(&std::any::type_name::<<T::Ty as TypeTrait>::Lang>(), lang);
                        // use crate::types::Lang;
                        // <T::Ty as TypeTrait>::Lang::make(kind[self.index as usize])
                        todo!()
                    },
                )*}
            }
        }
        impl<'a,T> HashedNodeRef<'a,T> {
            pub fn get_lang(&self) -> &'static str {
                match self.s_ref {$(
                    VariantRef::$c{ entities: variants::$c{lang,..}, ..} => {
                        lang
                    },
                )*}
            }
            pub fn get_raw_type(&self) -> u16 {
                match self.s_ref {$(
                    VariantRef::$c{ entities: variants::$c{lang, kind,..}, ..} => {
                        kind[self.index as usize]
                    },
                )*}
            }
        }
        impl<'a,T> crate::types::WithStats for HashedNodeRef<'a,T> {
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
                        // let mut index = HashMap::default();
                        // entities.rev.iter().enumerate().for_each(|(i, x)| {
                        //     index.insert(*x, i as u32);
                        // });
                        // Variant::$c{ index, entities }
                        Variant::$c{ entities }
                    },
                )*}
            }
        }
    };
}

variant_store!(NodeIdentifier, NodeIdentifier;
    /// Just a leaf with a type
    Typed => {
        kind: u16,
        size: usize,
    },
    Labeled => {
        kind: u16,
        label: LabelIdentifier,
        size: usize,
    },
    Children => {
        kind: u16,
        children: Vec<NodeIdentifier>,
        size: usize,
    },
    Both => {
        kind: u16,
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
    langs: Vec<&'static str>,
    // // label_ids: Vec<LabelIdentifier>,
    stockages: HashMap<Arch<&'static str>, RawVariant>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
struct Arch<S>(S, RawVariantDiscriminants);

impl<S: Hash> Hash for Arch<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        (self.1 as u32).hash(state);
    }
}

impl Default for SimplePackedBuilder {
    fn default() -> Self {
        let mut r = Self {
            langs: Default::default(),
            stockages: Default::default(),
        };
        // let arch = RawVariantDiscriminants::Typed as u32;
        // r.stockages.insert(
        //     arch,
        //     RawVariant::Typed {
        //         entities: variants::Typed::default(),
        //     },
        // );
        // let arch = RawVariantDiscriminants::Labeled as u32;
        // r.stockages.insert(
        //     arch,
        //     RawVariant::Labeled {
        //         entities: variants::Labeled::default(),
        //     },
        // );
        // let arch = RawVariantDiscriminants::Children as u32;
        // r.stockages.insert(
        //     arch,
        //     RawVariant::Children {
        //         entities: variants::Children::default(),
        //     },
        // );
        // let arch = RawVariantDiscriminants::Both as u32;
        // r.stockages.insert(
        //     arch,
        //     RawVariant::Both {
        //         entities: variants::Both::default(),
        //     },
        // );
        r
    }
}

impl SimplePackedBuilder {
    pub fn add<TS, T>(&mut self, type_store: &TS, id: NodeIdentifier, node: T)
    where
        TS: TypeStore<T, Marshaled = TypeIndex>,
        T::Type: 'static,
        T: crate::types::Tree<Label = defaults::LabelIdentifier> + crate::types::WithStats,
        <T::TreeId as NodeId>::IdN: Copy + Into<NodeIdentifier>,
    {
        // use crate::types::Lang;
        // let kind = type_store.resolve_type(&node);
        let TypeIndex {
            lang: lang_name,
            ty: type_id,
        } = type_store.marshal_type(&node);
        // let l_id = self
        //     .langs
        //     .iter()
        //     .position(|x| x == &lang_name)
        //     .unwrap_or_else(|| {
        //         let l = self.langs.len();
        //         assert!(l < 8); // NOTE: limit for now
        //         self.langs.push(lang_name);
        //         l
        //     }) as u32;
        // if true {
        //     panic!("{}", lang_name);
        // }
        if let Some(children) = node.children() {
            let children = children.iter_children().map(|x| (*x).into()).collect();
            if node.has_label() {
                // let arch = l_id.rotate_right(3) | RawVariantDiscriminants::Both as u32;
                match self
                    .stockages
                    .entry(Arch(lang_name, RawVariantDiscriminants::Both))
                    .or_insert_with(|| RawVariant::Both {
                        entities: variants::Both::lang(lang_name),
                    }) {
                    RawVariant::Both {
                        entities: a @ variants::Both { .. },
                    } => {
                        a.rev.push(id);
                        a.kind.push(type_id);

                        a.children.push(children);
                        a.label.push(node.get_label_unchecked().into());

                        a.size.push(node.size());
                    }
                    _ => unreachable!("SimplePackedBuilder::add variant Both"),
                }
            } else {
                // let arch = l_id.rotate_right(3) |  RawVariantDiscriminants::Children as u32;
                match self
                    .stockages
                    .entry(Arch(lang_name, RawVariantDiscriminants::Children))
                    .or_insert_with(|| RawVariant::Children {
                        entities: variants::Children::lang(lang_name),
                    }) {
                    RawVariant::Children {
                        entities: a @ variants::Children { .. },
                    } => {
                        a.rev.push(id);
                        a.kind.push(type_id);

                        a.children.push(children);

                        a.size.push(node.size());
                    }
                    _ => unreachable!("SimplePackedBuilder::add variant Children"),
                }
            }
        } else if node.has_label() {
            // let arch = l_id.rotate_right(3) |  RawVariantDiscriminants::Labeled as u32;
            match self
                .stockages
                .entry(Arch(lang_name, RawVariantDiscriminants::Labeled))
                .or_insert_with(|| RawVariant::Labeled {
                    entities: variants::Labeled::lang(lang_name),
                }) {
                RawVariant::Labeled {
                    entities: a @ variants::Labeled { .. },
                } => {
                    a.rev.push(id);
                    a.kind.push(type_id);

                    a.label.push(node.get_label_unchecked().into());

                    a.size.push(node.size());
                }
                _ => unreachable!("SimplePackedBuilder::add variant Labeled"),
            }
        } else {
            // let arch = l_id.rotate_right(3) |  RawVariantDiscriminants::Typed as u32;
            match self
                .stockages
                .entry(Arch(lang_name, RawVariantDiscriminants::Typed))
                .or_insert_with(|| RawVariant::Typed {
                    entities: variants::Typed::lang(lang_name),
                }) {
                RawVariant::Typed {
                    entities: a @ variants::Typed { .. },
                } => {
                    a.rev.push(id);
                    a.kind.push(type_id);

                    a.size.push(node.size());
                }
                _ => unreachable!("SimplePackedBuilder::add variant Typed"),
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
    ) -> SimplePacked<&'static str> {
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
pub struct SimplePacked<S> {
    // label_ids: Vec<LabelIdentifier>,
    // labels: Vec<String>,
    storages_arch: Vec<Arch<S>>,
    storages_variants: Vec<RawVariant>,
}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    type R<'a> = HashedNodeRef<'a, AnyType>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        todo!()
    }
}
impl<TIdN: TypedNodeId> crate::types::TypedNodeStore<TIdN> for NodeStore
where
    TIdN::Ty: TypeTrait,
{
    type R<'a> = HashedNodeRef<'a, TIdN>;
    fn resolve(&self, id: &TIdN) -> Self::R<'_> {
        todo!()
    }

    fn try_typed(&self, id: &<TIdN as NodeId>::IdN) -> Option<TIdN> {
        todo!()
    }
}

impl NodeStore {
    pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef<'_, AnyType> {
        todo!()
    }
    pub fn try_resolve<T>(&self, id: NodeIdentifier) -> Option<HashedNodeRef<'_, T>> {
        // // let loc: Location = id.into();
        // // self.stockages
        // //     .get(&loc.arch)
        // //     .and_then(|s| s.try_resolve(loc.offset))
        // for v in self.stockages.values() {
        //     let r = v.try_resolve(id);
        //     if r.is_some() {
        //         return r;
        //     }
        // }
        let (variant, offset) = self.index.get(&id)?;
        Some(self.variants[*variant as usize].get(*offset))
    }

    pub fn unavailable_node<T>(&self) -> HashedNodeRef<'_, T> {
        HashedNodeRef {
            index: 0,
            s_ref: VariantRef::Typed {
                entities: UNAILABLE_NODE,
            },
            phantom: PhantomData,
        }
    }
}

const UNAILABLE_NODE: &'static variants::Typed = &variants::Typed {
    lang: "",
    rev: vec![],
    kind: vec![],
    size: vec![],
};

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
        for a in &self.variants {
            total += a.rev().len()
        }
        total
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            // stockages: Default::default(),
            index: Default::default(),
            vindex: Default::default(),
            variants: Default::default(),
        }
    }

    pub fn extend(&mut self, raw: SimplePacked<String>) //-> (Vec<LabelIdentifier>, Vec<String>)
    {
        for (arch, entities) in raw.storages_arch.into_iter().zip(raw.storages_variants) {
            self._extend_from_raw(arch, entities)
        }
        // (raw.label_ids, raw.labels)
    }

    // fn _extend(&mut self, arch: u32, mut entities: Variant) {
    //     match self.stockages.entry(arch) {
    //         hashbrown::hash_map::Entry::Occupied(mut occ) => {
    //             occ.get_mut().extend(entities);
    //         }
    //         hashbrown::hash_map::Entry::Vacant(vac) => {
    //             let (index, rev) = entities.index_and_rev_mut();
    //             if index.is_empty() {
    //                 rev.iter().enumerate().for_each(|(i, x)| {
    //                     index.insert(*x, i as u32);
    //                 })
    //             }
    //             vac.insert(entities);
    //         }
    //     }
    // }

    fn _extend_from_raw(&mut self, arch: Arch<String>, entities: RawVariant) {
        // let var_index = self.variants.len() as u32;
        // let new =  ();
        // for ent in entities.rev() {

        // }
        match self.vindex.entry(arch) {
            hashbrown::hash_map::Entry::Occupied(occ) => {
                let var = &mut self.variants[*occ.get() as usize];
                let mut offset = var.rev().len() as u32;
                for ent in entities.rev() {
                    self.index.insert(*ent, (*occ.get() as u32, offset));
                    offset += 1;
                }
                var.extend_raw(entities);
            }
            hashbrown::hash_map::Entry::Vacant(vac) => {
                let len = self.variants.len();
                vac.insert(len as u32);
                let mut offset = 0;
                for ent in entities.rev() {
                    self.index.insert(*ent, (len as u32, offset));
                    offset += 1;
                }
                self.variants.push(entities.into());
            }
        }
    }
}
