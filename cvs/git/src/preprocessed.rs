use std::{
    collections::{BTreeMap, HashMap, HashSet},
    iter::Peekable,
    path::{Components, PathBuf},
    time::Instant,
};

use git2::{Oid, Repository};
use hyper_ast::{
    filter::{Bloom, BF},
    hashed::{self, SyntaxNodeHashs},
    store::{
        nodes::legion::{compo, EntryRef, NodeStore, CS},
        nodes::DefaultNodeIdentifier as NodeIdentifier,
    },
    tree_gen::SubTreeMetrics,
    types::{LabelStore as _, Labeled, Tree, Type, Typed, WithChildren},
    utils::memusage_linux,
};
use log::info;
use rusted_gumtree_gen_ts_java::{
    filter::BloomSize,
    impact::{element::RefPtr, partial_analysis::PartialAnalysis},
    java_tree_gen_full_compress_legion_ref::{self, hash32, BulkHasher},
    usage::declarations::IterDeclarationsUnstableOpti,
};

use crate::{
    git::{all_commits_between, retrieve_commit, BasicGitObjects},
    java::{handle_java_file, JavaAcc},
    maven::{handle_pom_file, IterMavenModules2, MavenModuleAcc, POM},
    Commit, SimpleStores, MAX_REFS, MD,
};
use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
use rusted_gumtree_gen_ts_xml::xml_tree_gen::XmlTreeGen;
use tuples::CombinConcat;

/// preprocess a git repository
/// using the hyperAST and caching git object transformations
/// for now only work with java with maven
pub struct PreProcessedRepository {
    name: String,
    pub(crate) main_stores: SimpleStores,
    java_md_cache: java_tree_gen::MDCache,
    pub object_map: BTreeMap<git2::Oid, (hyper_ast::store::nodes::DefaultNodeIdentifier, MD)>,
    pub object_map_pom: BTreeMap<git2::Oid, POM>,
    pub object_map_java: BTreeMap<(git2::Oid, Vec<u8>), (java_tree_gen::Local, bool)>,
    pub commits: HashMap<git2::Oid, Commit>,
    pub processing_ordered_commits: Vec<git2::Oid>,
}

impl PreProcessedRepository {
    pub fn main_stores(&mut self) -> &mut SimpleStores {
        &mut self.main_stores
    }

    fn is_handled(name: &Vec<u8>) -> bool {
        name.ends_with(b".java") || name.ends_with(b".xml")
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
        let line_break = if text.contains(&b"\r"[0]) {
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
        self.java_md_cache.clear()
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
            main_stores: SimpleStores::default(),
            java_md_cache: Default::default(),
            object_map: BTreeMap::default(),
            object_map_pom: BTreeMap::default(),
            object_map_java: BTreeMap::default(),
            commits: Default::default(),
            processing_ordered_commits: Default::default(),
        }
    }

    pub fn pre_process(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(40) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self.handle_maven_commit::<true>(&repository, dir_path, oid);
                self.processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
    }

