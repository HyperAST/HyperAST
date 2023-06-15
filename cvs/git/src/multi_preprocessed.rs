use std::collections::{HashMap, HashSet};

use git2::Repository;
use hyper_ast::store::nodes::DefaultNodeIdentifier as NodeIdentifier;

use crate::{
    git::all_commits_between, maven::MavenModuleAcc, maven_processor::make,
    preprocessed::RepositoryProcessor, Commit, SimpleStores, MD,
};

/// Preprocess git repositories
/// share most components with PreProcessedRepository

#[derive(Default)]
pub struct PreProcessedRepositories {
    /// map repository names to some objects they contain (branches, references, commit).
    /// At least keeps roots
    pub repositories_by_name: HashMap<String, HashSet<git2::Oid>>,
    pub commits: HashMap<git2::Oid, Commit>,
    pub processor: RepositoryProcessor,
    // pub processing_ordered_commits: HashMap<String,Vec<git2::Oid>>,
}

pub(crate) type IsSkippedAna = bool;

impl PreProcessedRepositories {
    // pub fn name(&self) -> &str {
    //     &self.name
    // }
    pub fn new(name: &str) -> PreProcessedRepositories {
        let mut by_name = HashMap::default();
        by_name.insert(name.to_owned(), Default::default());
        PreProcessedRepositories {
            repositories_by_name: by_name,
            processor: Default::default(),
            commits: Default::default(),
        }
    }

    pub fn purge_caches(&mut self) {
        self.processor.purge_caches()
    }

    pub fn pre_process_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after)?;
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            .take(limit) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self
                    .processor
                    .handle_make_commit::<true>(&repository, dir_path, oid);
                processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
        Ok(processing_ordered_commits)
    }

    pub fn pre_process_with_config(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        config: ProcessingConfig<&'static str>,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after)?;
        match config {
            ProcessingConfig::JavaMaven { limit, dir_path } => {
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c =
                            self.processor
                                .handle_maven_commit::<true>(&repository, dir_path, oid);
                        processing_ordered_commits.push(oid.clone());
                        self.commits.insert(oid.clone(), c);
                    });
            }
            ProcessingConfig::CppMake { limit, dir_path } => {
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c =
                            self.processor
                                .handle_make_commit::<true>(&repository, dir_path, oid);
                        processing_ordered_commits.push(oid.clone());
                        self.commits.insert(oid.clone(), c);
                    });
            }
        }
        Ok(processing_ordered_commits)
    }

    pub fn make(acc: MavenModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
        make(acc, stores)
    }
}

pub enum ProcessingConfig<P> {
    JavaMaven { limit: usize, dir_path: P },
    CppMake { limit: usize, dir_path: P },
}

impl ProcessingConfig<&'static str> {
    pub fn java_maven(limit: usize) -> Self {
        Self::JavaMaven {
            limit,
            dir_path: "",
        }
    }
    pub fn cpp_make(limit: usize) -> Self {
        Self::CppMake {
            limit,
            dir_path: "src",
        }
    }
}
