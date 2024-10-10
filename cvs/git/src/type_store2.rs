use core::panic;

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

pub struct TStore;

impl Default for TStore {
    fn default() -> Self {
        Self
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
    // fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
    //     on_multi!(n, [
    //             hyper_ast_gen_ts_java,
    //             hyper_ast_gen_ts_cpp,
    //             hyper_ast_gen_ts_xml
    //         ],
    //         (t, u) => u::types::as_any(t),
    //         {
    //             dbg!(n, n.archetype().layout().component_types());
    //             panic!()
    //         }
    //     )
    // }

    // fn resolve_lang(
    //     &self,
    //     n: &HashedNodeRef<'a, NodeIdentifier>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     on_multi!(n, [
    //             hyper_ast_gen_ts_java,
    //             hyper_ast_gen_ts_cpp,
    //             hyper_ast_gen_ts_xml
    //         ],
    //         (_t, u) => From::<&'static (dyn LangRef<AnyType>)>::from(&u::types::Lang),
    //         {
    //             dbg!(n, n.archetype().layout().component_types());
    //             panic!()
    //         }
    //     )
    // }

    // fn type_eq(
    //     &self,
    //     n: &HashedNodeRef<'a, NodeIdentifier>,
    //     m: &HashedNodeRef<'a, NodeIdentifier>,
    // ) -> bool {
    //     on_multi!(n, [
    //             hyper_ast_gen_ts_java,
    //             hyper_ast_gen_ts_cpp,
    //             hyper_ast_gen_ts_xml
    //         ],
    //         (t, u) =>{
    //             if let Ok(tt) = m.get_component::<u::types::Type>() {
    //                 t == tt
    //             } else {
    //                 false
    //             }},
    //         {
    //             dbg!(n, n.archetype().layout().component_types());
    //             panic!()
    //         }
    //     )
    // }
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
        >::resolve_field(&hyper_ast_gen_ts_java::types::TStore, lang, field_id)
    }

    fn intern_role(&self, lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
        hyper_ast::types::RoleStore::<
            hyper_ast::store::nodes::legion::HashedNodeRef<
                'a,
                hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>,
            >,
        >::intern_role(&hyper_ast_gen_ts_java::types::TStore, lang, role)
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
            "hyper_ast_gen_ts_java::types::Lang" => {
                hyper_ast::types::RoleStore::<
                    hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
                >::resolve_field(
                    &hyper_ast_gen_ts_java::types::TStore, lang, field_id
                )
            }
            "hyper_ast_gen_ts_cpp::types::Lang" => {
                hyper_ast_gen_ts_cpp::types::TStore.resolve_field(lang, field_id)
            }
            "hyper_ast_gen_ts_xml::types::Lang" => {
                hyper_ast_gen_ts_xml::types::TStore.resolve_field(lang, field_id)
            }
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
                >::intern_role(&hyper_ast_gen_ts_java::types::TStore, lang, role)
            }
            "hyper_ast_gen_ts_cpp::types::Lang" => {
                hyper_ast_gen_ts_cpp::types::TStore.intern_role(lang, role)
            }
            "hyper_ast_gen_ts_xml::types::Lang" => {
                hyper_ast_gen_ts_xml::types::TStore.intern_role(lang, role)
            }
            x => panic!("{}", x),
        }
    }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for TStore {
    type Ty = AnyType;
    // fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
    //     self.resolve_type(n.as_ref())
    // }

    // fn resolve_lang(
    //     &self,
    //     _n: &NoSpaceWrapper<'a, NodeIdentifier>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!()
    // }

    // fn type_eq(
    //     &self,
    //     _n: &NoSpaceWrapper<'a, NodeIdentifier>,
    //     _m: &NoSpaceWrapper<'a, NodeIdentifier>,
    // ) -> bool {
    //     todo!()
    // }
}
// impl<'a, I: AsRef<HashedNodeRef<'a, NodeIdentifier>>> TypeStore<I> for &TStore {
//     type Ty = AnyType;
//     fn resolve_type(&self, n: &I) -> Self::Ty {
//         let n = n.as_ref();
//         <TStore as TypeStore<HashedNodeRef<'a, NodeIdentifier>>>::resolve_type(self, n)
//     }

//     fn resolve_lang(&self, n: &I) -> hyper_ast::types::LangWrapper<Self::Ty> {
//         todo!()
//     }