    pub fn check_random_files_reserialization(
        &mut self,
        repository: &mut Repository,
        // before: &str,
        // after: &str,
        // dir_path: &str,
    ) -> (usize,usize) {

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
        repository.odb().unwrap().foreach(|&oid| {
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
        }).unwrap();
        let mut eq = 0;
        let mut not = 0;
        for oid in oids {
            let blob = repository.find_blob(oid).unwrap();
            if let Ok(_) = std::str::from_utf8(blob.content()) {
                // log::debug!("content: {}", z);
                let text = blob.content();
                if let Ok(full_node) = handle_java_file(&mut self.java_generator(text), b"", text) {
                    let mut out = BuffOut {
                        buff: "".to_owned(),
                    };
                    rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref::serialize(
                        &self.main_stores.node_store,
                        &self.main_stores.label_store,
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
        (eq,not)
    }

    pub fn pre_process_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            .take(limit) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self.handle_maven_commit::<true>(&repository, dir_path, oid);
                self.processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
    }

    pub fn pre_process_single(
        &mut self,
        repository: &mut Repository,
        ref_or_commit: &str,
        dir_path: &str,
    ) {
        let oid = retrieve_commit(repository, ref_or_commit).unwrap().id();
        let c = self.handle_maven_commit::<false>(&repository, dir_path, oid);
        self.processing_ordered_commits.push(oid.clone());
        self.commits.insert(oid.clone(), c);
    }

    pub fn pre_process_no_maven(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) {
        log::info!(
            "commits to process: {}",
            all_commits_between(&repository, before, after).count()
        );
        let rw = all_commits_between(&repository, before, after);
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            // .take(2)
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = self.handle_java_commit(&repository, dir_path, oid);
                self.processing_ordered_commits.push(oid.clone());
                self.commits.insert(oid.clone(), c);
            });
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

    /// module_path: path to wanted root module else ""
    fn handle_maven_commit<const RMS: bool>(
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
            self.handle_maven_module::<RMS>(repository, &mut dir_path, b"", tree.id());
        // let root_full_node = self.fast_fwd(repository, &mut dir_path, b"", tree.id()); // used to directly access specific java sources

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

        let root_full_node = self.fast_fwd(repository, &mut dir_path, b"", tree.id()); // used to directly access specific java sources

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

    fn fast_fwd(
        &mut self,
        repository: &Repository,
        mut dir_path: &mut Peekable<Components>,
        name: &[u8],
        oid: git2::Oid,
    ) -> (NodeIdentifier, MD) {
        let dir_hash = hash32(&Type::MavenDirectory);
        let root_full_node;
        let tree = repository.find_tree(oid).unwrap();

        /// sometimes order of files/dirs can be important, similarly to order of statement
        /// exploration order for example
        fn prepare_dir_exploration(
            tree: git2::Tree,
            dir_path: &mut Peekable<Components>,
        ) -> Vec<BasicGitObjects> {
            let mut children_objects: Vec<BasicGitObjects> = tree
                .iter()
                .map(TryInto::try_into)
                .filter_map(|x| x.ok())
                .collect();
            if dir_path.peek().is_none() {
                let p = children_objects.iter().position(|x| match x {
                    BasicGitObjects::Blob(_, n) => n.eq(b"pom.xml"),
                    _ => false,
                });
                if let Some(p) = p {
                    children_objects.swap(0, p); // priority to pom.xml processing
                    children_objects.reverse(); // we use it like a stack
                }
            }
            children_objects
        }
        let prepared = prepare_dir_exploration(tree, &mut dir_path);
        let mut stack: Vec<(Oid, Vec<BasicGitObjects>, MavenModuleAcc)> = vec![(
            oid,
            prepared,
            MavenModuleAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
        )];
        loop {
            if let Some(current_dir) = stack.last_mut().expect("never empty").1.pop() {
                match current_dir {
                    BasicGitObjects::Tree(x, name) => {
                        if let Some(s) = dir_path.peek() {
                            if name.eq(std::os::unix::prelude::OsStrExt::as_bytes(s.as_os_str())) {
                                dir_path.next();
                                stack.last_mut().expect("never empty").1.clear();
                                let tree = repository.find_tree(x).unwrap();
                                let prepared = prepare_dir_exploration(tree, &mut dir_path);
                                stack.push((
                                    x,
                                    prepared,
                                    MavenModuleAcc::new(
                                        std::str::from_utf8(&name).unwrap().to_string(),
                                    ),
                                ));
                                continue;
                            } else {
                                continue;
                            }
                        } else {
                            if let Some(already) = self.object_map.get(&x) {
                                // reinit already computed node for post order
                                let full_node = already.clone();

                                let name = self
                                    .main_stores()
                                    .label_store
                                    .get_or_insert(std::str::from_utf8(&name).unwrap());
                                let n = self.main_stores().node_store.resolve(full_node.0);
                                let already_name = *n.get_label();
                                if name != already_name {
                                    let already_name = self
                                        .main_stores()
                                        .label_store
                                        .resolve(&already_name)
                                        .to_string();
                                    let name = self.main_stores().label_store.resolve(&name);
                                    panic!("{} != {}", name, already_name);
                                } else if stack.is_empty() {
                                    root_full_node = full_node;
                                    break;
                                } else {
                                    let w = &mut stack.last_mut().unwrap().2;
                                    assert!(!w.children_names.contains(&name));
                                    w.push_submodule(name, full_node);
                                    continue;
                                }
                            }
                            let tree = repository.find_tree(x).unwrap();
                            let full_node = self.handle_java_src(repository, &name, tree.id());
                            let paren_acc = &mut stack.last_mut().unwrap().2;
                            let name = self
                                .main_stores()
                                .label_store
                                .get_or_insert(std::str::from_utf8(&name).unwrap());
                            assert!(!paren_acc.children_names.contains(&name));
                            paren_acc.push_source_directory(name, full_node);
                        }
                    }
                    BasicGitObjects::Blob(_, _) => {
                        continue;
                    }
                }
            } else if let Some((id, _, mut acc)) = stack.pop() {
                // commit node
                let hashed_label = hash32(&acc.name);
                let hsyntax = hashed::inner_node_hash(
                    &dir_hash,
                    &0,
                    &acc.metrics.size,
                    &acc.metrics.hashs.syntax,
                );
                let label = self
                    .main_stores()
                    .label_store
                    .get_or_insert(acc.name.clone());

                let eq = |x: EntryRef| {
                    let t = x.get_component::<Type>().ok();
                    if &t != &Some(&Type::MavenDirectory) {
                        return false;
                    }
                    let l = x.get_component::<java_tree_gen::LabelIdentifier>().ok();
                    if l != Some(&label) {
                        return false;
                    } else {
                        let cs = x.get_component::<Vec<NodeIdentifier>>().ok();
                        let r = cs == Some(&acc.children);
                        if !r {
                            return false;
                        }
                    }
                    true
                };
                let ana = {
                    let new_sub_modules = drain_filter_strip(&mut acc.sub_modules, b"..");
                    let new_main_dirs = drain_filter_strip(&mut acc.main_dirs, b"..");
                    let new_test_dirs = drain_filter_strip(&mut acc.test_dirs, b"..");
                    let ana = acc.ana;
                    if !new_sub_modules.is_empty()
                        || !new_main_dirs.is_empty()
                        || !new_test_dirs.is_empty()
                    {
                        log::error!(
                            "{:?} {:?} {:?}",
                            new_sub_modules,
                            new_main_dirs,
                            new_test_dirs
                        );
                        todo!("also prepare search for modules and sources in parent, should also tell from which module it is required");
                    }
                    // println!("refs in directory");
                    // println!("ref count in dir {}", ana.refs_count());
                    // ana.print_refs(self.main_stores().label_store);
                    // println!("decls in directory");
                    // ana.print_decls(self.main_stores().label_store);
                    ana.resolve()
                };
                // println!("ref count in dir after resolver {}", ana.refs_count());
                // println!("refs in directory after resolve");
                // ana.print_refs(self.main_stores().label_store);
                let insertion = self
                    .main_stores()
                    .node_store
                    .prepare_insertion(&hsyntax, eq);
                let hashs = SyntaxNodeHashs {
                    structt: hashed::inner_node_hash(
                        &dir_hash,
                        &0,
                        &acc.metrics.size,
                        &acc.metrics.hashs.structt,
                    ),
                    label: hashed::inner_node_hash(
                        &dir_hash,
                        &hashed_label,
                        &acc.metrics.size,
                        &acc.metrics.hashs.label,
                    ),
                    syntax: hsyntax,
                };
                let node_id = if let Some(id) = insertion.occupied_id() {
                    id
                } else {
                    log::info!("make mm {} {}", &acc.name, acc.children.len());
                    let vacant = insertion.vacant();
                    assert_eq!(acc.children_names.len(), acc.children.len());
                    NodeStore::insert_after_prepare(
                        vacant,
                        (
                            Type::MavenDirectory,
                            label,
                            hashs,
                            CS(acc.children_names),
                            CS(acc.children),
                            BloomSize::Much,
                        ),
                    )
                };

                // {
                //     let n = self.main_stores.node_store.resolve(node_id);
                //     if !n.has_children() {
                //         log::warn!(
                //             "z {} {:?} {:?} {:?} {:?}",
                //             n.get_component::<CS<NodeIdentifier>>().is_ok(),
                //             n.get_component::<CS<NodeIdentifier>>()
                //                 .map_or(&CS(vec![]), |x| x),
                //             n.get_component::<CS<NodeIdentifier>>().map(|x| x.0.len()),
                //             n.has_children(),
                //             n.get_component::<CS<NodeIdentifier>>()
                //                 .map(|x| !x.0.is_empty())
                //                 .unwrap_or(false)
                //         );
                //     }
                // }

                let metrics = SubTreeMetrics {
                    size: acc.metrics.size + 1,
                    height: acc.metrics.height + 1,
                    hashs,
                };

                let full_node = (
                    node_id.clone(),
                    MD {
                        metrics: metrics,
                        ana,
                    },
                );

                self.object_map.insert(id, full_node.clone());

                if stack.is_empty() {
                    root_full_node = full_node;
                    break;
                } else {
                    log::info!("dir: {}", &acc.name);
                    let w = &mut stack.last_mut().unwrap().2;
                    let name = self.main_stores().label_store.get_or_insert(acc.name);
                    assert!(!w.children_names.contains(&name));
                    w.push_submodule(name, full_node);
                }
            } else {
                panic!("never empty")
            }
        }
        root_full_node
    }

    /// RMS: resursive module search
    fn handle_maven_module<const RMS: bool>(
        &mut self,
        repository: &Repository,
        mut dir_path: &mut Peekable<Components>,
        name: &[u8],
        oid: git2::Oid,
    ) -> (NodeIdentifier, MD) {
        // use java_tree_gen::{hash32, EntryR, NodeIdentifier, NodeStore,};

        let dir_hash = hash32(&Type::MavenDirectory);
        let root_full_node;
        let tree = repository.find_tree(oid).unwrap();

        /// sometimes order of files/dirs can be important, similarly to order of statement
        /// exploration order for example
        fn prepare_dir_exploration(
            tree: git2::Tree,
            dir_path: &mut Peekable<Components>,
        ) -> Vec<BasicGitObjects> {
            let mut children_objects: Vec<BasicGitObjects> = tree
                .iter()
                .map(TryInto::try_into)
                .filter_map(|x| x.ok())
                .collect();
            if dir_path.peek().is_none() {
                let p = children_objects.iter().position(|x| match x {
                    BasicGitObjects::Blob(_, n) => n.eq(b"pom.xml"),
                    _ => false,
                });
                if let Some(p) = p {
                    children_objects.swap(0, p); // priority to pom.xml processing
                    children_objects.reverse(); // we use it like a stack
                }
            }
            children_objects
        }
        let prepared = prepare_dir_exploration(tree, &mut dir_path);
        let mut stack: Vec<(Oid, Vec<BasicGitObjects>, MavenModuleAcc)> = vec![(
            oid,
            prepared,
            MavenModuleAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
        )];
        loop {
            if let Some(current_dir) = stack.last_mut().expect("never empty").1.pop() {
                match current_dir {
                    BasicGitObjects::Tree(x, name) => {
                        if let Some(s) = dir_path.peek() {
                            if name.eq(std::os::unix::prelude::OsStrExt::as_bytes(s.as_os_str())) {
                                dir_path.next();
                                stack.last_mut().expect("never empty").1.clear();
                                let tree = repository.find_tree(x).unwrap();
                                let prepared = prepare_dir_exploration(tree, &mut dir_path);
                                stack.push((
                                    x,
                                    prepared,
                                    MavenModuleAcc::new(
                                        std::str::from_utf8(&name).unwrap().to_string(),
                                    ),
                                ));
                                continue;
                            } else {
                                continue;
                            }
                        }
                        // println!("h tree {:?}", std::str::from_utf8(&name));
                        // check if module or src/main/java or src/test/java
                        if let Some(already) = self.object_map.get(&x) {
                            // reinit already computed node for post order
                            let full_node = already.clone();

                            if stack.is_empty() {
                                root_full_node = full_node;
                                break;
                            } else {
                                let w = &mut stack.last_mut().unwrap().2;
                                let name = self
                                    .main_stores()
                                    .label_store
                                    .get_or_insert(std::str::from_utf8(&name).unwrap());
                                assert!(!w.children_names.contains(&name));
                                w.push_submodule(name, full_node);
                            }
                            continue;
                        }
                        // TODO use maven pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
                        // TODO maybe at some point try to handle maven modules and source dirs that reference parent directory in their path
                        log::info!("mm tree {:?}", std::str::from_utf8(&name));
                        let tree = repository.find_tree(x).unwrap();

                        let parent_acc = &mut stack.last_mut().unwrap().2;
                        // println!(
                        //     "{} source_dirs {:?}",
                        //     std::str::from_utf8(&name).unwrap(),
                        //     parent_acc.main_dirs
                        // );
                        let mut new_sub_modules =
                            drain_filter_strip(&mut parent_acc.sub_modules, &name);
                        let mut new_main_dirs =
                            drain_filter_strip(&mut parent_acc.main_dirs, &name);
                        let mut new_test_dirs =
                            drain_filter_strip(&mut parent_acc.test_dirs, &name);

                        // println!("matched source_dirs {:?}", new_main_dirs);

                        let is_source_dir = new_main_dirs
                            .drain_filter(|x| x.components().next().is_none())
                            .count()
                            > 0;
                        let is_test_source_dir = new_test_dirs
                            .drain_filter(|x| x.components().next().is_none())
                            .count()
                            > 0;
                        if is_source_dir || is_test_source_dir {
                            // handle as source dir
                            let full_node = self.handle_java_src(repository, &name, tree.id());
                            let paren_acc = &mut stack.last_mut().unwrap().2;
                            let name = self
                                .main_stores()
                                .label_store
                                .get_or_insert(std::str::from_utf8(&name).unwrap());
                            assert!(!paren_acc.children_names.contains(&name));
                            if is_source_dir {
                                paren_acc.push_source_directory(name, full_node);
                            } else {
                                // is_test_source_dir
                                paren_acc.push_test_source_directory(name, full_node);
                            }
                        }

                        let is_maven_module = new_sub_modules
                            .drain_filter(|x| x.components().next().is_none())
                            .count()
                            > 0;
                        // println!(
                        //     "{} {} {}",
                        //     is_source_dir, is_test_source_dir, is_maven_module
                        // );
                        // TODO check it we can use more info from context and prepare analysis more specifically
                        if is_maven_module
                            || !new_sub_modules.is_empty()
                            || !new_main_dirs.is_empty()
                            || !new_test_dirs.is_empty()
                        {
                            let prepared = prepare_dir_exploration(tree, &mut dir_path);
                            if is_maven_module {
                                // handle as maven module
                                stack.push((
                                    x,
                                    prepared,
                                    MavenModuleAcc::with_content(
                                        std::str::from_utf8(&name).unwrap().to_string(),
                                        new_sub_modules,
                                        new_main_dirs,
                                        new_test_dirs,
                                    ),
                                ));
                            } else {
                                // search further inside
                                stack.push((
                                    x,
                                    prepared,
                                    MavenModuleAcc::with_content(
                                        std::str::from_utf8(&name).unwrap().to_string(),
                                        new_sub_modules,
                                        new_main_dirs,
                                        new_test_dirs,
                                    ),
                                ));
                            };
                        } else if RMS && !(is_source_dir || is_test_source_dir) {
                            // anyway try to find maven modules, but maybe can do better
                            let prepared = prepare_dir_exploration(tree, &mut dir_path);
                            stack.push((
                                x,
                                prepared,
                                MavenModuleAcc::with_content(
                                    std::str::from_utf8(&name).unwrap().to_string(),
                                    new_sub_modules,
                                    new_main_dirs,
                                    new_test_dirs,
                                ),
                            ));
                        }
                    }
                    BasicGitObjects::Blob(x, name) => {
                        if dir_path.peek().is_some() {
                            continue;
                        } else if name.eq(b"pom.xml") {
                            if let Some(already) = self.object_map_pom.get(&x) {
                                // TODO reinit already computed node for post order
                                let full_node = already.clone();
                                let w = &mut stack.last_mut().unwrap().2;
                                let name = self
                                    .main_stores()
                                    .label_store
                                    .get_or_insert(std::str::from_utf8(&name).unwrap());
                                assert!(!w.children_names.contains(&name));
                                w.push_pom(name, full_node);
                                continue;
                            }
                            log::info!("blob {:?}", std::str::from_utf8(&name));
                            let a = repository.find_blob(x).unwrap();
                            if let Ok(z) = std::str::from_utf8(a.content()) {
                                log::error!("{:?} contains errors", std::str::from_utf8(&name));
                                // println!("content: {}", z);
                                let text = a.content();
                                let parent_acc = &mut stack.last_mut().unwrap().2;

                                // let g = XmlTreeGen {
                                //     line_break: "\n".as_bytes().to_vec(),
                                //     stores: self.main_stores,
                                // };
                                // let full_node =
                                //     handle_pom_file(&mut g, &name, text);
                                let full_node =
                                    handle_pom_file(&mut self.xml_generator(), &name, text);
                                let x = full_node.unwrap();
                                self.object_map_pom.insert(a.id(), x.clone());
                                let name = self
                                    .main_stores()
                                    .label_store
                                    .get_or_insert(std::str::from_utf8(&name).unwrap());
                                assert!(!parent_acc.children_names.contains(&name));
                                parent_acc.push_pom(name, x);
                            }
                        }
                    }
                }
            } else if let Some((id, _, mut acc)) = stack.pop() {
                // commit node
                let hashed_label = hash32(&acc.name);
                let hsyntax = hashed::inner_node_hash(
                    &dir_hash,
                    &0,
                    &acc.metrics.size,
                    &acc.metrics.hashs.syntax,
                );
                let label = self
                    .main_stores()
                    .label_store
                    .get_or_insert(acc.name.clone());

                let eq = |x: EntryRef| {
                    let t = x.get_component::<Type>().ok();
                    if &t != &Some(&Type::MavenDirectory) {
                        return false;
                    }
                    let l = x.get_component::<java_tree_gen::LabelIdentifier>().ok();
                    if l != Some(&label) {
                        return false;
                    } else {
                        let cs = x.get_component::<Vec<NodeIdentifier>>().ok();
                        let r = cs == Some(&acc.children);
                        if !r {
                            return false;
                        }
                    }
                    true
                };
                let ana = {
                    let new_sub_modules = drain_filter_strip(&mut acc.sub_modules, b"..");
                    let new_main_dirs = drain_filter_strip(&mut acc.main_dirs, b"..");
                    let new_test_dirs = drain_filter_strip(&mut acc.test_dirs, b"..");
                    let ana = acc.ana;
                    if !new_sub_modules.is_empty()
                        || !new_main_dirs.is_empty()
                        || !new_test_dirs.is_empty()
                    {
                        log::error!(
                            "{:?} {:?} {:?}",
                            new_sub_modules,
                            new_main_dirs,
                            new_test_dirs
                        );
                        todo!("also prepare search for modules and sources in parent, should also tell from which module it is required");
                    }
                    // println!("refs in directory");
                    // println!("ref count in dir {}", ana.refs_count());
                    // ana.print_refs(self.main_stores().label_store);
                    // println!("decls in directory");
                    // ana.print_decls(self.main_stores().label_store);
                    ana.resolve()
                };
                // println!("ref count in dir after resolver {}", ana.refs_count());
                // println!("refs in directory after resolve");
                // ana.print_refs(self.main_stores().label_store);
                let insertion = self
                    .main_stores()
                    .node_store
                    .prepare_insertion(&hsyntax, eq);
                let hashs = SyntaxNodeHashs {
                    structt: hashed::inner_node_hash(
                        &dir_hash,
                        &0,
                        &acc.metrics.size,
                        &acc.metrics.hashs.structt,
                    ),
                    label: hashed::inner_node_hash(
                        &dir_hash,
                        &hashed_label,
                        &acc.metrics.size,
                        &acc.metrics.hashs.label,
                    ),
                    syntax: hsyntax,
                };
                let node_id = if let Some(id) = insertion.occupied_id() {
                    id
                } else {
                    log::info!("make mm {} {}", &acc.name, acc.children.len());
                    let vacant = insertion.vacant();
                    assert_eq!(acc.children_names.len(), acc.children.len());
                    NodeStore::insert_after_prepare(
                        vacant,
                        (
                            Type::MavenDirectory,
                            label,
                            hashs,
                            CS(acc.children_names), // TODO extract dir names
                            CS(acc.children),
                            BloomSize::Much,
                        ),
                    )
                };

                // {
                //     let n = self.main_stores.node_store.resolve(node_id);
                //     if !n.has_children() {
                //         println!(
                //             "z {} {:?} {:?} {:?} {:?}",
                //             n.get_component::<CS<NodeIdentifier>>().is_ok(),
                //             n.get_component::<CS<NodeIdentifier>>()
                //                 .map_or(&CS(vec![]), |x| x),
                //             n.get_component::<CS<NodeIdentifier>>().map(|x| x.0.len()),
                //             n.has_children(),
                //             n.get_component::<CS<NodeIdentifier>>()
                //                 .map(|x| !x.0.is_empty())
                //                 .unwrap_or(false)
                //         );
                //     }
                // }

                let metrics = SubTreeMetrics {
                    size: acc.metrics.size + 1,
                    height: acc.metrics.height + 1,
                    hashs,
                };

                let full_node = (
                    node_id.clone(),
                    MD {
                        metrics: metrics,
                        ana,
                    },
                );

                self.object_map.insert(id, full_node.clone());

                if stack.is_empty() {
                    root_full_node = full_node;
                    break;
                } else {
                    let w = &mut stack.last_mut().unwrap().2;
                    let name = self.main_stores().label_store.get_or_insert(acc.name);
                    assert!(!w.children_names.contains(&name), "{:?}", name);
                    w.push_submodule(name, full_node);
                    // println!("dir: {}", &acc.name);
                }
            } else {
                panic!("never empty")
            }
        }
        root_full_node
    }

    /// oid : Oid of a dir surch that */src/main/java/ or */src/test/java/
    fn handle_java_src(
        &mut self,
        repository: &Repository,
        name: &[u8],
        oid: git2::Oid,
    ) -> java_tree_gen::Local {
        // use java_tree_gen::{hash32, EntryR, NodeIdentifier, NodeStore,};

        let dir_hash = hash32(&Type::Directory);

        let root_full_node;

        let tree = repository.find_tree(oid).unwrap();
        let prepared: Vec<BasicGitObjects> = tree
            .iter()
            .rev()
            .map(TryInto::try_into)
            .filter_map(|x| x.ok())
            .collect();
        let mut stack: Vec<(Oid, Vec<BasicGitObjects>, JavaAcc)> = vec![(
            oid,
            prepared,
            JavaAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
        )];
        loop {
            if let Some(current_dir) = stack.last_mut().expect("never empty").1.pop() {
                match current_dir {
                    BasicGitObjects::Tree(x, name) => {
                        if let Some((already, skiped_ana)) =
                            self.object_map_java.get(&(x, name.clone()))
                        {
                            // reinit already computed node for post order
                            let full_node = already.clone();

                            let name = self
                                .main_stores
                                .label_store
                                .get(std::str::from_utf8(&name).unwrap())
                                .unwrap();
                            let n = self
                                .main_stores
                                .node_store
                                .resolve(full_node.compressed_node);
                            let already_name = *n.get_label();
                            if name != already_name {
                                let already_name = self
                                    .main_stores()
                                    .label_store
                                    .resolve(&already_name)
                                    .to_string();
                                let name = self.main_stores().label_store.resolve(&name);
                                panic!("{} != {}", name, already_name);
                            } else if stack.is_empty() {
                                root_full_node = full_node;
                                break;
                            } else {
                                let w = &mut stack.last_mut().unwrap().2;
                                assert!(!w.children_names.contains(&name));
                                w.push_dir(name, full_node, *skiped_ana);
                            }
                            continue;
                        }
                        // TODO use maven pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
                        log::info!("tree {:?}", std::str::from_utf8(&name));
                        let a = repository.find_tree(x).unwrap();
                        let prepared: Vec<BasicGitObjects> = a
                            .iter()
                            .rev()
                            .map(TryInto::try_into)
                            .filter_map(|x| x.ok())
                            .collect();
                        stack.push((
                            x,
                            prepared,
                            JavaAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
                        ));
                    }
                    BasicGitObjects::Blob(x, name) => {
                        if !Self::is_handled(&name) {
                            continue;
                        } else if let Some((already, _)) =
                            self.object_map_java.get(&(x, name.clone()))
                        {
                            // TODO reinit already computed node for post order
                            let full_node = already.clone();

                            let name = self
                                .main_stores()
                                .label_store
                                .get_or_insert(std::str::from_utf8(&name).unwrap());
                            let n = self
                                .main_stores()
                                .node_store
                                .resolve(full_node.compressed_node);
                            let already_name = *n.get_label();
                            if name != already_name {
                                let already_name = self
                                    .main_stores()
                                    .label_store
                                    .resolve(&already_name)
                                    .to_string();
                                let name = self.main_stores().label_store.resolve(&name);
                                panic!("{} != {}", name, already_name);
                            } else if stack.is_empty() {
                                root_full_node = full_node;
                                break;
                            } else {
                                let w = &mut stack.last_mut().unwrap().2;
                                assert!(!w.children_names.contains(&name));
                                w.push(name, full_node);
                            }
                            continue;
                        }
                        log::info!("blob {:?}", std::str::from_utf8(&name));
                        // if std::str::from_utf8(&name).unwrap().eq("package-info.java") {
                        //     println!("module info:  {:?}", std::str::from_utf8(&name));
                        // } else
                        if std::str::from_utf8(&name).unwrap().ends_with(".java") {
                            let a = repository.find_blob(x).unwrap();
                            if let Ok(z) = std::str::from_utf8(a.content()) {
                                // log::debug!("content: {}", z);
                                let text = a.content();
                                if let Ok(full_node) =
                                    handle_java_file(&mut self.java_generator(text), &name, text)
                                {
                                    let full_node = full_node.local;
                                    // log::debug!("gen java");
                                    self.object_map_java
                                        .insert((a.id(), name.clone()), (full_node.clone(), false));
                                    let w = &mut stack.last_mut().unwrap().2;
                                    let name = self
                                        .main_stores()
                                        .label_store
                                        .get_or_insert(std::str::from_utf8(&name).unwrap());
                                    assert!(!w.children_names.contains(&name));
                                    w.push(name, full_node);
                                }
                            }
                        } else {
                            log::debug!("not java source file {:?}", std::str::from_utf8(&name));
                        }
                    }
                }
            } else if let Some((id, _, acc)) = stack.pop() {
                // commit node

                let hashed_label = hash32(&acc.name);

                let hsyntax = hashed::inner_node_hash(
                    &dir_hash,
                    &0,
                    &acc.metrics.size,
                    &acc.metrics.hashs.syntax,
                );
                let label = self
                    .main_stores()
                    .label_store
                    .get_or_insert(acc.name.clone());

                let eq = |x: EntryRef| {
                    let t = x.get_component::<Type>().ok();
                    if &t != &Some(&Type::Directory) {
                        return false;
                    }
                    let l = x.get_component::<java_tree_gen::LabelIdentifier>().ok();
                    if l != Some(&label) {
                        return false;
                    } else {
                        let cs = x.get_component::<Vec<NodeIdentifier>>().ok();
                        let r = cs == Some(&acc.children);
                        if !r {
                            return false;
                        }
                    }
                    true
                };
                let hashs = SyntaxNodeHashs {
                    structt: hashed::inner_node_hash(
                        &dir_hash,
                        &0,
                        &acc.metrics.size,
                        &acc.metrics.hashs.structt,
                    ),
                    label: hashed::inner_node_hash(
                        &dir_hash,
                        &hashed_label,
                        &acc.metrics.size,
                        &acc.metrics.hashs.label,
                    ),
                    syntax: hsyntax,
                };
                let ana = {
                    let ana = acc.ana;
                    let c = ana.estimated_refs_count();
                    if acc.skiped_ana {
                        log::info!(
                            "shop ana with at least {} refs",
                            ana.lower_estimate_refs_count()
                        );
                        ana
                    } else {
                        log::info!(
                            "ref count lower estimate in dir {}",
                            ana.lower_estimate_refs_count()
                        );
                        log::debug!("refs in directory");
                        for x in ana.display_refs(&self.main_stores().label_store) {
                            log::debug!("    {}", x);
                        }
                        log::debug!("decls in directory");
                        for x in ana.display_decls(&self.main_stores().label_store) {
                            log::debug!("    {}", x);
                        }
                        if c < MAX_REFS {
                            ana.resolve()
                        } else {
                            ana
                        }
                    }
                };
                log::info!(
                    "ref count in dir after resolver {}",
                    ana.lower_estimate_refs_count()
                );
                log::debug!("refs in directory after resolve: ");
                for x in ana.display_refs(&self.main_stores().label_store) {
                    log::debug!("    {}", x);
                }
                let insertion = self
                    .main_stores()
                    .node_store
                    .prepare_insertion(&hsyntax, eq);
                let node_id = if let Some(id) = insertion.occupied_id() {
                    id
                } else {
                    let vacant = insertion.vacant();
                    macro_rules! insert {
                        ( $c:expr, $t:ty ) => {{
                            let it = ana.solver.iter_refs();
                            let it =
                                BulkHasher::<_, <$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::from(it);
                            NodeStore::insert_after_prepare(
                                vacant,
                                $c.concat((<$t>::SIZE, <$t>::from(it))),
                            )
                        }
                            // NodeStore::insert_after_prepare(
                            //     vacant,
                            //     $c.concat((<$t>::SIZE, <$t>::from(ana.refs()))),
                            // )
                        };
                    }
                    // NodeStore::insert_after_prepare(
                    //     vacant,
                    //     (
                    //         Type::Directory,
                    //         label,
                    //         hashs,
                    //         CS(acc.children),
                    //         BloomSize::Much,
                    //     ),
                    // )
                    match acc.children.len() {
                        0 => NodeStore::insert_after_prepare(
                            vacant,
                            (Type::Directory, label, hashs, BloomSize::None),
                        ),
                        _ => {
                            assert_eq!(acc.children_names.len(), acc.children.len());
                            let c = (
                                Type::Directory,
                                label,
                                compo::Size(acc.metrics.size + 1),
                                compo::Height(acc.metrics.height + 1),
                                hashs,
                                CS(acc.children_names),
                                CS(acc.children),
                            );
                            match ana.estimated_refs_count() {
                                x if x > 2048 || acc.skiped_ana => NodeStore::insert_after_prepare(
                                    vacant,
                                    c.concat((BloomSize::Much,)),
                                ),
                                x if x > 1024 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 32]>)
                                }
                                x if x > 512 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 32]>)
                                }
                                x if x > 256 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 16]>)
                                    //1024
                                }
                                x if x > 150 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 8]>)
                                }
                                x if x > 100 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 4]>)
                                }
                                x if x > 30 => {
                                    insert!(c, Bloom::<&'static [u8], [u64; 2]>)
                                }
                                x if x > 15 => {
                                    insert!(c, Bloom::<&'static [u8], u64>)
                                }
                                x if x > 8 => {
                                    insert!(c, Bloom::<&'static [u8], u32>)
                                }
                                x if x > 0 => {
                                    insert!(c, Bloom::<&'static [u8], u16>)
                                }
                                _ => NodeStore::insert_after_prepare(
                                    vacant,
                                    c.concat((BloomSize::None,)),
                                ),
                            }
                        }
                    }
                };

                let metrics = java_tree_gen_full_compress_legion_ref::SubTreeMetrics {
                    size: acc.metrics.size + 1,
                    height: acc.metrics.height + 1,
                    hashs,
                };

                let full_node = java_tree_gen::Local {
                    compressed_node: node_id.clone(),
                    metrics,
                    ana: Some(ana.clone()),
                };
                self.object_map_java
                    .insert((id, name.to_vec()), (full_node.clone(), acc.skiped_ana));
                if stack.is_empty() {
                    root_full_node = full_node;
                    break;
                } else {
                    let w = &mut stack.last_mut().unwrap().2;
                    let name = self
                        .main_stores()
                        .label_store
                        .get_or_insert(acc.name.clone());
                    assert!(!w.children_names.contains(&name));
                    w.push_dir(name, full_node.clone(), acc.skiped_ana);
                    log::info!("dir: {}", &acc.name);
                }
            } else {
                panic!("never empty")
            }
        }
        root_full_node
    }
    pub fn child_by_name(&self, d: NodeIdentifier, name: &str) -> Option<NodeIdentifier> {
        let n = self.main_stores.node_store.resolve(d);
        n.get_child_by_name(&self.main_stores.label_store.get(name)?)
        // let s = n
        //     .get_children()
        //     .iter()
        //     .find(|x| {
        //         let n = self.main_stores.node_store.resolve(**x);

        //         if n.has_label() {
        //             self.main_stores.label_store.resolve(n.get_label()).eq(name)
        //         } else {
        //             false
        //         }
        //     })
        //     .map(|x| *x);
        // s
    }
    pub fn child_by_name_with_idx(
        &self,
        d: NodeIdentifier,
        name: &str,
    ) -> Option<(NodeIdentifier, usize)> {
        let n = self.main_stores.node_store.resolve(d);
        log::info!("{}", name);
        let i = n.get_child_idx_by_name(&self.main_stores.label_store.get(name)?);
        i.map(|i| (n.get_child(&i), i as usize))
        // let s = n
        //     .get_children()
        //     .iter()
        //     .enumerate()
        //     .find(|(_, x)| {
        //         let n = self.main_stores.node_store.resolve(**x);
        //         if n.has_label() {
        //             self.main_stores.label_store.resolve(n.get_label()).eq(name)
        //         } else {
        //             false
        //         }
        //     })
        //     .map(|(i, x)| (*x, i));
        // s
    }
    pub fn child_by_type(&self, d: NodeIdentifier, t: &Type) -> Option<(NodeIdentifier, usize)> {
        let n = self.main_stores.node_store.resolve(d);
        let s = n
            .get_children()
            .iter()
            .enumerate()
            .find(|(_, x)| {
                let n = self.main_stores.node_store.resolve(**x);
                n.get_type().eq(t)
            })
            .map(|(i, x)| (*x, i));
        s
    }
}

fn drain_filter_strip(v: &mut Option<Vec<PathBuf>>, name: &[u8]) -> Vec<PathBuf> {
    let mut new_sub_modules = vec![];
    let name = std::str::from_utf8(&name).unwrap();
    if let Some(sub_modules) = v {
        sub_modules
            .drain_filter(|x| x.starts_with(name))
            .for_each(|x| {
                let x = x.strip_prefix(name).unwrap().to_owned();
                new_sub_modules.push(x);
            });
    }
    new_sub_modules
}
