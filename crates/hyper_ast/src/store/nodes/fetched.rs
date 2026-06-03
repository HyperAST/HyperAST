use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[cfg(feature = "native")]
use string_interner::DefaultHashBuilder;
use string_interner::Symbol;

use crate::types::{AAAA, AnyType, Children, HyperType, NodeId, TypeTrait, TypedNodeId};

use strum_macros::*;
#[cfg(feature = "native")]
type HashMap<K, V> = hashbrown::HashMap<K, V, DefaultHashBuilder>;
#[cfg(not(feature = "native"))]
type HashMap<K, V> = hashbrown::HashMap<K, V, std::collections::hash_map::RandomState>;

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
        Self(value.0)
    }
}
impl From<&crate::store::defaults::LabelIdentifier> for LabelIdentifier {
    fn from(value: &crate::store::defaults::LabelIdentifier) -> Self {
        Self(value.0)
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
pub struct NodeIdentifier(std::num::NonZeroU32);

impl AAAA for NodeIdentifier {}
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
    pub fn to_u32(&self) -> u32 {
        self.0.into()
    }
    pub fn from_u32(value: std::num::NonZeroU32) -> Self {
        Self(value)
    }
}

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

pub struct FetchedLabels(HashMap<LabelIdentifier, String>);

impl crate::types::LabelStore<str> for FetchedLabels {
    type I = LabelIdentifier;

    fn get_or_insert<T: std::borrow::Borrow<str>>(&mut self, _node: T) -> Self::I {
        unimplemented!("definitely very inefficient, so should avoid using it anyway")
    }

    fn get<T: std::borrow::Borrow<str>>(&self, _node: T) -> Option<Self::I> {
        unimplemented!("definitely very inefficient, so should avoid using it anyway")
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

impl<'a, T> HashedNodeRef<'a, T> {
    fn _children(&self) -> Option<&[NodeIdentifier]> {
        match self.s_ref {
            VariantRef::Typed { .. } => None,
            VariantRef::Labeled { .. } => None,
            VariantRef::Children { entities, .. } => {
                Some(&entities.children[self.index as usize][..])
            }
            VariantRef::Both { entities, .. } => Some(&entities.children[self.index as usize][..]),
        }
    }
}

impl<'a, T> crate::types::CLending<'a, u16, NodeIdentifier> for HashedNodeRef<'_, T> {
    type Children = crate::types::ChildrenSlice<'a, NodeIdentifier>;
}

impl<'a, T> crate::types::WithChildren for HashedNodeRef<'a, T> {
    type ChildIdx = u16;
    // type Children<'b>
    //     = MySlice<<Self::TreeId as NodeId>::IdN>
    // where
    //     Self: 'b;

    fn child_count(&self) -> Self::ChildIdx {
        self.children().map_or(0, |x| x.child_count())
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        self.children().and_then(|x| x.get(*idx).copied())
    }

    fn child_rev(&self, _idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN> {
        todo!()
        // self.children().and_then(|x| x.rev(*idx)).copied()
    }

    fn children(
        &self,
    ) -> Option<crate::types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as NodeId>::IdN>> {
        self._children().map(|x| x.into())
    }
}

impl<'a, Id> super::ErasedHolder for HashedNodeRef<'a, Id> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if std::any::TypeId::of::<T>() == tid {
            todo!("{:?}", std::any::type_name::<T>())
            // if std::any::TypeId::of::<T>() == std::any::TypeId::of::<AnyType>() {
            // let lang = self.get_lang();
            // let raw = self.get_raw_type();
            // match lang {
            //     "hyperast_gen_ts_java::types::Lang" => {
            //         let t: &'static dyn hyperast::types::HyperType = hyperast_gen_ts_java::types::Lang::make(raw);
            //         t.into()
            //     },
            //     "hyperast_gen_ts_cpp::types::Lang" => {
            //         let t: &'static dyn hyperast::types::HyperType = hyperast_gen_ts_cpp::types::Lang::make(raw);
            //         t.into()
            //     },
            //     "hyperast_gen_ts_xml::types::Lang" => {
            //         let t: &'static dyn hyperast::types::HyperType = hyperast_gen_ts_xml::types::Lang::make(raw);
            //         t.into()
            //     },
            //     l => unreachable!("{}", l)
            // }
        } else {
            None
        }
    }
}

impl<'a, T: TypedNodeId> crate::types::Tree for HashedNodeRef<'a, T> {
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
                todo!("not sure how to do it")
                // match self.s_ref {$(
                //     VariantRef::$c{ entities: variants::$c{lang, kind,..}, ..} => {
                //         // assert_eq!(&std::any::type_name::<<T::Ty as TypeTrait>::Lang>(), lang);
                //         // use crate::types::Lang;
                //         // <T::Ty as TypeTrait>::Lang::make(kind[self.index as usize])
                //         todo!()
                //     },
                // )*}
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
                    VariantRef::$c{ entities: variants::$c{kind,..}, ..} => {
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

            fn line_count(&self) -> usize {
                todo!()
            }
        }
        impl From<RawVariant> for Variant {
            fn from(value: RawVariant) -> Self {
                match value {$(
                    RawVariant::$c{ entities } => {
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

// #[derive(Default)]
pub struct SimplePackedBuilder {
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
        Self {
            stockages: Default::default(),
        }
    }
}

impl SimplePackedBuilder {
    pub fn add<'store, HAST: crate::types::HyperAST>(&mut self, store: &'store HAST, id: &HAST::IdN)
    where
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: crate::types::WithStats,
        HAST::IdN: Into<NodeIdentifier> + Copy,
        HAST::IdN: NodeId<IdN = HAST::IdN>,
        HAST::Label: Into<LabelIdentifier> + Clone,
    {
        use crate::types::NodeStore;
        let node = store.node_store().resolve(id);
        let ty = store.resolve_type(id);
        let id = id.clone().into();
        let lang = ty.get_lang();
        use crate::types::LangRef;
        let lang_name = lang.name();
        let type_id = lang.to_u16(ty);
        use crate::types::Labeled;
        use crate::types::Tree;
        use crate::types::WithChildren;
        use crate::types::WithStats;
        if let Some(children) = node.children() {
            let children = children.map(|x| x.into()).collect();
            if node.has_label() {
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
                        a.label.push((node.get_label_unchecked().clone()).into());

                        a.size.push(node.size());
                    }
                    _ => unreachable!("SimplePackedBuilder::add variant Both"),
                }
            } else {
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

                    a.label.push((node.get_label_unchecked().clone()).into());

                    a.size.push(node.size());
                }
                _ => unreachable!("SimplePackedBuilder::add variant Labeled"),
            }
        } else {
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

    pub fn build(self) -> SimplePacked<&'static str> {
        let mut res = SimplePacked::default();
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

impl<'a> crate::types::NLending<'a, NodeIdentifier> for NodeStore {
    type N = HashedNodeRef<'a, AnyType>;
}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    fn resolve(&self, id: &NodeIdentifier) -> HashedNodeRef<'_, AnyType> {
        self.try_resolve(*id).unwrap()
    }
}

impl NodeStore {
    pub fn try_resolve<T>(&self, id: NodeIdentifier) -> Option<HashedNodeRef<'_, T>> {
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
            index: Default::default(),
            vindex: Default::default(),
            variants: Default::default(),
        }
    }

    pub fn extend(&mut self, raw: SimplePacked<String>) {
        for (arch, entities) in raw.storages_arch.into_iter().zip(raw.storages_variants) {
            self._extend_from_raw(arch, entities)
        }
    }

    fn _extend_from_raw(&mut self, arch: Arch<String>, entities: RawVariant) {
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
