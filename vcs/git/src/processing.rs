use git2::Repository;

use crate::git::Repo;

mod blob_caching;

pub mod erased;
pub use erased::ParametrizedCommitProcessorHandle;

pub enum BuildSystem {
    Maven,
    Make,
    Npm,
    None,
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

impl std::str::FromStr for RepoConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Cpp" => Self::CppMake,
            "cpp" => Self::CppMake,
            "Java" => Self::JavaMaven,
            "java" => Self::JavaMaven,
            "typescript" => Self::TsNpm,
            "javascript" => Self::TsNpm,
            "Ts" => Self::TsNpm,
            "ts" => Self::TsNpm,
            "any" => Self::Any,
            x => return Err(format!("'{}' is not anvailable config", x)),
        })
    }
}

impl From<&RepoConfig> for ProcessingConfig<&'static str> {
    fn from(value: &RepoConfig) -> Self {
        match value {
            RepoConfig::CppMake => Self::CppMake {
                limit: 3,
                dir_path: "",
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
    type Config;
    fn config(&self) -> &Self::Config;
}

pub struct ConfiguredRepoHandle {
    pub spec: Repo,
    pub config: RepoConfig,
}

// NOTE could have impl deref but it is a bad idea (see rust book, related to ownership)
impl ConfiguredRepoTrait for ConfiguredRepoHandle {
    fn spec(&self) -> &Repo {
        &self.spec
    }
    type Config = RepoConfig;
    fn config(&self) -> &Self::Config {
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

#[derive(Debug, Clone)]
pub struct ConfiguredRepoHandle2 {
    pub spec: Repo,
    pub config: ParametrizedCommitProcessorHandle,
}

// NOTE could have impl deref but it is a bad idea (see rust book, related to ownership)
impl ConfiguredRepoTrait for ConfiguredRepoHandle2 {
    fn spec(&self) -> &Repo {
        &self.spec
    }
    type Config = ParametrizedCommitProcessorHandle;

    fn config(&self) -> &Self::Config {
        &self.config
    }
}
impl ConfiguredRepoHandle2 {
    pub fn fetch(self) -> ConfiguredRepo2 {
        ConfiguredRepo2 {
            repo: self.spec.fetch(),
            spec: self.spec,
            config: self.config,
        }
    }
    pub fn nofetch(self) -> ConfiguredRepo2 {
        ConfiguredRepo2 {
            repo: self.spec.nofetch(),
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
    type Config = RepoConfig;
    fn config(&self) -> &Self::Config {
        &self.config
    }
}

pub struct ConfiguredRepo2 {
    pub spec: Repo,
    pub repo: Repository,
    pub config: ParametrizedCommitProcessorHandle,
}

impl ConfiguredRepoTrait for ConfiguredRepo2 {
    fn spec(&self) -> &Repo {
        &self.spec
    }
    type Config = ParametrizedCommitProcessorHandle;
    fn config(&self) -> &Self::Config {
        &self.config
    }
}

pub trait CachesHolding {
    /// WARN if you use the same cache type in multiple holders it mean that they are effectively shared caches
    /// TIPs use a wrapping type to protect againts inadvertent sharing
    type Caches;

    // fn mut_or_default(&mut self) -> &mut Self::Caches;
}

pub trait CacheHolding<Caches> {
    fn get_caches_mut(&mut self) -> &mut Caches;
    fn get_caches(&self) -> &Caches;
}

pub trait HoldedCache {
    type Holder: CacheHolding<Self>
    where
        Self: Sized;
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

    use hyperast::store::defaults::NodeIdentifier;

    use crate::preprocessed::IsSkippedAna;

    use super::ObjectName;

    pub(crate) type OidMap<T> = std::collections::BTreeMap<git2::Oid, T>;
    pub(crate) type NamedMap<T> = std::collections::BTreeMap<(git2::Oid, ObjectName), T>;

    #[derive(Default)]
    pub struct Java {
        pub(crate) md_cache: hyperast_gen_ts_java::legion_with_refs::MDCache,
        /// Passed to subtree builder when deriving different data (assumed to be incompatible).
        pub(crate) dedup: hyperast::store::nodes::legion::DedupMap,
        pub object_map: NamedMap<(hyperast_gen_ts_java::legion_with_refs::Local, IsSkippedAna)>,
    }

    impl super::ObjectMapper for Java {
        type K = (git2::Oid, ObjectName);

        type V = (hyperast_gen_ts_java::legion_with_refs::Local, IsSkippedAna);

        fn get(&self, key: &Self::K) -> Option<&Self::V> {
            self.object_map.get(key)
        }

        fn insert(&mut self, key: Self::K, value: Self::V) -> Option<Self::V> {
            self.object_map.insert(key, value)
        }
    }

    #[derive(Default)]
    pub struct Cpp {
        pub(crate) md_cache: hyperast_gen_ts_cpp::legion::MDCache,
        pub object_map: NamedMap<(hyperast_gen_ts_cpp::legion::Local, IsSkippedAna)>,
    }

    impl super::ObjectMapper for Cpp {
        type K = (git2::Oid, ObjectName);

        type V = (hyperast_gen_ts_cpp::legion::Local, IsSkippedAna);

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
    pub struct Makefile {
        pub object_map: OidMap<crate::make::MakeFile>,
    }

    impl super::ObjectMapper for Makefile {
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
    use super::{CachesHolding, ObjectName};

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

    impl CachesHolding for Maven {
        type Caches = super::caches::Maven;
    }

    #[cfg(feature = "maven")]
    pub struct Pom;

    #[cfg(feature = "maven")]
    impl CachesHolding for Pom {
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

    impl CachesHolding for Java {
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

    impl CachesHolding for Make {
        type Caches = super::caches::Make;
    }

    #[cfg(feature = "make")]
    pub struct MakeFile;

    impl CachesHolding for MakeFile {
        type Caches = super::caches::Makefile;
    }

    impl super::InFiles for MakeFile {
        fn matches(name: &ObjectName) -> bool {
            name.0.eq(b"Makefile")
        }
    }

    #[cfg(feature = "cpp")]
    pub struct Cpp;

    impl CachesHolding for Cpp {
        type Caches = super::caches::Cpp;
    }

    /// CAUTION about when you change this value,
    /// advice: change it only at the very begining
    #[doc(hidden)]
    pub static mut ONLY_SWITCHES: bool = false;

    impl super::InFiles for Cpp {
        fn matches(name: &ObjectName) -> bool {
            if unsafe { ONLY_SWITCHES } {
                name.0.ends_with(b"switches.h") || name.0.ends_with(b"switches.cc")
            } else {
                name.0.ends_with(b".cpp")
                    || name.0.ends_with(b".c")
                    || name.0.ends_with(b".cc")
                    || name.0.ends_with(b".cxx")
                    || name.0.ends_with(b".h")
                    || name.0.ends_with(b".hpp")
            }
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
    ) -> hyperast::store::defaults::LabelIdentifier {
        use hyperast::types::LabelStore;
        let s: &str = name.borrow().try_into().unwrap();
        self.main_stores.label_store.get_or_insert(s)
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
