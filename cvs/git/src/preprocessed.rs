use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
    path::{Components, PathBuf},
    time::{Duration, Instant},
    todo, usize,
};

use git2::{Oid, Repository};
use hyperast::{
    store::{defaults::LabelIdentifier, nodes::DefaultNodeIdentifier as NodeIdentifier},
    types::{AnyType, Childrn, LabelStore as _, WithChildren},
    utils::memusage,
};
use hyperast_gen_ts_java::legion_with_refs::PartialAnalysis;
use log::info;

use crate::{
    git::{all_commits_between, all_first_parents_between, retrieve_commit},
    make::MakeModuleAcc,
    make_processor::MakeProcessor,
    maven::MavenModuleAcc,
    maven_processor::MavenProcessor,
    processing::{file_sys, ConfiguredRepo2},
    Commit, DefaultMetrics, Processor, SimpleStores,
};
// use hyperast_gen_ts_cpp::legion as cpp_tree_gen;

/// Preprocess a git repository
/// using the hyperAST and caching git object transformations
/// for now only work with java & maven
/// Its only function should be to persist caches accoss processings
/// and exposing apis to hyperAST users/maker
pub struct PreProcessedRepository {
    pub name: String,
    pub commits: HashMap<git2::Oid, Commit>,

    pub processor: RepositoryProcessor,
}

#[derive(Default)]
pub struct RepositoryProcessor {
    pub main_stores: SimpleStores,
    pub processing_systems: crate::processing::erased::ProcessorMap,
    pub parsing_time: Duration,
    pub processing_time: Duration,
}
// NOTE what about making a constraints between sys processors
// it should be a 1..n relation so it must be impl on the target
// Examples:
// Any -> Java
// Any -> Maven when detecting a pom.xml
// Maven -> Java on source/ and test/ directories (also look at relevant fields in pom.xml)
// Any -> Make when detecting a Makefile
// Make -> Cpp on src/

pub(crate) type IsSkippedAna = bool;

trait GeneratorProvider<Generator> {
    fn generator(&mut self, text: &[u8]) -> Generator;
}

impl RepositoryProcessor {
    pub fn main_stores_mut(&mut self) -> &mut SimpleStores {
        &mut self.main_stores
    }
    pub fn main_stores(&self) -> &SimpleStores {
        &self.main_stores
    }

    pub fn intern_label(&mut self, name: &str) -> LabelIdentifier {
        self.main_stores.label_store.get_or_insert(name)
    }

    pub fn get_or_insert_label(
        &mut self,
        s: &str,
    ) -> hyperast::store::labels::DefaultLabelIdentifier {
        use hyperast::types::LabelStore;
        self.main_stores.label_store.get_or_insert(s)
    }

    pub fn print_refs(&self, ana: &PartialAnalysis) {
        #[cfg(feature = "impact")]
        ana.print_refs(&self.main_stores.label_store);
    }

    pub fn purge_caches(&mut self) {
        self.processing_systems.clear();
    }
}

impl PreProcessedRepository {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn new(name: &str) -> Self {
        let name = name.to_owned();
        Self {
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
    pub fn child_by_name_with_idx(
        &self,
        d: NodeIdentifier,
        name: &str,
    ) -> Option<(NodeIdentifier, u16)> {
        self.processor.child_by_name_with_idx(d, name)
    }
    pub fn child_by_type(&self, d: NodeIdentifier, t: &AnyType) -> Option<(NodeIdentifier, u16)> {
        self.processor.child_by_type(d, t)
    }
}
impl RepositoryProcessor {
    /// If `before` and `after` are unrelated then only one commit will be processed.
    pub(crate) fn pre_process(
        &mut self,
        repository: &mut ConfiguredRepo2,
        before: &str,
        after: &str,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository.repo, before, after).map(|x| x.count())
        );
        let rw = all_commits_between(&repository.repo, before, after)?;
        let mut rw = rw.map(|x| x.unwrap());
        let r = self.pre_pro(&mut rw, repository, usize::MAX);
        Ok(r)
    }

