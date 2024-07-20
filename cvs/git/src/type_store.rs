use core::panic;
use std::{fmt::Display, hash::Hash};

use hyper_ast::{
    store::{defaults::NodeIdentifier, nodes::legion::HashedNodeRef},
    types::{AnyType, HyperType, LangRef, LangWrapper, Shared, TypeIndex, TypeStore, Typed},
};
#[cfg(feature = "cpp")]
use hyper_ast_gen_ts_cpp::types::CppEnabledTypeStore;
#[cfg(feature = "java")]
use hyper_ast_gen_ts_java::types::JavaEnabledTypeStore;
#[cfg(feature = "maven")]
use hyper_ast_gen_ts_xml::types::XmlEnabledTypeStore;

use crate::no_space::{MIdN, NoSpaceWrapper};

#[repr(u8)]
pub enum TStore {
    Maven = 0,
    Java = 1,
    Cpp = 2,
}

impl<'a> From<&'a str> for MultiType {
    fn from(value: &'a str) -> Self {
        MultiType::Java(value.into())
    }
}

impl Default for TStore {
    fn default() -> Self {
        Self::Maven
    }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for hyper_ast_gen_ts_java::types::TStore {
    type Ty = AnyType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
        n.inner.get_type()
    }

    fn resolve_lang(
        &self,
        n: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Marshaled {
        todo!()
        // hyper_ast_gen_ts_java::types::TStore::Java::marshal_type
        // TypeIndex {
        //     lang: LangRef::<hyper_ast_gen_ts_java::types::Type>::name(&hyper_ast_gen_ts_java::types::Lang),
        //     ty: *n.get_component::<hyper_ast_gen_ts_java::types::Type>().unwrap() as u16,
        // }
    }
    fn type_eq(
        &self,
        n: &NoSpaceWrapper<'a, NodeIdentifier>,
        m: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> bool {
        todo!()
        // use hecs::entity_ref::ComponentRef;
        // n.get_component::<hyper_ast_gen_ts_java::types::Type>().unwrap() == m.get_component::<hyper_ast_gen_ts_java::types::Type>().unwrap()
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        todo!()
        // hyper_ast_gen_ts_java::types::id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

type TypeInternalSize = u16;

macro_rules! on_multi {
    ($n:expr, [$on0:ident $(, $on:ident)*], ($with:ident, $with1:ident) => $body:expr, $default:expr) => {
        if let Ok($with) = $n.get_component::<$on0::types::Type>() {
            use $on0 as $with1;
            $body
        } $( else if let Ok($with) = $n.get_component::<$on::types::Type>() {
            use $on as $with1;
            $body
        })* else {
            $default
        }
    };
}

impl<'a> TypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
    type Ty = AnyType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
        on_multi!(n, [
                hyper_ast_gen_ts_java,
                hyper_ast_gen_ts_cpp,
                hyper_ast_gen_ts_xml
            ],
            (t, u) => u::types::as_any(t),
            {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        )
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, NodeIdentifier>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        on_multi!(n, [
                hyper_ast_gen_ts_java,
                hyper_ast_gen_ts_cpp,
                hyper_ast_gen_ts_xml
            ],
            (_t, u) => From::<&'static (dyn LangRef<AnyType>)>::from(&u::types::Lang),
            {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        )
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
        on_multi!(n, [
                hyper_ast_gen_ts_java,
                hyper_ast_gen_ts_cpp,
                hyper_ast_gen_ts_xml
            ],
            (t, u) => {
                let ty = <u::types::Lang as hyper_ast::types::Lang<_>>::to_u16(*t);
                let lang = hyper_ast::types::LangRef::<u::types::Type>::name(
                    &u::types::Lang,
                );
                TypeIndex { lang, ty }
            },
            {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        )
    }

    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, NodeIdentifier>,
        m: &HashedNodeRef<'a, NodeIdentifier>,
    ) -> bool {
        on_multi!(n, [
                hyper_ast_gen_ts_java,
                hyper_ast_gen_ts_cpp,
                hyper_ast_gen_ts_xml
            ],
            (t, u) =>{
                if let Ok(tt) = m.get_component::<u::types::Type>() {
                    t == tt
                } else {
                    false
                }},
            {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        )
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}

impl<'a>
    hyper_ast::types::RoleStore<
        HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    > for TStore
{
    type IdF = u16;

    type Role = hyper_ast::types::Role;

    fn resolve_field(&self, lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
        hyper_ast::types::RoleStore::<
            hyper_ast::store::nodes::legion::HashedNodeRef<
                'a,
                hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>,
            >,
        >::resolve_field(&hyper_ast_gen_ts_java::types::TStore::Java, lang, field_id)
    }

    fn intern_role(&self, lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
        hyper_ast::types::RoleStore::<
            hyper_ast::store::nodes::legion::HashedNodeRef<
                'a,
                hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>,
            >,
        >::intern_role(&hyper_ast_gen_ts_java::types::TStore::Java, lang, role)
    }
}

impl<'a> hyper_ast::types::RoleStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
    type IdF = u16;

    type Role = hyper_ast::types::Role;

    fn resolve_field(&self, lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
        // match self {
        //     TStore::Maven => todo!(),
        //     TStore::Java => todo!(),
        //     TStore::Cpp => todo!(),
        // }
        match lang.name() {
            "Java" => hyper_ast::types::RoleStore::<
                hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            >::resolve_field(
                &hyper_ast_gen_ts_java::types::TStore::Java, lang, field_id
            ),
            "Cpp" => hyper_ast_gen_ts_cpp::types::TStore::Cpp.resolve_field(lang, field_id),
            "Xml" => hyper_ast_gen_ts_xml::types::TStore::Xml.resolve_field(lang, field_id),
            x => panic!("{}", x),
        }
        // TODO fix that

        // let s = tree_sitter_java::language()
        //     .field_name_for_id(field_id)
        //     .ok_or_else(|| format!("{}", field_id))
        //     .unwrap();
        // hyper_ast::types::Role::try_from(s).expect(s)
    }

    fn intern_role(&self, lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
        // TODO fix that
        match lang.name() {
            "hyper_ast_gen_ts_java::types::Lang" => {
                hyper_ast::types::RoleStore::<
                    hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
                >::intern_role(
                    &hyper_ast_gen_ts_java::types::TStore::Java, lang, role
                )
            }
            "hyper_ast_gen_ts_cpp::types::Lang" => {
                hyper_ast_gen_ts_cpp::types::TStore::Cpp.intern_role(lang, role)
            }
            "hyper_ast_gen_ts_xml::types::Lang" => {
                hyper_ast_gen_ts_xml::types::TStore::Xml.intern_role(lang, role)
            }
            x => panic!("{}", x),
        }
        // let field_name = role.to_string();
        // tree_sitter_java::language()
        //     .field_id_for_name(field_name)
        //     .unwrap()
        //     .into()
    }
}

#[allow(unused)] // TODO find a better way of declaring type stores
impl<'a> TypeStore<HashedNodeRef<'a, MIdN<NodeIdentifier>>> for TStore {
    type Ty = MultiType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>) -> Self::Ty {
        use hyper_ast::types::Typed;
        n.get_type()
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
        // if let Ok(t) = n.get_component::<hyper_ast_gen_ts_java::types::Type>() {
        //     From::<&'static (dyn LangRef<MultiType>)>::from(&hyper_ast_gen_ts_java::types::Java)
        // } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_cpp::types::Type>() {
        //     From::<&'static (dyn LangRef<MultiType>)>::from(&hyper_ast_gen_ts_cpp::types::Cpp)
        // } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_xml::types::Type>() {
        //     From::<&'static (dyn LangRef<MultiType>)>::from(&hyper_ast_gen_ts_xml::types::Xml)
        // } else {
        //     dbg!(n, n.archetype().layout().component_types());
        //     panic!()
        // }
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>) -> Self::Marshaled {
        todo!()
    }
    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
        m: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
    ) -> bool {
        todo!("{:?} {:?}", n, m)
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}

// impl<I: AsRef<HashedNodeRef<'static, NodeIdentifier>>> TypeStore<I> for TStore {
//     type Ty = AnyType;
//     const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

//     fn resolve_type(&self, n: &I) -> Self::Ty {
//         todo!()
//     }
// }
// impl<'a, I: Deref<Target=HashedNodeRef<'a, NodeIdentifier>>> TypeStore<I> for TStore {
//     type Ty = AnyType;
//     const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

//     fn resolve_type(&self, n: &I) -> Self::Ty {
//         todo!()
//     }
// }
impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for TStore {
    type Ty = AnyType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
        self.resolve_type(n.as_ref())
    }

    fn resolve_lang(
        &self,
        _n: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, _n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Marshaled {
        todo!()
    }
    fn type_eq(
        &self,
        _n: &NoSpaceWrapper<'a, NodeIdentifier>,
        _m: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> bool {
        todo!()
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}
// impl<'a, I: AsRef<HashedNodeRef<'a, NodeIdentifier>>> TypeStore<I> for &TStore {
//     type Ty = AnyType;
//     const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

//     fn resolve_type(&self, n: &I) -> Self::Ty {
//         let n = n.as_ref();
//         <TStore as TypeStore<HashedNodeRef<'a, NodeIdentifier>>>::resolve_type(self, n)
//     }

//     fn resolve_lang(&self, n: &I) -> hyper_ast::types::LangWrapper<Self::Ty> {
//         todo!()
//     }

//     type Marshaled = TypeIndex;

//     fn marshal_type(&self, n: &I) -> Self::Marshaled {
//         todo!()
//     }
// }

impl<'a> TypeStore<HashedNodeRef<'a, MIdN<NodeIdentifier>>> for &TStore {
    type Ty = MultiType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>) -> Self::Ty {
        let n = n.as_ref();
        n.get_type()
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!("{:?}", n)
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, _n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>) -> Self::Marshaled {
        todo!()
    }
    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
        m: &HashedNodeRef<'a, MIdN<NodeIdentifier>>,
    ) -> bool {
        todo!("{:?} {:?}", n, m)
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, MIdN<NodeIdentifier>>> for &TStore {
    type Ty = MultiType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>) -> Self::Ty {
        let n = n.as_ref();
        n.get_type()
    }

    fn resolve_lang(
        &self,
        _n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, _n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>) -> Self::Marshaled {
        todo!()
    }

    fn type_eq(
        &self,
        _n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
        _m: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
    ) -> bool {
        todo!()
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for &TStore {
    type Ty = MultiType;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
        n.get_type()
        // on_multi!(n.as_ref(), [
        //     hyper_ast_gen_ts_java,
        //     hyper_ast_gen_ts_cpp,
        //     hyper_ast_gen_ts_xml
        // ],
        // (t, u) => u::types::as_any(t),
        // {
        //     dbg!(n.as_ref().archetype().layout().component_types());
        //     panic!()
        // }
        // )
    }

    fn resolve_lang(
        &self,
        _n: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    type Marshaled = TypeIndex;

    fn marshal_type(&self, _n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Marshaled {
        todo!()
    }

    fn type_eq(
        &self,
        n: &NoSpaceWrapper<'a, NodeIdentifier>,
        m: &NoSpaceWrapper<'a, NodeIdentifier>,
    ) -> bool {
        on_multi!(n.as_ref(), [
                hyper_ast_gen_ts_java,
                hyper_ast_gen_ts_cpp,
                hyper_ast_gen_ts_xml
            ],
            (t, u) =>{
                if let Ok(tt) = m.as_ref().get_component::<u::types::Type>() {
                    t == tt
                } else {
                    false
                }
            },
            {
                dbg!(n.as_ref().archetype().layout().component_types());
                panic!()
            }
        )
    }
}

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_java::types::Type;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    ) -> Self::Ty {
        n.get_type()
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!("{:?}", n)
    }

    type Marshaled = TypeIndex;

    fn marshal_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    ) -> Self::Marshaled {
        TypeIndex {
            lang: LangRef::<hyper_ast_gen_ts_java::types::Type>::name(
                &hyper_ast_gen_ts_java::types::Lang,
            ),
            ty: *n
                .get_component::<hyper_ast_gen_ts_java::types::Type>()
                .unwrap() as u16,
        }
    }

    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
        m: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    ) -> bool {
        n.get_component::<hyper_ast_gen_ts_java::types::Type>()
            .unwrap()
            == m.get_component::<hyper_ast_gen_ts_java::types::Type>()
                .unwrap()
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}
impl<'a> JavaEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>>
    for TStore
{
}

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_xml::types::Type;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    ) -> Self::Ty {
        todo!("{:?}", n)
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!("{:?}", n)
    }

    type Marshaled = TypeIndex;

    fn marshal_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    ) -> Self::Marshaled {
        todo!("{:?}", n)
    }

    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
        m: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    ) -> bool {
        todo!("{:?} {:?}", n, m)
    }
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}
impl<'a> XmlEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>>
    for TStore
{
    const LANG: u16 = 0;

    fn _intern(l: u16, t: u16) -> Self::Ty {
        unimplemented!("remove _intern {} {}", l, t)
    }

    fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_xml::types::Type {
        todo!("{:?}", t)
    }
}

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_cpp::types::Type;
    const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    fn resolve_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    ) -> Self::Ty {
        todo!("{:?}", n)
    }

    fn resolve_lang(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    ) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!("{:?}", n)
    }

    type Marshaled = TypeIndex;

    fn marshal_type(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    ) -> Self::Marshaled {
        todo!("{:?}", n)
    }

    fn type_eq(
        &self,
        n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
        m: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    ) -> bool {
        todo!("{:?} {:?}", n, m)
    }
}
impl<'a> CppEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>>
    for TStore
{
    const LANG: u16 = 0;

    fn _intern(l: u16, t: u16) -> Self::Ty {
        *<hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<
            hyper_ast_gen_ts_cpp::types::Type,
        >>::make(t)
    }

    fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_cpp::types::Type {
        todo!("{:?}", t)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MultiType {
    Java(hyper_ast_gen_ts_java::types::Type),
    Cpp(hyper_ast_gen_ts_cpp::types::Type),
    Xml(hyper_ast_gen_ts_xml::types::Type),
}

macro_rules! on_multi2 {
    ($on:expr, $with:ident => $body:expr) => {
        match $on {
            MultiType::Java($with) => $body,
            MultiType::Cpp($with) => $body,
            MultiType::Xml($with) => $body,
        }
    };
    ($on1:expr, $on2:expr, ($with1:ident,$with2:ident) => $body:expr, _ => $default:expr) => {
        match ($on1, $on2) {
            (MultiType::Java($with1), MultiType::Java($with2)) => $body,
            (MultiType::Cpp($with1), MultiType::Cpp($with2)) => $body,
            (MultiType::Xml($with1), MultiType::Xml($with2)) => $body,
            _ => $default,
        }
    };
}

unsafe impl Send for MultiType {}
unsafe impl Sync for MultiType {}
impl PartialEq for MultiType {
    fn eq(&self, other: &Self) -> bool {
        on_multi2!(self, other, (s, o) => s == o, _ => false)
    }
}
impl Eq for MultiType {}
impl Hash for MultiType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        on_multi2!(self, t => t.hash(state))
    }
}
impl Display for MultiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        on_multi2!(self, t => std::fmt::Display::fmt(t, f))
    }
}

