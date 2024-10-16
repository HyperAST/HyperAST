use core::panic;

use hyper_ast::types::{AnyType, HyperType, LangRef, LangWrapper, TypeStore};

#[derive(Clone)]
pub struct TStore;

#[cfg(feature = "cpp")]
impl hyper_ast::store::TyDown<hyper_ast_gen_ts_cpp::types::TStore> for TStore {}
#[cfg(feature = "java")]
impl hyper_ast::store::TyDown<hyper_ast_gen_ts_java::types::TStore> for TStore {}
#[cfg(feature = "maven")]
impl hyper_ast::store::TyDown<hyper_ast_gen_ts_xml::types::TStore> for TStore {}

impl Default for TStore {
    fn default() -> Self {
        Self
    }
}

// TODO use/adapt it for decompress_type and roles
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

impl<'a> hyper_ast::types::RoleStore for TStore {
    type IdF = u16;

    type Role = hyper_ast::types::Role;

    fn resolve_field(lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
        match lang.name() {
            #[cfg(feature = "java")]
            "hyper_ast_gen_ts_java::types::Lang" => {
                let t = hyper_ast_gen_ts_java::types::TType::new(
                    hyper_ast_gen_ts_java::types::Type::Spaces,
                );
                hyper_ast_gen_ts_java::types::TStore::resolve_field(t.get_lang(), field_id)
            }
            #[cfg(feature = "cpp")]
            "hyper_ast_gen_ts_cpp::types::Lang" => {
                let t = hyper_ast_gen_ts_cpp::types::TType::new(
                    hyper_ast_gen_ts_cpp::types::Type::Spaces,
                );
                hyper_ast_gen_ts_cpp::types::TStore::resolve_field(t.get_lang(), field_id)
            }
            #[cfg(feature = "maven")]
            "hyper_ast_gen_ts_xml::types::Lang" => {
                let t = hyper_ast_gen_ts_xml::types::TType::new(
                    hyper_ast_gen_ts_xml::types::Type::Spaces,
                );
                hyper_ast_gen_ts_xml::types::TStore::resolve_field(t.get_lang(), field_id)
            }
            x => panic!("{}", x),
        }
    }

    fn intern_role(lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
        // TODO fix that, the lang thing, both parameter and the get_lang() should be respectively extracted and removed
        match lang.name() {
            #[cfg(feature = "java")]
            "hyper_ast_gen_ts_java::types::Lang" => {
                let t = hyper_ast_gen_ts_java::types::TType::new(
                    hyper_ast_gen_ts_java::types::Type::Spaces,
                );
                hyper_ast_gen_ts_java::types::TStore::intern_role(t.get_lang(), role)
            }
            #[cfg(feature = "cpp")]
            "hyper_ast_gen_ts_cpp::types::Lang" => {
                let t = hyper_ast_gen_ts_cpp::types::TType::new(
                    hyper_ast_gen_ts_cpp::types::Type::Spaces,
                );
                hyper_ast_gen_ts_cpp::types::TStore::intern_role(t.get_lang(), role)
            }
            #[cfg(feature = "maven")]
            "hyper_ast_gen_ts_xml::types::Lang" => {
                let t = hyper_ast_gen_ts_xml::types::TType::new(
                    hyper_ast_gen_ts_xml::types::Type::Spaces,
                );
                hyper_ast_gen_ts_xml::types::TStore::intern_role(t.get_lang(), role)
            }
            x => panic!("{}", x),
        }
    }
}

impl TypeStore for TStore {
    type Ty = AnyType;

    fn decompress_type(
        erazed: &impl hyper_ast::types::ErasedHolder,
        tid: std::any::TypeId,
    ) -> Self::Ty {
        unsafe {
            erazed.unerase_ref_unchecked::<hyper_ast_gen_ts_java::types::TType>(
                std::any::TypeId::of::<hyper_ast_gen_ts_java::types::TType>(),
            )
        }
        .map(|t| t.as_static().into())
        .or_else(|| {
            unsafe {
                erazed.unerase_ref_unchecked::<hyper_ast_gen_ts_cpp::types::TType>(
                    std::any::TypeId::of::<hyper_ast_gen_ts_cpp::types::TType>(),
                )
            }
            .map(|t| t.as_static().into())
        })
        .or_else(|| {
            unsafe {
                erazed.unerase_ref_unchecked::<hyper_ast_gen_ts_xml::types::TType>(
                    std::any::TypeId::of::<hyper_ast_gen_ts_xml::types::TType>(),
                )
            }
            .map(|t| t.as_static().into())
        })
        .unwrap_or_else(|| {
            dbg!(tid);
            dbg!(std::any::type_name::<Self::Ty>());
            unreachable!()
        })
    }
}