    /// If `before` and `after` are unrelated then only one commit will be retrieved.
    pub fn ensure_pre_processed_with_limit(
        &self,
        repository: &ConfiguredRepo2,
        before: &str,
        after: &str,
        limit: usize,
    ) -> Result<Result<Vec<git2::Oid>, Vec<git2::Oid>>, git2::Error> {
        log::info!(
            "commits to retrieve: {:?}",
            all_commits_between(&repository.repo, before, after).map(|x| x.count())
        );
        let rw = all_commits_between(&repository.repo, before, after)?;
        let mut rw = rw.map(|x| x.unwrap()).take(limit).peekable();
        let r = self.ensure_prepro(&mut rw, repository);
        Ok(r)
    }

    pub fn ensure_prepro(
        &self,
        rw: &mut Peekable<impl Iterator<Item = git2::Oid>>,
        repository: &ConfiguredRepo2,
    ) -> Result<Vec<Oid>, Vec<Oid>> {
        let mut r = vec![];
        loop {
            let Some(&oid) = rw.peek() else { break };
            let commit_processor = self
                .processing_systems
                .by_id(&repository.config.0)
                .unwrap()
                .get(repository.config.1);
            if let Some(c) = commit_processor.get_commit(oid) {
                rw.next();
                r.push(oid);
            } else {
                return Err(r);
            }
        }
        Ok(r)
    }

    /// If `before` and `after` are unrelated then only one commit will be processed.
    pub fn pre_process_with_limit(
        &mut self,
        repository: &ConfiguredRepo2,
        before: &str,
        after: &str,
        limit: usize,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process {before} {after}: {:?}",
            all_commits_between(&repository.repo, before, after).map(|x| x.count())
        );
        let rw = all_commits_between(&repository.repo, before, after)?;
        let mut rw = rw.take(limit).map(|x| x.unwrap());
        let r = self.pre_pro(&mut rw, repository, usize::MAX);
        Ok(r)
    }

    pub fn pre_pro(
        &mut self,
        rw: &mut impl Iterator<Item = git2::Oid>,
        repository: &ConfiguredRepo2,
        size: usize,
    ) -> Vec<Oid> {
        let mut r = Vec::with_capacity(rw.size_hint().0);
        for _ in 0..size {
            let Some(oid) = rw.next() else { break };
            let builder = crate::preprocessed::CommitBuilder::start(&repository.repo, oid);
            let commit_processor = self
                .processing_systems
                .by_id_mut(&repository.config.0)
                .unwrap()
                .get_mut(repository.config.1);
            let _id = commit_processor
                .prepare_processing(&repository.repo, builder, repository.config)
                .process(self);
            r.push(oid);
        }
        r
    }
}