impl HyperType for MultiType {
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // elegant solution leveraging the static nature of node types
        std::ptr::eq(self.as_static(), other.as_static())
    }

    fn is_file(&self) -> bool {
        on_multi2!(self, t => t.is_file())
    }

    fn is_directory(&self) -> bool {
        on_multi2!(self, t => t.is_directory())
    }

    fn is_spaces(&self) -> bool {
        on_multi2!(self, t => t.is_spaces())
    }

    fn is_syntax(&self) -> bool {
        on_multi2!(self, t => t.is_syntax())
    }

    fn as_shared(&self) -> Shared {
        on_multi2!(self, t => t.as_shared())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        on_multi2!(self, t => t.as_any())
    }

    fn as_static(&self) -> &'static dyn HyperType {
        on_multi2!(self, t => t.as_static())
    }

    fn as_static_str(&self) -> &'static str {
        on_multi2!(self, t => t.to_str())
    }

    fn is_hidden(&self) -> bool {
        on_multi2!(self, t => t.is_hidden())
    }

    fn is_supertype(&self) -> bool {
        on_multi2!(self, t => t.is_supertype())
    }

    fn is_named(&self) -> bool {
        on_multi2!(self, t => t.is_named())
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        match self {
            MultiType::Java(_t) => {
                LangWrapper::from(&MultiLang::Java as &'static dyn LangRef<MultiType>)
            }
            MultiType::Cpp(_t) => {
                LangWrapper::from(&MultiLang::Cpp as &'static dyn LangRef<MultiType>)
            }
            MultiType::Xml(_t) => {
                LangWrapper::from(&MultiLang::Xml as &'static dyn LangRef<MultiType>)
            }
        }
    }
    fn lang_ref(&self) -> LangWrapper<AnyType> {
        todo!()
    }
}

enum MultiLang {
    Java,
    Cpp,
    Xml,
}

impl LangRef<MultiType> for MultiLang {
    fn name(&self) -> &'static str {
        match self {
            MultiLang::Java => "Java",
            MultiLang::Cpp => "Cpp",
            MultiLang::Xml => "Xml",
        }
    }

    fn make(&self, t: u16) -> &'static MultiType {
        unimplemented!()
    }

    fn to_u16(&self, t: MultiType) -> u16 {
        todo!()
    }

    fn ts_symbol(&self, t: MultiType) -> u16 {
        todo!()
    }
}