//     fn marshal_type(&self, n: &I) -> Self::Marshaled {
//         todo!()
//     }
// }

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_java::types::TType;
    // fn resolve_type(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    // ) -> Self::Ty {
    //     n.get_type()
    // }

    // fn resolve_lang(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!("{:?}", n)
    // }

    // fn type_eq(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    //     m: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
    // ) -> bool {
    //     n.get_component::<hyper_ast_gen_ts_java::types::Type>()
    //         .unwrap()
    //         == m.get_component::<hyper_ast_gen_ts_java::types::Type>()
    //             .unwrap()
    // }
}
impl<'a> JavaEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>>
    for TStore
{
    fn intern(&self, t: hyper_ast_gen_ts_java::types::Type) -> Self::Ty {
        // *<hyper_ast_gen_ts_java::types::Java as hyper_ast::types::Lang<
        //     hyper_ast_gen_ts_java::types::Type,
        // >>::make(t as u16)
        t.into()
    }

    fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_java::types::Type {
        t.e()
    }
}

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_xml::types::TType;
    // fn resolve_type(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    // ) -> Self::Ty {
    //     todo!("{:?}", n)
    // }

    // fn resolve_lang(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!("{:?}", n)
    // }

    // fn type_eq(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    //     m: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
    // ) -> bool {
    //     todo!("{:?} {:?}", n, m)
    // }
}
impl<'a> XmlEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>>
    for TStore
{
    fn intern(&self, t: hyper_ast_gen_ts_xml::types::Type) -> Self::Ty {
        t.into()
    }

    fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_xml::types::Type {
        t.e()
    }
}

impl<'a> TypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>>
    for TStore
{
    type Ty = hyper_ast_gen_ts_cpp::types::TType;
    // fn resolve_type(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    // ) -> Self::Ty {
    //     todo!("{:?}", n)
    // }

    // fn resolve_lang(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!("{:?}", n)
    // }

    // fn type_eq(
    //     &self,
    //     n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    //     m: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
    // ) -> bool {
    //     todo!("{:?} {:?}", n, m)
    // }
}
impl<'a> CppEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>>
    for TStore
{
    fn intern(&self, t: hyper_ast_gen_ts_cpp::types::Type) -> Self::Ty {
        // *<hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<
        //     hyper_ast_gen_ts_cpp::types::Type,
        // >>::make(t as u16)
        t.into()
    }

    fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_cpp::types::Type {
        todo!("{:?}", t)
    }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, MIdN<NodeIdentifier>>> for &TStore {
    type Ty = AnyType;
    // fn resolve_type(&self, n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>) -> Self::Ty {
    //     let n = n.as_ref();
    //     todo!()
    //     // n.get_type()
    // }

    // fn resolve_lang(
    //     &self,
    //     _n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!()
    // }

    // fn type_eq(
    //     &self,
    //     _n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
    //     _m: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
    // ) -> bool {
    //     todo!()
    // }
}

impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for &TStore {
    type Ty = AnyType;

    fn decompress_type(
        &self,
        erazed: impl hyper_ast::types::ErasedCompo,
        tid: std::any::TypeId,
    ) -> Self::Ty {
        unsafe { erazed.unerase_ref::<hyper_ast_gen_ts_java::types::TType>(tid) }
            .map(|t| todo!())
            // .or_else(|| unsafe { ptr.unerase_ref::<ByteLenU32>(tid) }.map(ByteLenU32::decompresses))
            // .or_else(|| unsafe { ptr.unerase_ref::<ByteLenU16>(tid) }.map(ByteLenU16::decompresses))
            .unwrap_or_else(|| unreachable!())
    }
    // fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
    //     todo!()
    //     // n.get_type()
    //     // on_multi!(n.as_ref(), [
    //     //     hyper_ast_gen_ts_java,
    //     //     hyper_ast_gen_ts_cpp,
    //     //     hyper_ast_gen_ts_xml
    //     // ],
    //     // (t, u) => u::types::as_any(t),
    //     // {
    //     //     dbg!(n.as_ref().archetype().layout().component_types());
    //     //     panic!()
    //     // }
    //     // )
    // }

    // fn resolve_lang(
    //     &self,
    //     _n: &NoSpaceWrapper<'a, NodeIdentifier>,
    // ) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //     todo!()
    // }

    // fn type_eq(
    //     &self,
    //     n: &NoSpaceWrapper<'a, NodeIdentifier>,
    //     m: &NoSpaceWrapper<'a, NodeIdentifier>,
    // ) -> bool {
    //     on_multi!(n.as_ref(), [
    //             hyper_ast_gen_ts_java,
    //             hyper_ast_gen_ts_cpp,
    //             hyper_ast_gen_ts_xml
    //         ],
    //         (t, u) =>{
    //             if let Ok(tt) = m.as_ref().get_component::<u::types::Type>() {
    //                 t == tt
    //             } else {
    //                 false
    //             }
    //         },
    //         {
    //             dbg!(n.as_ref().archetype().layout().component_types());
    //             panic!()
    //         }
    //     )
    // }
}
