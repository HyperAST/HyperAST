use std::{
    collections::{BTreeMap, HashMap, HashSet},
    iter::Peekable,
    path::{Components, PathBuf},
    time::Instant,
};

use git2::{Oid, Repository};
use hyper_ast::{
    store::{defaults::LabelIdentifier, nodes::DefaultNodeIdentifier as NodeIdentifier},
    types::{IterableChildren, LabelStore as _, Type, Typed, WithChildren},
    utils::memusage_linux,
};
use hyper_ast_gen_ts_java::impact::partial_analysis::PartialAnalysis;
use log::info;

use crate::{
    git::{all_commits_between, retrieve_commit},
    java::{handle_java_file, JavaAcc},
    java_processor::JavaProcessor,
    maven::{handle_pom_file, MavenModuleAcc, POM},
    maven_processor::MavenProcessor,
    Commit, Processor, SimpleStores, MD,
};
use hyper_ast_gen_ts_java::legion_with_refs as java_tree_gen;
use hyper_ast_gen_ts_xml::legion::XmlTreeGen;

/// Preprocess a git repository
/// using the hyperAST and caching git object transformations
/// for now only work with java & maven
/// Its only function should be to persist caches accoss processings
/// and exposing apis to hyperAST users/maker
pub struct PreProcessedRepository {
    pub name: String,
    pub commits: HashMap<git2::Oid, Commit>,

    pub processor: RepositoryProcessor,
    // pub main_stores: SimpleStores,

    // pub object_map: BTreeMap<git2::Oid, (hyper_ast::store::nodes::DefaultNodeIdentifier, MD)>,
    // pub object_map_pom: BTreeMap<git2::Oid, POM>,
    // pub(super) java_md_cache: java_tree_gen::MDCache,
    // pub object_map_java: BTreeMap<(git2::Oid, Vec<u8>), (java_tree_gen::Local, IsSkippedAna)>,
}

#[derive(Default)]
pub struct RepositoryProcessor {
    pub main_stores: SimpleStores,

    pub object_map: BTreeMap<git2::Oid, (hyper_ast::store::nodes::DefaultNodeIdentifier, MD)>,
    pub object_map_pom: BTreeMap<git2::Oid, POM>,
    pub(super) java_md_cache: java_tree_gen::MDCache,
    pub object_map_java: BTreeMap<(git2::Oid, Vec<u8>), (java_tree_gen::Local, IsSkippedAna)>,
}

pub(crate) type IsSkippedAna = bool;

impl RepositoryProcessor {
    pub fn main_stores_mut(&mut self) -> &mut SimpleStores {
        &mut self.main_stores
    }
    pub fn main_stores(&mut self) -> &SimpleStores {
        &self.main_stores
    }
    pub fn intern_label(&mut self, name: &str) -> LabelIdentifier {
        self.main_stores.label_store.get(name).unwrap()
    }

    pub fn get_or_insert_label(
        &mut self,
        s: &str,
    ) -> hyper_ast::store::labels::DefaultLabelIdentifier {
        use hyper_ast::types::LabelStore;
        self.main_stores.label_store.get_or_insert(s)
    }

    pub fn print_refs(&self, ana: &PartialAnalysis) {
        ana.print_refs(&self.main_stores.label_store);
    }

    fn xml_generator(&mut self) -> XmlTreeGen {
        XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut self.main_stores,
        }
    }

    fn java_generator(&mut self, text: &[u8]) -> java_tree_gen::JavaTreeGen {
        let line_break = if text.contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };
        java_tree_gen::JavaTreeGen {
            line_break,
            stores: &mut self.main_stores,
            md_cache: &mut self.java_md_cache,
        }
    }

    pub fn purge_caches(&mut self) {
        self.object_map.clear();
        self.object_map_java.clear();
        self.object_map_pom.clear();
        self.java_md_cache.clear();
    }
}

