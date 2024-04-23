#![feature(test)]
#![feature(extract_if)]
#![feature(trait_upcasting)]
#![feature(os_str_bytes)]
pub mod allrefs;
pub mod cpp;
pub mod git;
pub mod java;
pub mod make;
pub mod maven;

#[cfg(feature = "cpp")]
pub mod cpp_processor;
#[cfg(feature = "java")]
pub mod java_processor;
#[cfg(feature = "make")]
pub mod make_processor;
#[cfg(feature = "maven")]
pub mod maven_processor;
pub mod multi_preprocessed;
pub mod no_space;
/// for now only tested on maven repositories with a pom in root.
pub mod preprocessed;
pub mod processing;
mod utils;

#[cfg(test)]
pub mod tests;

use git::BasicGitObject;
use git2::Oid;
use hyper_ast::{store::defaults::LabelIdentifier, utils::Bytes};
extern crate test;

// use hyper_ast_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
// use hyper_ast_gen_ts_xml::xml_tree_gen::{self, XmlTreeGen};

pub type SimpleStores = hyper_ast::store::SimpleStores<TStore>;

// might also skip
pub(crate) const PROPAGATE_ERROR_ON_BAD_CST_NODE: bool = false;

pub(crate) const MAX_REFS: u32 = 10000; //4096;

pub(crate) type DefaultMetrics =
    hyper_ast::tree_gen::SubTreeMetrics<hyper_ast::hashed::SyntaxNodeHashs<u32>>;

pub struct Diffs();
pub struct Impacts();

#[derive(Clone)]
pub struct Commit {
    pub parents: Vec<git2::Oid>,
    processing_time: u128,
    memory_used: Bytes,
    pub ast_root: hyper_ast::store::nodes::DefaultNodeIdentifier,
    pub tree_oid: git2::Oid,
}

impl Commit {
    pub fn processing_time(&self) -> u128 {
        self.processing_time
    }
    pub fn memory_used(&self) -> Bytes {
        self.memory_used
    }
}
trait Accumulator: hyper_ast::tree_gen::Accumulator<Node = (LabelIdentifier, Self::Unlabeled)> {
    type Unlabeled;
    // fn push(&mut self, name: LabelIdentifier, full_node: Self::Node);
}

trait Processor<Acc: Accumulator> {
    fn process(&mut self) -> Acc::Unlabeled {
        loop {
            if let Some(current_dir) = self.stack().last_mut().expect("never empty").1.pop() {
                self.pre(current_dir)
            } else if let Some((oid, _, acc)) = self.stack().pop() {
                if let Some(x) = self.post(oid, acc) {
                    return x;
                }
            } else {
                panic!("never empty")
            }
        }
    }
    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, Acc)>;

    fn pre(&mut self, current_dir: BasicGitObject);
    fn post(&mut self, oid: Oid, acc: Acc) -> Option<Acc::Unlabeled>;
}

#[derive(Debug)]
pub(crate) enum ParseErr {
    NotUtf8(std::str::Utf8Error),
    IllFormed,
}

impl From<std::str::Utf8Error> for ParseErr {
    fn from(value: std::str::Utf8Error) -> Self {
        ParseErr::NotUtf8(value)
    }
}

mod type_store {
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

    impl Default for TStore {
        fn default() -> Self {
            Self::Maven
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
    }

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
            todo!()
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
            n: &NoSpaceWrapper<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Marshaled {
            todo!()
        }
        fn type_eq(
            &self,
            n: &NoSpaceWrapper<'a, NodeIdentifier>,
            m: &NoSpaceWrapper<'a, NodeIdentifier>,
        ) -> bool {
            todo!()
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
            todo!()
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
            todo!()
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
            n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>) -> Self::Marshaled {
            todo!()
        }

        fn type_eq(
            &self,
            n: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
            m: &NoSpaceWrapper<'a, MIdN<NodeIdentifier>>,
        ) -> bool {
            todo!()
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
            n: &NoSpaceWrapper<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Marshaled {
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
                    }},
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
            todo!()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
        ) -> Self::Marshaled {
            todo!()
        }

        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
            m: &HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>,
        ) -> bool {
            todo!()
        }
    }
    impl<'a>
        JavaEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_java::types::TIdN<NodeIdentifier>>>
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
            todo!()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
        ) -> Self::Marshaled {
            todo!()
        }

        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
            m: &HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>,
        ) -> bool {
            todo!()
        }
    }
    impl<'a>
        XmlEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_xml::types::TIdN<NodeIdentifier>>>
        for TStore
    {
        const LANG: u16 = 0;

        fn _intern(l: u16, t: u16) -> Self::Ty {
            hyper_ast_gen_ts_xml::types::Type::resolve(t)
        }

        fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_xml::types::Type {
            todo!()
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
            todo!()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
        ) -> Self::Marshaled {
            todo!()
        }

        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
            m: &HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>,
        ) -> bool {
            todo!()
        }
    }
    impl<'a>
        CppEnabledTypeStore<HashedNodeRef<'a, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>>
        for TStore
    {
        const LANG: u16 = 0;

        fn _intern(l: u16, t: u16) -> Self::Ty {
            hyper_ast_gen_ts_cpp::types::Type::resolve(t)
        }

        fn resolve(&self, t: Self::Ty) -> hyper_ast_gen_ts_cpp::types::Type {
            todo!()
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum MultiType {
        Java(hyper_ast_gen_ts_java::types::Type),
        Cpp(hyper_ast_gen_ts_cpp::types::Type),
        Xml(hyper_ast_gen_ts_xml::types::Type),
    }

    macro_rules! on_multi {
        ($on:ident, $with:ident => $body:expr) => {
            match $on {
                MultiType::Java($with) => $body,
                MultiType::Cpp($with) => $body,
                MultiType::Xml($with) => $body,
            }
        };
        ($on1:ident, $on2:ident, ($with1:ident,$with2:ident) => $body:expr, _ => $default:expr) => {
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
            on_multi!(self, other, (s, o) => s == o, _ => false)
        }
    }
    impl Eq for MultiType {}
    impl Hash for MultiType {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            on_multi!(self, t => t.hash(state))
        }
    }
    impl Display for MultiType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            on_multi!(self, t => std::fmt::Display::fmt(t, f))
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
            on_multi!(self, t => t.is_file())
        }

        fn is_directory(&self) -> bool {
            on_multi!(self, t => t.is_directory())
        }

        fn is_spaces(&self) -> bool {
            on_multi!(self, t => t.is_spaces())
        }

        fn is_syntax(&self) -> bool {
            on_multi!(self, t => t.is_syntax())
        }

        fn as_shared(&self) -> Shared {
            on_multi!(self, t => t.as_shared())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            on_multi!(self, t => t.as_any())
        }

        fn as_static(&self) -> &'static dyn HyperType {
            on_multi!(self, t => t.as_static())
        }

        fn get_lang(&self) -> LangWrapper<Self>
        where
            Self: Sized,
        {
            // self.0.get_lang()
            panic!()
        }
    }
}

pub use type_store::MultiType;
pub use type_store::TStore;

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
