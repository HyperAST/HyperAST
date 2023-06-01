use std::fmt::Display;

use git2::Repository;
use hyper_ast::types::LangRef;

use crate::git::Repo;

pub enum BuildSystem {
    Maven,
    Make,
    Npm,
    None,
}

enum Language {
    Java,
    Cpp,
    Ts,
    Xml,
}

pub enum ProcessingConfig<P> {
    JavaMaven { limit: usize, dir_path: P },
    CppMake { limit: usize, dir_path: P },
    TsNpm { limit: usize, dir_path: P },
    Any { limit: usize, dir_path: P },
}

/// Contains repository configuration,
/// where each config given the same commit should produce the same result in the hyperast
#[derive(serde::Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RepoConfig {
    CppMake,
    JavaMaven,
    TsNpm,
    Any,
}
impl From<&RepoConfig> for ProcessingConfig<&'static str> {
    fn from(value: &RepoConfig) -> Self {
        match value {
            RepoConfig::CppMake => Self::CppMake {
                limit: 3,
                dir_path: "src",
            },
            RepoConfig::JavaMaven => Self::JavaMaven {
                limit: 3,
                dir_path: "",
            },
            RepoConfig::TsNpm => todo!(),
            RepoConfig::Any => todo!(),
        }
    }
}
impl From<RepoConfig> for ProcessingConfig<&'static str> {
    fn from(value: RepoConfig) -> Self {
        (&value).into()
    }
}

pub trait ConfiguredRepoTrait {
    fn spec(&self) -> &Repo;
    fn config(&self) -> &RepoConfig;
}

pub struct ConfiguredRepoHandle {
    pub spec: Repo,
    pub config: RepoConfig,
}
// NOTE could have impl deref bug it is a bad idea (see book), related to ownership
impl ConfiguredRepoTrait for ConfiguredRepoHandle {
    fn spec(&self) -> &Repo {
        &self.spec
    }

    fn config(&self) -> &RepoConfig {
        &self.config
    }
}

impl ConfiguredRepoHandle {
    pub fn fetch(self) -> ConfiguredRepo {
        ConfiguredRepo {
            repo: self.spec.fetch(),
            spec: self.spec,
            config: self.config,
        }
    }
}

pub struct ConfiguredRepo {
    pub spec: Repo,
    pub repo: Repository,
    pub config: RepoConfig,
}

impl ConfiguredRepoTrait for ConfiguredRepo {
    fn spec(&self) -> &Repo {
        &self.spec
    }

    fn config(&self) -> &RepoConfig {
        &self.config
    }
}

pub trait CachesHolder {
    /// WARN if you use the same cache type in multiple holders it mean that they are effectively shared caches
    /// TIPs use a wrapping type to protect againts inadvertent sharing
    type Caches;

    // fn mut_or_default(&mut self) -> &mut Self::Caches;
}
pub trait InFiles {
    fn matches(name: &ObjectName) -> bool;
}

pub trait ObjectMapper {
    type K;
    type V;
    fn get(&self, key: &Self::K) -> Option<&Self::V>;
    fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V>;
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct ObjectName(Vec<u8>);

// TODO make a slice variant like str and String

impl ObjectName {
    pub fn try_str(&self) -> Result<&str, std::str::Utf8Error> {
        self.try_into()
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for ObjectName {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl<const L: usize> From<&[u8; L]> for ObjectName {
    fn from(value: &[u8; L]) -> Self {
        Self(value.to_vec())
    }
}

impl<'a> TryInto<&'a str> for &'a ObjectName {
    type Error = std::str::Utf8Error;

    fn try_into(self) -> Result<&'a str, Self::Error> {
        std::str::from_utf8(&self.0)
    }
}

impl<'a> TryInto<String> for &ObjectName {
    type Error = std::str::Utf8Error;

    fn try_into(self) -> Result<String, Self::Error> {
        std::str::from_utf8(&self.0).map(|x| x.to_string())
    }
}

impl<'a> TryInto<String> for ObjectName {
    type Error = std::str::Utf8Error;