impl PreProcessedRepository {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn new(name: &str) -> PreProcessedRepository {
        let name = name.to_owned();
        PreProcessedRepository {
            name,
            commits: Default::default(),
            processor: Default::default(),
        }
    }

    pub fn purge_caches(&mut self) {
        self.processor.purge_caches()
    }

    pub fn child_by_name(&self, d: NodeIdentifier, name: &str) -> Option<NodeIdentifier> {
        self.processor.child_by_name(d, name)
    }

    pub fn child_by_name_with_idx(
        &self,
        d: NodeIdentifier,
        name: &str,
    ) -> Option<(NodeIdentifier, usize)> {
        self.processor.child_by_name_with_idx(d, name)
    }
    pub fn child_by_type(&self, d: NodeIdentifier, t: &Type) -> Option<(NodeIdentifier, usize)> {
        self.processor.child_by_type(d, t)
    }
    pub fn pre_process(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(40) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self
                    .processor
                    .handle_maven_commit::<true>(&repository, dir_path, oid);
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        processing_ordered_commits
    }

    pub fn check_random_files_reserialization(
        &mut self,
        repository: &mut Repository,
        // before: &str,
        // after: &str,
        // dir_path: &str,
    ) -> (usize, usize) {
        struct BuffOut {
            buff: String,
        }

        impl std::fmt::Write for BuffOut {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                Ok(self.buff.extend(s.chars()))
            }
        }
        // log::info!(
        //     "commits to process: {}",
        //     all_commits_between(&repository, before, after).count()
        // );
        // let rw = all_commits_between(&repository, before, after);
        let mut oids = HashSet::<_>::default();
        repository
            .odb()
            .unwrap()
            .foreach(|&oid| {
                // easy deterministic sampling of objects
                if (oid.as_bytes()[0] & 0b11000000) != 0 {
                    return true;
                }
                if let Ok(tree) = repository.find_tree(oid) {
                    tree.iter().for_each(|entry| {
                        let name = entry.name_bytes().to_owned();
                        if name.ends_with(b".java") {
                            oids.insert(entry.id());
                        }
                    })
                    //if let Ok(blob) = repository.find_blob(oid) {
                }
                true
            })
            .unwrap();
        let mut eq = 0;
        let mut not = 0;
        for oid in oids {
            let blob = repository.find_blob(oid).unwrap();
            if let Ok(_) = std::str::from_utf8(blob.content()) {
                // log::debug!("content: {}", z);
                let text = blob.content();
                if let Ok(full_node) =
                    handle_java_file(&mut self.processor.java_generator(text), b"", text)
                {
                    let mut out = BuffOut {
                        buff: "".to_owned(),
                    };
                    hyper_ast_gen_ts_java::legion_with_refs::serialize(
                        &self.processor.main_stores.node_store,
                        &self.processor.main_stores.label_store,
                        &full_node.local.compressed_node,
                        &mut out,
                        &std::str::from_utf8(&"\n".as_bytes().to_vec()).unwrap(),
                    );
                    if std::str::from_utf8(text).unwrap() == out.buff {
                        eq += 1;
                    } else {
                        not += 1;
                    }
                }
            }
        }
        // let set = HashSet
        // rw.for_each(|oid| {
        //     let oid = oid.unwrap();