#[cfg(feature = "maven_java")]
impl PreProcessedRepository {
    /// If `before` and `after` are unrelated then only one commit will be processed.
    pub fn pre_process(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        let Ok(rw) = rw else {
            dbg!(rw.err());
            return vec![];
        };
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(40) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = CommitProcessor::<file_sys::Maven>::handle_commit::<true>(
                    &mut self.processor,
                    &repository,
                    dir_path,
                    oid,
                );
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
        //     all_commits_between(&repository, before, after).map(|x|x.count())
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
                if let Ok(full_node) = self.processor.handle_java_file(&b"".into(), text)
                // handle_java_file(&mut self.processor.java_generator(text), b"", text)
                {
                    let out = hyperast::nodes::TextSerializer::new(
                        &self.processor.main_stores,
                        full_node.local.compressed_node,
                    )
                    .to_string();
                    println!("{}", out);
                    if std::str::from_utf8(text).unwrap() == out {
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

    /// Preprocess commits in `repository` between `before` and `after`.
    ///
    /// `limit`: the number of commits that will be preprocessed.
    /// `dir_path`: the subdirectory to consider for the analysis.
    ///
    /// If `before` and `after` are unrelated then only one commit will be processed.
    ///
    /// # Panics in debug mode
    ///
    /// Panics in debug mode if `before` and 'after' are unrelated.
    pub fn pre_process_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Vec<git2::Oid> {
        let count = all_commits_between(&repository, before, after).map(|x| x.count());
        log::info!("commits to process: {:?}", count);
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        let Ok(rw) = rw else {
            dbg!(rw.err());
            return vec![];
        };
        rw.take(limit).for_each(|oid| {
            let oid = oid.unwrap();
            let c = CommitProcessor::<file_sys::Maven>::handle_commit::<true>(
                &mut self.processor,
                &repository,
                dir_path,
                oid,
            );
            processing_ordered_commits.push(oid.clone());
            self.commits.insert(oid.clone(), c);
        });
        processing_ordered_commits
    }

    pub fn pre_process_first_parents_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Vec<git2::Oid> {
        let count = all_first_parents_between(&repository, before, after).map(|x| x.count());
        log::info!("commits to process: {:?}", count);
        let mut processing_ordered_commits = vec![];
        let rw = all_first_parents_between(&repository, before, after);
        let Ok(rw) = rw else {
            dbg!(rw.err());
            return vec![];
        };
        rw.take(limit).for_each(|oid| {
            let oid = oid.unwrap();
            let c = CommitProcessor::<file_sys::Maven>::handle_commit::<true>(
                &mut self.processor,
                &repository,
                dir_path,
                oid,
            );
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
        let c = CommitProcessor::<file_sys::Maven>::handle_commit::<false>(
            &mut self.processor,
            &repository,
            dir_path,
            oid,
        );
        self.commits.insert(oid.clone(), c);
        oid
    }
}

#[cfg(feature = "java")]
impl PreProcessedRepository {
    /// Preprocess commits in `repository` between `before` and `after`.
    ///
    /// `dir_path`: the subdirectory to consider for the analysis.
    ///
    /// If `before` and `after` are unrelated then only one commit will be processed.
    ///
    /// # Panics in debug mode
    ///
    /// Panics in debug mode if `before` and 'after' are unrelated.
    pub fn pre_process_no_maven(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        let Ok(rw) = rw else {
            dbg!(rw.err());
            return vec![];
        };
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(2)
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = CommitProcessor::<file_sys::Java>::handle_commit::<false>(
                    &mut self.processor,
                    &repository,
                    dir_path,
                    oid,
                );
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        processing_ordered_commits
    }
}

#[cfg(feature = "make_cpp")]
impl PreProcessedRepository {
    /// Preprocess commits in `repository` between `before` and `after`.
    ///
    /// `dir_path`: the subdirectory to consider for the analysis.
    ///
    /// If `before` and `after` are unrelated then only one commit will be processed.
    ///
    /// # Panics in debug mode
    ///
    /// Panics in debug mode if `before` and 'after' are unrelated.
    pub fn pre_process_make_project_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Vec<git2::Oid> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after);
        let Ok(rw) = rw else {
            dbg!(rw.err());
            return vec![];
        };
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            .take(limit) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = CommitProcessor::<file_sys::Make>::handle_commit::<false>(
                    &mut self.processor,
                    &repository,
                    dir_path,
                    oid,
                );
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        processing_ordered_commits
    }

    // TODO auto detect and selectect processor,
    // TODO pass processor as dyn param
    pub fn pre_process_make_project(
        &mut self,
        repository: &mut Repository,
        ref_or_commit: &str,
        dir_path: &str,
    ) -> git2::Oid {
        let oid = retrieve_commit(repository, ref_or_commit).unwrap().id();
        let c = CommitProcessor::<file_sys::Make>::handle_commit::<false>(
            &mut self.processor,
            &repository,
            dir_path,
            oid,
        );
        self.commits.insert(oid.clone(), c);
        oid
    }
}

pub(crate) trait CommitProcessor<Sys> {
    type Module: IdHolder<Id = NodeIdentifier>;
    /// How to handle a module in a commit eg. maven modules, cargo crate.
    ///
    /// In a codebase such module system can help with compile time.
    /// In rust to avoid loosing performances you might have to enable link time optimizations (lto).
    ///
    /// RMS: Resursive Module Search
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self::Module;

    /// How to handle a commit eg.
    ///
    /// * Maven: the structure of modules might need to be considered
    /// * Java: at the filesystem level there are 3 kinds of directories: main, tests, resources
    ///     where most of the time you do not compile resources and might not compile tests (while still needing to compile source to compile tests)
    fn handle_commit<const RMS: bool>(
        &mut self,
        repository: &Repository,
        module_path: &str,
        commit_oid: git2::Oid,
    ) -> Commit {
        let dir_path = PathBuf::from(module_path);
        let mut dir_path = dir_path.components().peekable();
        let builder = CommitBuilder::start(repository, commit_oid);
        let module = self.handle_module::<RMS>(repository, &mut dir_path, b"", builder.tree_oid());
        builder.finish(module.id())
    }
}

pub trait IdHolder {
    type Id;
    fn id(&self) -> Self::Id;
}

/// Help building a commit, also measure time and memory usage
///
/// WARN the memory usage is actually the diference of heap size between the start and end of processing,
/// and it would be biased by concurent building (not possible at the time of writing this warning)
pub struct CommitBuilder {
    commit_oid: git2::Oid,
    tree_oid: git2::Oid,
    parents: Vec<git2::Oid>,
    memory_used: hyperast_gen_ts_java::utils::MemoryUsage,
    time: Instant,
}

impl CommitBuilder {
    #[must_use]
    pub(crate) fn start(repository: &Repository, commit_oid: git2::Oid) -> Self {
        let commit = repository.find_commit(commit_oid).unwrap();
        let tree_oid = commit.tree().unwrap().id();

        let parents = commit.parents().into_iter().map(|x| x.id()).collect();

        info!("handle commit: {}", commit_oid);

        let memory_used = memusage();
        let time = Instant::now();
        Self {
            commit_oid,
            tree_oid,
            parents,
            time,
            memory_used,
        }
    }

    pub(crate) fn tree_oid(&self) -> git2::Oid {
        self.tree_oid
    }

    pub(crate) fn commit_oid(&self) -> git2::Oid {
        self.commit_oid
    }

    pub(crate) fn finish(self, ast_root: NodeIdentifier) -> Commit {
        let processing_time = self.time.elapsed().as_nanos();
        let memory_used = memusage() - self.memory_used;
        let memory_used = memory_used.into();
        let tree_oid = self.tree_oid;
        let parents = self.parents;

        Commit {
            parents,
            tree_oid,
            ast_root,
            processing_time,
            memory_used,
        }
    }
}

#[cfg(feature = "any")]
impl CommitProcessor<file_sys::Any> for RepositoryProcessor {
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> NodeIdentifier {
        let root_full_node = MavenProcessor::<RMS, false, MavenModuleAcc>::new(
            repository, self, dir_path, name, oid,
        )
        .process();
        root_full_node.0
    }
}

impl<H: IdHolder, T> IdHolder for (H, T) {
    type Id = H::Id;
    fn id(&self) -> Self::Id {
        self.0.id()
    }
}

impl IdHolder for NodeIdentifier {
    type Id = NodeIdentifier;
    fn id(&self) -> Self::Id {
        self.clone()
    }
}

#[cfg(feature = "maven")]
impl CommitProcessor<file_sys::Maven> for RepositoryProcessor {
    type Module = (NodeIdentifier, crate::maven::MD);
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self::Module {
        let root_full_node = MavenProcessor::<RMS, false, MavenModuleAcc>::new(
            repository,
            self,
            dir_path,
            name,
            oid,
            todo!("para"),
        )
        .process();
        // self.object_map_maven
        //     .insert(commit_oid, root_full_node.clone());
        root_full_node
    }
}
#[cfg(feature = "java")]
impl CommitProcessor<file_sys::Java> for RepositoryProcessor {
    type Module = (NodeIdentifier, crate::maven::MD);
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self::Module {
        let root_full_node = MavenProcessor::<RMS, true, MavenModuleAcc>::new(
            repository,
            self,
            dir_path,
            name,
            oid,
            todo!("para"),
        )
        .process();
        root_full_node
    }
}

#[cfg(feature = "make")]
impl CommitProcessor<file_sys::Make> for RepositoryProcessor {
    type Module = (NodeIdentifier, crate::make::MD);
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        repository: &'a Repository,
        dir_path: &'b mut Peekable<Components<'b>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self::Module {
        let root_full_node =
            MakeProcessor::<RMS, false, MakeModuleAcc>::new(repository, self, dir_path, name, oid, todo!("para"))
                .process();
        // self.object_map_make
        //     .insert(commit_oid, root_full_node.clone());
        root_full_node
    }
}

impl CommitProcessor<file_sys::Any> for RepositoryProcessor {
    type Module = (NodeIdentifier, DefaultMetrics);
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        _repository: &'a Repository,
        _dir_path: &'b mut Peekable<Components<'b>>,
        _name: &[u8],
        _oid: git2::Oid,
    ) -> Self::Module {
        todo!("still not sure how to dispatch")
    }
}

/// plan to work on all languges of the family of typesript ie. ts, js, tsx, jsx
/// - [ ] ts
/// - [ ] js
/// - [ ] tsx
/// - [ ] jsx
/// - [ ] d.ts
/// - [ ] various transpiler configs
///   - [ ] babel
///   - [ ] ts
#[cfg(feature = "npm")]
impl CommitProcessor<file_sys::Npm> for RepositoryProcessor {
    type Module = (NodeIdentifier, DefaultMetrics);
    fn handle_module<'a, 'b, const RMS: bool>(
        &mut self,
        _repository: &'a Repository,
        _dir_path: &'b mut Peekable<Components<'b>>,
        _name: &[u8],
        _oid: git2::Oid,
    ) -> Self::Module {
        todo!("need to implement NpmProcessor")
        // let root_full_node = NpmProcessor::<RMS, FFWD, NpmModuleAcc>::new(repository, self, dir_path, name, oid)
        //     .process();
        // // self.object_map_make
        // //     .insert(commit_oid, root_full_node.clone());
        // root_full_node.0
    }
}

impl RepositoryProcessor {
    pub fn child_by_name(&self, d: NodeIdentifier, name: &str) -> Option<NodeIdentifier> {
        child_by_name(&self.main_stores, d, name)
    }

    pub fn child_by_name_with_idx(
        &self,
        d: NodeIdentifier,
        name: &str,
    ) -> Option<(NodeIdentifier, u16)> {
        child_by_name_with_idx(&self.main_stores, d, name)
    }
    pub fn child_by_type(&self, d: NodeIdentifier, t: &AnyType) -> Option<(NodeIdentifier, u16)> {
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
) -> Option<(NodeIdentifier, u16)> {
    let n = stores.node_store.resolve(d);
    log::debug!("{}", name);
    let i = n.get_child_idx_by_name(&stores.label_store.get(name)?);
    i.map(|i| (n.child(&i).unwrap(), i))
}

pub fn child_by_type(
    stores: &SimpleStores,
    d: NodeIdentifier,
    t: &AnyType,
) -> Option<(NodeIdentifier, u16)> {
    let n = stores.node_store.resolve(d);
    let s = n
        .children()
        .unwrap()
        .iter_children()
        .enumerate()
        .find(|(_, x)| {
            let n = stores.node_store.resolve(*x);
            use hyperast::types::HyperAST;
            stores.resolve_type(x).eq(t)
        })
        .map(|(i, x)| (x, i as u16));
    s
}

pub fn child_at_path<'a>(
    stores: &SimpleStores,
    mut d: NodeIdentifier,
    path: impl Iterator<Item = &'a str>,
) -> Option<NodeIdentifier> {
    for name in path {
        if name.trim().is_empty() {
            continue;
        }
        let n = stores.node_store.resolve(d);
        d = n.get_child_by_name(&stores.label_store.get(name)?)?
    }
    Some(d)
}

pub fn child_at_path_tracked<'a>(
    stores: &SimpleStores,
    mut d: NodeIdentifier,
    path: impl Iterator<Item = &'a str>,
) -> Option<(NodeIdentifier, Vec<usize>)> {
    let mut offsets = vec![];
    for name in path {
        let n = stores.node_store.resolve(d);
        let idx = n.get_child_idx_by_name(&stores.label_store.get(name)?)?;
        d = n.child(&idx).unwrap();
        offsets.push(idx as usize);
    }
    Some((d, offsets))
}

// TODO try to separate processing from caching from git
#[cfg(test)]
#[allow(unused)]
mod experiments {
    use crate::Accumulator;

    use super::*;
    use hyperast_gen_ts_java::legion_with_refs as java_tree_gen;

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
        use std::collections::BTreeMap;

        use crate::maven::POM;

        use super::*;

        pub struct Maven<Id> {
            object_map: BTreeMap<
                Id,
                (
                    hyperast::store::nodes::DefaultNodeIdentifier,
                    crate::maven::MD,
                ),
            >,
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