    fn try_into(self) -> Result<String, Self::Error> {
        std::str::from_utf8(&self.0).map(|x| x.to_string())
    }
}

pub(crate) mod caches {
    use hyper_ast::store::defaults::NodeIdentifier;

    use crate::preprocessed::IsSkippedAna;

    use super::ObjectName;

    pub(crate) type OidMap<T> = std::collections::BTreeMap<git2::Oid, T>;
    pub(crate) type NamedMap<T> = std::collections::BTreeMap<(git2::Oid, ObjectName), T>;

    #[derive(Default)]
    pub struct Java {
        pub(crate) md_cache: hyper_ast_gen_ts_java::legion_with_refs::MDCache,
        pub object_map: NamedMap<(hyper_ast_gen_ts_java::legion_with_refs::Local, IsSkippedAna)>,
    }

    impl super::ObjectMapper for Java {
        type K = (git2::Oid, ObjectName);

        type V = (hyper_ast_gen_ts_java::legion_with_refs::Local, IsSkippedAna);

        fn get(&self, key: &Self::K) -> Option<&Self::V> {
            self.object_map.get(key)
        }

        fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V> {
            self.object_map.insert(key, value)
        }
    }

    #[derive(Default)]
    pub struct Cpp {
        pub(crate) md_cache: hyper_ast_gen_ts_cpp::legion::MDCache,
        pub object_map: NamedMap<(hyper_ast_gen_ts_cpp::legion::Local, IsSkippedAna)>,
    }

    impl super::ObjectMapper for Cpp {
        type K = (git2::Oid, ObjectName);

        type V = (hyper_ast_gen_ts_cpp::legion::Local, IsSkippedAna);

        fn get(&self, key: &Self::K) -> Option<&Self::V> {
            self.object_map.get(key)
        }

        fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V> {
            self.object_map.insert(key, value)
        }
    }

    #[derive(Default)]
    pub struct Maven {
        pub object_map: OidMap<(NodeIdentifier, crate::maven::MD)>,
    }

    #[derive(Default)]
    pub struct Pom {
        pub object_map: OidMap<crate::maven::POM>,
    }

    impl super::ObjectMapper for Pom {
        type K = git2::Oid;

        type V = crate::maven::POM;

        fn get(&self, key: &Self::K) -> Option<&Self::V> {
            self.object_map.get(key)
        }

        fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V> {
            self.object_map.insert(key, value)
        }
    }

    #[derive(Default)]
    pub struct Make {
        pub object_map: OidMap<(NodeIdentifier, crate::make::MD)>,
    }

    #[derive(Default)]
    pub struct MakeFile {
        pub object_map: OidMap<crate::make::MakeFile>,
    }

    impl super::ObjectMapper for MakeFile {
        type K = git2::Oid;

        type V = crate::make::MakeFile;

        fn get(&self, key: &Self::K) -> Option<&Self::V> {
            self.object_map.get(key)
        }

        fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V> {
            self.object_map.insert(key, value)
        }
    }

    // // any
    // pub object_map_any: OidMap<(NodeIdentifier, DefaultMetrics)>,
    // // maven
    // #[cfg(feature = "maven")]
    // pub object_map_maven: OidMap<(NodeIdentifier, crate::maven::MD)>,
    // // make
    // #[cfg(feature = "make")]
    // pub object_map_make: OidMap<(NodeIdentifier, crate::make::MD)>,
    // // npm
    // #[cfg(feature = "npm")]
    // pub object_map_npm: OidMap<(NodeIdentifier, DefaultMetrics)>,