#[test]
fn type_test_generic_eq() {
    use hyper_ast::types::HyperType;

    let t0 = hyper_ast_gen_ts_cpp::types::Type::FunctionDefinition;
    let t1 = hyper_ast_gen_ts_cpp::types::Type::EnumSpecifier;

    let k = MultiType::Cpp(t0);
    let k0 = MultiType::Cpp(t0);
    let k1 = MultiType::Cpp(t1);
    assert!(k.eq(&k));
    assert!(k.eq(&k0));
    assert!(k0.eq(&k));
    assert!(k1.eq(&k1));
    assert!(k.ne(&k1));
    assert!(k1.ne(&k));

    assert!(k.generic_eq(&k));
    assert!(k.generic_eq(&k0));
    assert!(k0.generic_eq(&k));
    assert!(k1.generic_eq(&k1));
    assert!(!k.generic_eq(&k1));
    assert!(!k1.generic_eq(&k));

    let ak = hyper_ast_gen_ts_cpp::types::as_any(&t0.clone());
    let ak0 = hyper_ast_gen_ts_cpp::types::as_any(&t0.clone());
    let ak1 = hyper_ast_gen_ts_cpp::types::as_any(&t1.clone());

    assert!(ak.generic_eq(&ak));
    assert!(ak.generic_eq(&ak0));
    assert!(ak0.generic_eq(&ak));
    assert!(ak1.generic_eq(&ak1));
    assert!(!ak.generic_eq(&ak1));
    assert!(!ak1.generic_eq(&ak));

    assert!(k.generic_eq(&ak));
    assert!(k.generic_eq(&ak0));
    assert!(k0.generic_eq(&ak));
    assert!(k1.generic_eq(&ak1));
    assert!(!k.generic_eq(&ak1));
    assert!(!k1.generic_eq(&ak));

    assert!(ak.generic_eq(&k));
    assert!(ak.generic_eq(&k0));
    assert!(ak0.generic_eq(&k));
    assert!(ak1.generic_eq(&k1));
    assert!(!ak.generic_eq(&k1));
    assert!(!ak1.generic_eq(&k));

    assert!(ak.eq(&ak));
    assert!(ak.eq(&ak0));
    assert!(ak0.eq(&ak));
    assert!(ak1.eq(&ak1));
    assert!(!ak.eq(&ak1));
    assert!(!ak1.eq(&ak));

    let ak = t0.clone();
    let ak0 = t0.clone();
    let ak1 = t1.clone();

    assert!(ak.generic_eq(&ak));
    assert!(ak.generic_eq(&ak0));
    assert!(ak0.generic_eq(&ak));
    assert!(ak1.generic_eq(&ak1));
    assert!(!ak.generic_eq(&ak1));
    assert!(!ak1.generic_eq(&ak));

    assert!(k.generic_eq(&ak));
    assert!(k.generic_eq(&ak0));
    assert!(k0.generic_eq(&ak));
    assert!(k1.generic_eq(&ak1));
    assert!(!k.generic_eq(&ak1));
    assert!(!k1.generic_eq(&ak));

    assert!(ak.generic_eq(&k));
    assert!(ak.generic_eq(&k0));
    assert!(ak0.generic_eq(&k));
    assert!(ak1.generic_eq(&k1));
    assert!(!ak.generic_eq(&k1));
    assert!(!ak1.generic_eq(&k));

    assert!(ak.eq(&ak));
    assert!(ak.eq(&ak0));
    assert!(ak0.eq(&ak));
    assert!(ak1.eq(&ak1));
    assert!(!ak.eq(&ak1));
    assert!(!ak1.eq(&ak));
}
