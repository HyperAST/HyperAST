#![feature(test)]
#![feature(extract_if)]
#![feature(trait_upcasting)]
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

pub(crate) type DefaultMetrics = hyper_ast::tree_gen::SubTreeMetrics<hyper_ast::hashed::SyntaxNodeHashs<u32>>;

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
    use std::{fmt::Display, hash::Hash, ops::Deref};

    use hyper_ast::{
        store::{defaults::NodeIdentifier, nodes::legion::HashedNodeRef},
        types::{
            AnyType, HyperType, Lang, LangRef, LangWrapper, NodeId, Shared, TypeIndex, TypeStore,
            Typed, TypedNodeId, T,
        },
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

    impl<'a> TypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
        type Ty = AnyType;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

        fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
            if let Ok(t) = n.get_component::<hyper_ast_gen_ts_java::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_java::types::Java as hyper_ast::types::Lang<_>>::make(t);
                From::<&'static (dyn HyperType)>::from(t)
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_cpp::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<_>>::make(t);
                From::<&'static (dyn HyperType)>::from(t)
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_xml::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_xml::types::Xml as hyper_ast::types::Lang<_>>::make(t);
                From::<&'static (dyn HyperType)>::from(t)
            } else {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            if let Ok(t) = n.get_component::<hyper_ast_gen_ts_java::types::Type>() {
                From::<&'static (dyn LangRef<AnyType>)>::from(&hyper_ast_gen_ts_java::types::Java)
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_cpp::types::Type>() {
                From::<&'static (dyn LangRef<AnyType>)>::from(&hyper_ast_gen_ts_cpp::types::Cpp)
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_xml::types::Type>() {
                From::<&'static (dyn LangRef<AnyType>)>::from(&hyper_ast_gen_ts_xml::types::Xml)
            } else {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
            if let Ok(t) = n.get_component::<hyper_ast_gen_ts_java::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_java::types::Java as hyper_ast::types::Lang<_>>::make(t);
                let lang = hyper_ast::types::LangRef::<hyper_ast_gen_ts_java::types::Type>::name(
                    &hyper_ast_gen_ts_java::types::Java,
                );
                let ty = *t as u16;
                TypeIndex { lang, ty }
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_cpp::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_cpp::types::Cpp as hyper_ast::types::Lang<_>>::make(t);
                let lang = hyper_ast::types::LangRef::<hyper_ast_gen_ts_cpp::types::Type>::name(
                    &hyper_ast_gen_ts_cpp::types::Cpp,
                );
                let ty = *t as u16;
                TypeIndex { lang, ty }
            } else if let Ok(t) = n.get_component::<hyper_ast_gen_ts_xml::types::Type>() {
                let t = *t as u16;
                let t = <hyper_ast_gen_ts_xml::types::Xml as hyper_ast::types::Lang<_>>::make(t);
                let lang = hyper_ast::types::LangRef::<hyper_ast_gen_ts_xml::types::Type>::name(
                    &hyper_ast_gen_ts_xml::types::Xml,
                );
                let ty = *t as u16;
                TypeIndex { lang, ty }
            } else {
                dbg!(n, n.archetype().layout().component_types());
                panic!()
            }
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
    }

    impl<'a> TypeStore<NoSpaceWrapper<'a, NodeIdentifier>> for &TStore {
        type Ty = MultiType;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

        fn resolve_type(&self, n: &NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
            n.get_type()
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

    unsafe impl Send for MultiType {}
    unsafe impl Sync for MultiType {}
    impl PartialEq for MultiType {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (MultiType::Java(s), MultiType::Java(o)) => s == o,
                (MultiType::Cpp(s), MultiType::Cpp(o)) => s == o,
                (MultiType::Xml(s), MultiType::Xml(o)) => s == o,
                _ => false,
            }
        }
    }
    impl Eq for MultiType {}
    impl Hash for MultiType {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                MultiType::Java(t) => t.hash(state),
                MultiType::Cpp(t) => t.hash(state),
                MultiType::Xml(t) => t.hash(state),
            }
        }
    }
    impl Display for MultiType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MultiType::Java(t) => std::fmt::Display::fmt(t, f),
                MultiType::Cpp(t) => std::fmt::Display::fmt(t, f),
                MultiType::Xml(t) => std::fmt::Display::fmt(t, f),
            }
        }
    }

    impl HyperType for MultiType {
        fn is_file(&self) -> bool {
            match self {
                MultiType::Java(t) => t.is_file(),
                MultiType::Cpp(t) => t.is_file(),
                MultiType::Xml(t) => t.is_file(),
            }
        }

        fn is_directory(&self) -> bool {
            match self {
                MultiType::Java(t) => t.is_file(),
                MultiType::Cpp(t) => t.is_file(),
                MultiType::Xml(t) => t.is_file(),
            }
        }

        fn is_spaces(&self) -> bool {
            match self {
                MultiType::Java(t) => t.is_spaces(),
                MultiType::Cpp(t) => t.is_spaces(),
                MultiType::Xml(t) => t.is_spaces(),
            }
        }

        fn is_syntax(&self) -> bool {
            match self {
                MultiType::Java(t) => t.is_syntax(),
                MultiType::Cpp(t) => t.is_syntax(),
                MultiType::Xml(t) => t.is_syntax(),
            }
        }

        fn as_shared(&self) -> Shared {
            match self {
                MultiType::Java(t) => t.as_shared(),
                MultiType::Cpp(t) => t.as_shared(),
                MultiType::Xml(t) => t.as_shared(),
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            match self {
                MultiType::Java(t) => t.as_any(),
                MultiType::Cpp(t) => t.as_any(),
                MultiType::Xml(t) => t.as_any(),
            }
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