    // // pom.xml
    // #[cfg(feature = "maven")]
    // pub object_map_pom: OidMap<POM>,
    // // MakeFile
    // #[cfg(feature = "make")]
    // pub object_map_makefile: OidMap<MakeFile>,
    // // Java
    // #[cfg(feature = "java")]
    // pub(super) java_md_cache: java_tree_gen::MDCache,
    // #[cfg(feature = "java")]
    // pub object_map_java: NamedMap<(java_tree_gen::Local, IsSkippedAna)>,
    // // Cpp
    // #[cfg(feature = "cpp")]
    // pub(super) cpp_md_cache: cpp_tree_gen::MDCache,
    // #[cfg(feature = "cpp")]
    // pub object_map_cpp: NamedMap<(cpp_tree_gen::Local, IsSkippedAna)>,
}

/// A git Commit contains a Tree (ie. a directory in a file system) that contain other Trees and end with Blobs (ie. files).
/// It can follow a specific scheme,
/// and is often related to a specific build system or language.
pub mod file_sys {

    // TODO move these things to their respective modules
    use super::{CachesHolder, ObjectName};

    /// The default file system, directories and files
    pub struct Any;

    /// The maven scheme https://maven.apache.org/guides/introduction/introduction-to-the-standard-directory-layout.html ,
    /// made of nested maven modules.
    /// Each maven module has a config file (often a pom.xml),
    /// a src/main/java/ directory that contains production code for java,
    /// a src/test/java/ directory that contains tests for java,
    /// a src/test/resources/ directory that contains resources that should not be compiled (most of the time),
    /// ... (see ref.)
    #[cfg(feature = "maven")]
    pub struct Maven;

    impl CachesHolder for Maven {
        type Caches = super::caches::Maven;
    }

    #[cfg(feature = "maven")]
    pub struct Pom;

    #[cfg(feature = "maven")]
    impl CachesHolder for Pom {
        type Caches = super::caches::Pom;
    }

    impl super::InFiles for Pom {
        fn matches(name: &ObjectName) -> bool {
            name.0.eq(b"pom.xml")
        }
    }

    /// The java scheme,
    /// made of packages and modules https://docs.oracle.com/javase/specs/jls/se11/html/jls-7.html
    #[cfg(feature = "maven")]
    pub struct Java;

    impl CachesHolder for Java {
        type Caches = super::caches::Java;
    }

    impl super::InFiles for Java {
        fn matches(name: &ObjectName) -> bool {
            name.0.ends_with(b".java")
        }
    }

    /// The make scheme,
    /// It contains a Makefile and different directories, often src/ or lib/, tests/ or tests/, and also third-party/ docs/ script/,
    /// but it is mostly community and programming language dependent.
    #[cfg(feature = "make")]
    pub struct Make;

    impl CachesHolder for Make {
        type Caches = super::caches::Make;
    }

    #[cfg(feature = "make")]
    pub struct MakeFile;

    impl CachesHolder for MakeFile {
        type Caches = super::caches::MakeFile;
    }

    impl super::InFiles for MakeFile {
        fn matches(name: &ObjectName) -> bool {
            name.0.eq(b"Makefile")
        }
    }

    #[cfg(feature = "cpp")]
    pub struct Cpp;

    impl CachesHolder for Cpp {
        type Caches = super::caches::Cpp;
    }

    impl super::InFiles for Cpp {
        fn matches(name: &ObjectName) -> bool {
            name.0.ends_with(b".cpp")
                || name.0.ends_with(b".c")
                || name.0.ends_with(b".cxx")
                || name.0.ends_with(b".h")
                || name.0.ends_with(b".hpp")
        }
    }

    /// The npm scheme,
    /// it contains a package.json then,
    /// in its simplest form contains an index.js and a src/ directory,
    /// or is a collection of packages that contains a packages/ directory where each package is located
    #[cfg(feature = "npm")]
    pub struct Npm;
}

impl crate::preprocessed::RepositoryProcessor {
    pub fn intern_object_name<T: std::borrow::Borrow<ObjectName>>(
        &mut self,
        name: T,
    ) -> hyper_ast::store::defaults::LabelIdentifier {
        use hyper_ast::types::LabelStore;
        let s: &str = name.borrow().try_into().unwrap();
        self.main_stores.label_store.get_or_insert(s)
    }
}