        //     let commit = repository.find_commit(oid).unwrap();
        //     let tree = commit.tree().unwrap();
        //     tree.walk(git2::TreeWalkMode::PreOrder, callback);
        //     let c = self.handle_java_commit(&repository, dir_path, oid);
        //     todo!()
        // })
        // .collect()
        (eq, not)
    }

    pub fn pre_process_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            .take(limit) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self
                    .processor
                    .handle_maven_commit::<true>(&repository, dir_path, oid);
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        processing_ordered_commits
    }

    pub fn pre_process_single(
        &mut self,
        repository: &mut Repository,
        ref_or_commit: &str,
        dir_path: &str,
    ) -> git2::Oid {
        let oid = retrieve_commit(repository, ref_or_commit).unwrap().id();
        let c = self
            .processor
            .handle_maven_commit::<false>(&repository, dir_path, oid);
        self.commits.insert(oid.clone(), c);
        oid
    }

    pub fn pre_process_no_maven(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(2)
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self
                    .processor
                    .handle_java_commit(&repository, dir_path, oid);
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        processing_ordered_commits
    }

    // pub fn first(before: &str, after: &str) -> Diffs {
    //     todo!()
    // }

    // pub fn compute_diff(before: &str, after: &str) -> Diffs {
    //     todo!()
    // }

    // pub fn compute_impacts(diff: &Diffs) -> Impacts {
    //     todo!()
    // }

    // pub fn find_declaration(reff: ExplorableRef) {
    //     todo!()
    // }

    // pub fn find_references(decl: ExplorableDecl) {
    //     todo!()
    // }
}

impl RepositoryProcessor {
    /// module_path: path to wanted root module else ""
    pub(crate) fn handle_maven_commit<const RMS: bool>(
        &mut self,
        repository: &Repository,
        module_path: &str,
        commit_oid: git2::Oid,
    ) -> Commit {
        let dir_path = PathBuf::from(module_path);
        let mut dir_path = dir_path.components().peekable();
        let commit = repository.find_commit(commit_oid).unwrap();
        let tree = commit.tree().unwrap();

        info!("handle commit: {}", commit_oid);

        let memory_used = memusage_linux();
        let now = Instant::now();

        let root_full_node =
            self.handle_maven_module::<RMS, false>(repository, &mut dir_path, b"", tree.id());
        // let root_full_node = self.fast_fwd(repository, &mut dir_path, b"", tree.id()); // used to directly access specific java sources

        self.object_map.insert(commit_oid, root_full_node.clone());

        let processing_time = now.elapsed().as_nanos();
        let memory_used = memusage_linux() - memory_used;
        let memory_used = memory_used.into();

        Commit {
            meta_data: root_full_node.1,
            parents: commit.parents().into_iter().map(|x| x.id()).collect(),
            ast_root: root_full_node.0,
            processing_time,
            memory_used,
        }
    }

    fn handle_java_commit(
        &mut self,
        repository: &Repository,
        module_path: &str,
        commit_oid: git2::Oid,
    ) -> Commit {
        let dir_path = PathBuf::from(module_path);
        let mut dir_path = dir_path.components().peekable();
        let commit = repository.find_commit(commit_oid).unwrap();
        let tree = commit.tree().unwrap();

        info!("handle commit: {}", commit_oid);

        let memory_used = memusage_linux();
        let now = Instant::now();

        let root_full_node =
            self.handle_maven_module::<false, true>(repository, &mut dir_path, b"", tree.id()); // used to directly access specific java sources

        let processing_time = now.elapsed().as_nanos();
        let memory_used = memusage_linux() - memory_used;
        let memory_used = memory_used.into();

        Commit {
            meta_data: root_full_node.1,
            parents: commit.parents().into_iter().map(|x| x.id()).collect(),
            ast_root: root_full_node.0,
            processing_time,
            memory_used,
        }
    }

    /// RMS: Resursive Module Search
    /// FFWD: Fast ForWarD to java directories without looking at maven stuff
    fn handle_maven_module<'a, 'b, const RMS: bool, const FFWD: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> (NodeIdentifier, MD) {
        MavenProcessor::<RMS, FFWD, MavenModuleAcc>::new(repository, self, dir_path, name, oid)
            .process()
    }