pub(crate) mod erased_processor_collection {
    use std::any::Any;

    #[derive(Clone)]
    pub struct ConfigParameters(std::rc::Rc<dyn std::any::Any>);
    pub trait Parametrized: ParametrizedCommitProc {
        type T: 'static;
        // Register a parameter to later be used by the processor,
        // each identical (structurally) parameter should identify exactly one processor
        fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle;
    }
    pub struct ConfigParametersHandle(usize);
    pub struct ParametrizedCommitProcessorHandle(CommitProcessorHandle, ConfigParametersHandle);
    pub struct CommitProcessorHandle(std::any::TypeId);
    pub trait CommitProc {
        fn p(&mut self);

        fn process_commit(
            &mut self,
            repository: &git2::Repository,
            commit_oid: git2::Oid,
        ) -> crate::Commit {
            let builder =
                crate::preprocessed::CommitMonitoringBuilder::start(repository, commit_oid);
            let id = self.process_root_tree(repository, builder.tree_oid());
            builder.finish(id)
        }

        fn process_root_tree(
            &mut self,
            repository: &git2::Repository,
            tree_oid: git2::Oid,
        ) -> hyper_ast::store::defaults::NodeIdentifier;
    }
    pub trait ParametrizedCommitProc: std::any::Any {
        fn erased_handle(&self) -> CommitProcessorHandle
        where
            Self: 'static,
        {
            CommitProcessorHandle(std::any::TypeId::of::<Self>())
        }

        fn get(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc;
    }

    #[test]
    fn t() {
        #[derive(Clone, PartialEq, Eq)]
        struct S(u8);
        #[derive(Clone, PartialEq, Eq)]
        struct S0(u8);
        #[derive(Default)]
        struct P0(Vec<P>);
        struct P(S);
        impl Parametrized for P0 {
            type T = S;
            fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
                let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
                    let l = self.0.len();
                    self.0.push(P(t));
                    l
                });
                ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
            }
        }
        impl CommitProc for P {
            fn process_root_tree(
                &mut self,
                repository: &git2::Repository,
                tree_oid: git2::Oid,
            ) -> hyper_ast::store::defaults::NodeIdentifier {
                todo!()
            }

            fn p(&mut self) {
                dbg!()
            }
        }
        impl ParametrizedCommitProc for P0 {
            fn get(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc {
                &mut self.0[parameters.0]
            }
        }

        pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);
        impl<V> Default for ProcessorMap<V> {
            fn default() -> Self {
                Self(Default::default())
            }
        }

        unsafe impl<V: Send> Send for ProcessorMap<V> {}
        unsafe impl<V: Sync> Sync for ProcessorMap<V> {}

        // Should not need to be public
        pub trait ErasableProcessor: Any + ToErasedProc + ParametrizedCommitProc {}
        pub trait ToErasedProc {
            fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor>;
        }

        impl<T: ErasableProcessor> ToErasedProc for T {
            fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor> {
                self
            }
        }
        impl<T> ErasableProcessor for T where T: Any + ParametrizedCommitProc {}

        // NOTE crazy good stuff
        impl ProcessorMap<Box<dyn ErasableProcessor>> {
            pub fn by_id(
                &mut self,
                id: &std::any::TypeId,
            ) -> Option<&mut (dyn ErasableProcessor + 'static)> {
                self.0.get_mut(id).map(|x| x.as_mut())
            }
            pub fn mut_or_default<T: 'static + ToErasedProc + Default + Send + Sync>(
                &mut self,
            ) -> &mut T {
                let r = self
                    .0
                    .entry(std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new(T::default()).to_erasable_processor());
                let r = r.as_mut();
                let r = <dyn Any>::downcast_mut(r);
                r.unwrap()
            }
        }

        let mut h = ProcessorMap::<Box<dyn ErasableProcessor>>::default();
        // The registered parameter is type checked
        let hh = h.mut_or_default::<P0>().register_param(S(42));
        // You can easily store hh in any collection.
        // You can easily add a method to CommitProc.
        h.by_id(&hh.0 .0).unwrap().get(hh.1).p();
    }

    pub type ProcessorMap = spreaded::ProcessorMap<Box<dyn spreaded::ErasableProcessor>>;

    mod spreaded {
        use super::*;

        pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);
        impl<V> Default for ProcessorMap<V> {
            fn default() -> Self {
                Self(Default::default())
            }
        }
        impl<V> ProcessorMap<V> {
            pub(crate) fn clear(&mut self) {
                self.clear()
            }
        }

        unsafe impl<V: Send> Send for ProcessorMap<V> {}
        unsafe impl<V: Sync> Sync for ProcessorMap<V> {}

        // Should not need to be public
        pub trait ErasableProcessor: Any + ToErasedProc + ParametrizedCommitProc {}
        pub trait ToErasedProc {
            fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor>;
            fn as_mut_any(&mut self) -> &mut dyn Any;
            fn as_any(&self) -> &dyn Any;
        }

        impl<T: ErasableProcessor> ToErasedProc for T {
            fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor> {
                self
            }
            fn as_mut_any(&mut self) -> &mut dyn Any {
                self
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }
        impl<T> ErasableProcessor for T where T: Any + ParametrizedCommitProc {}
        // NOTE crazy good stuff
        impl ProcessorMap<Box<dyn ErasableProcessor>> {
            pub fn by_id(
                &mut self,
                id: &std::any::TypeId,
            ) -> Option<&mut (dyn ErasableProcessor + 'static)> {
                self.0.get_mut(id).map(|x| x.as_mut())
            }
            pub fn mut_or_default<T: 'static + ToErasedProc + Default + Send + Sync>(
                &mut self,
            ) -> &mut T {
                let r = self
                    .0
                    .entry(std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new(T::default()).to_erasable_processor());
                let r = r.as_mut();
                let r = <dyn Any>::downcast_mut(r.as_mut_any());
                r.unwrap()
            }
        }
        #[test]
        fn a() {
            #[derive(Clone, PartialEq, Eq)]
            struct S(u8);
            #[derive(Clone, PartialEq, Eq)]
            struct S0(u8);
            #[derive(Default)]
            struct P0(Vec<P>);
            struct P(S);
            impl Parametrized for P0 {
                type T = S;
                fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
                    let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
                        let l = self.0.len();
                        self.0.push(P(t));
                        l
                    });
                    ParametrizedCommitProcessorHandle(
                        self.erased_handle(),
                        ConfigParametersHandle(l),
                    )
                }
            }
            impl CommitProc for P {
                fn process_root_tree(
                    &mut self,
                    repository: &git2::Repository,
                    tree_oid: git2::Oid,
                ) -> hyper_ast::store::defaults::NodeIdentifier {
                    todo!()
                }

                fn p(&mut self) {
                    dbg!()
                }
            }
            impl ParametrizedCommitProc for P0 {
                fn get(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc {
                    &mut self.0[parameters.0]
                }
            }

            let mut h = ProcessorMap::<Box<dyn ErasableProcessor>>::default();
            // The registered parameter is type checked
            let hh = h.mut_or_default::<P0>().register_param(S(42));
            // You can easily store hh in any collection.
            // You can easily add a method to CommitProc.
            h.by_id(&hh.0 .0).unwrap().get(hh.1).p();
        }
    }
}

macro_rules! make_multi {
    ($($wb:tt)*) => {};
}

make_multi! {
    Java(Java, ),
    Pom,
    Cpp,
    MakeFile,
    Ts,
    Js,
    ;
    Maven [Java] Xml => crate::maven::Md,
    Make [Cpp] MakeFile => crate::make::Md,
    Npm [Ts, Js] Xml => crate::make::Md,
    None => crate::make::Md,
}