    pub(crate) fn help_handle_java_folder<'a, 'b, 'c, 'd: 'c>(
        &'a mut self,
        repository: &'b Repository,
        dir_path: &'c mut Peekable<Components<'d>>,
        oid: Oid,
        name: &Vec<u8>,
    ) -> <JavaAcc as hyper_ast::tree_gen::Accumulator>::Node {
        let full_node = self.handle_java_directory(repository, dir_path, name, oid);
        let name = self
            .main_stores_mut()
            .label_store
            .get_or_insert(std::str::from_utf8(name).unwrap());
        (name, full_node)
    }

    pub(crate) fn help_handle_pom(
        &mut self,
        oid: Oid,
        parent_acc: &mut MavenModuleAcc,
        name: Vec<u8>,
        repository: &Repository,
    ) {
        if let Some(already) = self.object_map_pom.get(&oid) {
            // TODO reinit already computed node for post order
            let full_node = already.clone();
            let name = self
                .main_stores_mut()
                .label_store
                .get_or_insert(std::str::from_utf8(&name).unwrap());
            assert!(!parent_acc.children_names.contains(&name));
            parent_acc.push_pom(name, full_node);
            return;
        }
        log::info!("blob {:?}", std::str::from_utf8(&name));
        let blob = repository.find_blob(oid).unwrap();
        if std::str::from_utf8(blob.content()).is_err() {
            return;
        }
        let text = blob.content();
        let full_node = handle_pom_file(&mut self.xml_generator(), &name, text);
        let x = full_node.unwrap();
        self.object_map_pom.insert(blob.id(), x.clone());
        let name = self
            .main_stores_mut()
            .label_store
            .get_or_insert(std::str::from_utf8(&name).unwrap());
        assert!(!parent_acc.children_names.contains(&name));
        parent_acc.push_pom(name, x);
    }

    pub(crate) fn help_handle_java_file(
        &mut self,
        oid: Oid,
        w: &mut JavaAcc,
        name: Vec<u8>,
        repository: &Repository,
    ) {
        if let Some((already, skiped_ana)) = self.object_map_java.get(&(oid, name.clone())) {
            let full_node = already.clone();
            let skiped_ana = *skiped_ana;
            let name = self
                .main_stores_mut()
                .label_store
                .get_or_insert(std::str::from_utf8(&name).unwrap());
            assert!(!w.children_names.contains(&name));
            w.push(name, full_node, skiped_ana);
            return;
        }
        log::info!("blob {:?}", std::str::from_utf8(&name));
        let blob = repository.find_blob(oid).unwrap();
        if std::str::from_utf8(blob.content()).is_err() {
            return;
        }
        let text = blob.content();
        if let Ok(full_node) = handle_java_file(&mut self.java_generator(text), &name, text) {
            let full_node = full_node.local;
            let skiped_ana = false; // TODO ez upgrade to handle skipping in files
            self.object_map_java
                .insert((blob.id(), name.clone()), (full_node.clone(), skiped_ana));
            let name = self
                .main_stores_mut()
                .label_store
                .get_or_insert(std::str::from_utf8(&name).unwrap());
            assert!(!w.children_names.contains(&name));
            w.push(name, full_node, skiped_ana);
        }
    }

    /// oid : Oid of a dir such that */src/main/java/ or */src/test/java/
    fn handle_java_directory<'b, 'd: 'b>(
        &mut self,
        repository: &Repository,
        dir_path: &'b mut Peekable<Components<'d>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> (java_tree_gen::Local, IsSkippedAna) {
        JavaProcessor::<JavaAcc>::new(repository, self, dir_path, name, oid).process()
    }

    pub fn child_by_name(&self, d: NodeIdentifier, name: &str) -> Option<NodeIdentifier> {
        child_by_name(&self.main_stores, d, name)
    }

    pub fn child_by_name_with_idx(
        &self,
        d: NodeIdentifier,
        name: &str,
    ) -> Option<(NodeIdentifier, usize)> {
        child_by_name_with_idx(&self.main_stores, d, name)
    }
    pub fn child_by_type(&self, d: NodeIdentifier, t: &Type) -> Option<(NodeIdentifier, usize)> {
        child_by_type(&self.main_stores, d, t)
    }
}

pub fn child_by_name(
    stores: &SimpleStores,
    d: NodeIdentifier,
    name: &str,
) -> Option<NodeIdentifier> {
    let n = stores.node_store.resolve(d);
    n.get_child_by_name(&stores.label_store.get(name)?)
}

pub fn child_by_name_with_idx(
    stores: &SimpleStores,
    d: NodeIdentifier,
    name: &str,
) -> Option<(NodeIdentifier, usize)> {
    let n = stores.node_store.resolve(d);
    log::info!("{}", name);
    let i = n.get_child_idx_by_name(&stores.label_store.get(name)?);
    i.map(|i| (n.child(&i).unwrap(), i as usize))
}
pub fn child_by_type(
    stores: &SimpleStores,
    d: NodeIdentifier,
    t: &Type,
) -> Option<(NodeIdentifier, usize)> {
    let n = stores.node_store.resolve(d);
    let s = n
        .children()
        .unwrap()
        .iter_children()
        .enumerate()
        .find(|(_, x)| {
            let n = stores.node_store.resolve(**x);
            n.get_type().eq(t)
        })
        .map(|(i, x)| (*x, i));
    s
}

// TODO try to separate processing from caching from git
#[cfg(test)]
#[allow(unused)]
mod experiments {
    use crate::Accumulator;

    use super::*;

    pub struct PreProcessedRepository2 {
        name: String,
        pub(crate) main_stores: SimpleStores,

        pub commits: HashMap<git2::Oid, Commit>,
        pub processing_ordered_commits: Vec<git2::Oid>,

        maven: cache::Maven<(git2::Oid, Vec<u8>)>,
        pom: cache::Pom<(git2::Oid, Vec<u8>)>,
        java: cache::Java<(git2::Oid, Vec<u8>)>,
    }

    impl PreProcessedRepository2 {
        fn handle_maven_module<'a, 'b, const RMS: bool, const FFWD: bool>(
            &mut self,
            repository: &'a Repository,
            dir_path: &'b mut Peekable<Components<'b>>,
            name: &[u8],
            oid: git2::Oid,
        ) -> <MavenModuleAcc as Accumulator>::Unlabeled {
            processor_factory::ffwd::Maven {
                sources: &middle::MiddleWare { repository },
                maven: &mut self.maven,
                pom: &mut self.pom,
                java: &mut self.java,
                dir_path,
            };
            // MavenProcessor::<RMS, FFWD, MavenModuleAcc>::new(repository, self, dir_path, name, oid)
            //     .process()
            todo!()
        }
    }

    mod middle {
        use super::*;

        pub struct MiddleWare<'repo> {
            pub repository: &'repo Repository,
        }
    }

    mod cache {
        use super::*;

        pub struct Maven<Id> {
            object_map: BTreeMap<Id, (hyper_ast::store::nodes::DefaultNodeIdentifier, MD)>,
        }
        pub struct Pom<Id> {
            pub object_map_pom: BTreeMap<Id, POM>,
        }
        pub struct Java<Id> {
            java_md_cache: java_tree_gen::MDCache,
            object_map_java: BTreeMap<Id, (java_tree_gen::Local, IsSkippedAna)>,
        }
    }

    mod processor_factory {
        use super::*;

        pub mod ffwd {
            use super::*;
            use middle::MiddleWare;
            pub struct Maven<'a, 'b, 'd, 'c, Id> {
                pub sources: &'a MiddleWare<'a>,
                pub maven: &'b mut cache::Maven<Id>,
                pub pom: &'b mut cache::Pom<Id>,
                pub java: &'b mut cache::Java<Id>,
                pub dir_path: &'d mut Peekable<Components<'c>>,
            }
            pub struct Java<'a, 'd, 'c, Id> {
                pub java: &'a mut cache::Java<Id>,
                pub dir_path: &'d mut Peekable<Components<'c>>,
            }
        }

        pub struct Maven<'a, Id> {
            maven: &'a mut cache::Maven<Id>,
            pom: &'a mut cache::Pom<Id>,
            java: &'a mut cache::Java<Id>,
            // kotlin: &'a mut cached::Kotlin<Id>,
            // scala: &'a mut cached::Scala<Id>,
        }
    }
}
