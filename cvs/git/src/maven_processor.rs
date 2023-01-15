use std::{
    iter::Peekable,
    path::{Components, PathBuf},
};

use git2::{Oid, Repository};
use hyper_ast::{
    filter::BloomSize,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder},
    store::{
        defaults::NodeIdentifier,
        nodes::legion::{compo, NodeStore, CS},
    },
    tree_gen::SubTreeMetrics,
    types::{LabelStore, Type},
};
use hyper_ast_gen_ts_java::legion_with_refs::{eq_node, hash32};

use crate::{
    git::{BasicGitObject, NamedObject, ObjectType, TypedObject},
    maven::{MavenModuleAcc, MD},
    preprocessed::RepositoryProcessor,
    Processor, SimpleStores,
};

pub struct MavenProcessor<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc> {
    prepro: &'b mut RepositoryProcessor,
    repository: &'a Repository,
    stack: Vec<(Oid, Vec<BasicGitObject>, Acc)>,
    dir_path: &'c mut Peekable<Components<'c>>,
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc: From<String>>
    MavenProcessor<'a, 'b, 'c, RMS, FFWD, Acc>
{
    pub fn new(
        repository: &'a Repository,
        prepro: &'b mut RepositoryProcessor,
        mut dir_path: &'c mut Peekable<Components<'c>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree, &mut dir_path);
        let name = std::str::from_utf8(&name).unwrap().to_string();
        let stack = vec![(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
        }
    }
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool> Processor<MavenModuleAcc>
    for MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
{
    fn pre(&mut self, current_dir: BasicGitObject) {
        match current_dir {
            BasicGitObject::Tree(oid, name) => {
                if let Some(s) = self.dir_path.peek() {
                    if name.eq(std::os::unix::prelude::OsStrExt::as_bytes(s.as_os_str())) {
                        self.dir_path.next();
                        self.stack.last_mut().expect("never empty").1.clear();
                        let tree = self.repository.find_tree(oid).unwrap();
                        let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                        self.stack.push((
                            oid,
                            prepared,
                            MavenModuleAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
                        ));
                        return;
                    } else {
                        return;
                    }
                }
                // check if module or src/main/java or src/test/java
                if let Some(already) = self.prepro.object_map.get(&oid) {
                    // reinit already computed node for post order
                    let full_node = already.clone();

                    let w = &mut self.stack.last_mut().unwrap().2;
                    let name = self
                        .prepro
                        .main_stores_mut()
                        .label_store
                        .get_or_insert(std::str::from_utf8(&name).unwrap());
                    assert!(!w.children_names.contains(&name));
                    w.push_submodule(name, full_node);
                    return;
                }
                // TODO use maven pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
                // TODO maybe at some point try to handle maven modules and source dirs that reference parent directory in their path
                log::debug!("mm tree {:?}", std::str::from_utf8(&name));

                let parent_acc = &mut self.stack.last_mut().unwrap().2;
                if FFWD {
                    let (name, (full_node, _)) = self.prepro.help_handle_java_folder(
                        &self.repository,
                        &mut self.dir_path,
                        oid,
                        &name,
                    );
                    assert!(!parent_acc.children_names.contains(&name));
                    parent_acc.push_source_directory(name, full_node);
                    return;
                }

                let helper = MavenModuleHelper::from((parent_acc, name.as_ref()));
                if helper.source_directories.0 || helper.test_source_directories.0 {
                    // handle as source dir
                    let (name, (full_node, _)) = self.prepro.help_handle_java_folder(
                        &self.repository,
                        self.dir_path,
                        oid,
                        &name,
                    );
                    let parent_acc = &mut self.stack.last_mut().unwrap().2;
                    assert!(!parent_acc.children_names.contains(&name));
                    if helper.source_directories.0 {
                        parent_acc.push_source_directory(name, full_node);
                    } else {
                        // test_source_folders.0
                        parent_acc.push_test_source_directory(name, full_node);
                    }
                }
                // TODO check it we can use more info from context and prepare analysis more specifically
                if helper.submodules.0
                    || !helper.submodules.1.is_empty()
                    || !helper.source_directories.1.is_empty()
                    || !helper.test_source_directories.1.is_empty()
                {
                    let tree = self.repository.find_tree(oid).unwrap();
                    let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                    if helper.submodules.0 {
                        // handle as maven module
                        self.stack.push((oid, prepared, helper.into()));
                    } else {
                        // search further inside
                        self.stack.push((oid, prepared, helper.into()));
                    };
                } else if RMS && !(helper.source_directories.0 || helper.test_source_directories.0)
                {
                    let tree = self.repository.find_tree(oid).unwrap();
                    // anyway try to find maven modules, but maybe can do better
                    let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                    self.stack.push((oid, prepared, helper.into()));
                }
            }
            BasicGitObject::Blob(oid, name) => {
                if FFWD {
                    return;
                }
                if self.dir_path.peek().is_some() {
                    return;
                }
                if name.eq(b"pom.xml") {
                    self.prepro.help_handle_pom(
                        oid,
                        &mut self.stack.last_mut().unwrap().2,
                        name,
                        &self.repository,
                    )
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: MavenModuleAcc) -> Option<(NodeIdentifier, MD)> {
        let name = acc.name.clone();
        let full_node = Self::make(acc, self.prepro.main_stores_mut());
        self.prepro.object_map.insert(oid, full_node.clone());

        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some(full_node)
        } else {
            let w = &mut self.stack.last_mut().unwrap().2;
            assert!(
                !w.children_names.contains(&name),
                "{:?} {:?}",
                w.children_names,
                name
            );
            w.push_submodule(name, full_node);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, MavenModuleAcc)> {
        &mut self.stack
    }
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool>
    MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
{
    fn make(acc: MavenModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
        make(acc, stores)
    }
}

pub(crate) fn make(mut acc: MavenModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
    let dir_hash: u32 = hash32(&Type::Directory); // FIXME should be MavenDirectory ?
    let hashs = acc.metrics.hashs;
    let size = acc.metrics.size + 1;
    let height = acc.metrics.height + 1;
    let size_no_spaces = acc.metrics.size_no_spaces + 1;
    let hbuilder = hashed::Builder::new(hashs, &dir_hash, &acc.name, size_no_spaces);
    let hashable = hbuilder.most_discriminating();
    let label = stores.label_store.get_or_insert(acc.name.clone());

    let eq = eq_node(&Type::MavenDirectory, Some(&label), &acc.children);
    let ana = {
        let new_sub_modules = drain_filter_strip(&mut acc.sub_modules, b"..");
        let new_main_dirs = drain_filter_strip(&mut acc.main_dirs, b"..");
        let new_test_dirs = drain_filter_strip(&mut acc.test_dirs, b"..");
        let ana = acc.ana;
        if !new_sub_modules.is_empty() || !new_main_dirs.is_empty() || !new_test_dirs.is_empty()
        {
            log::error!(
                "{:?} {:?} {:?}",
                new_sub_modules,
                new_main_dirs,
                new_test_dirs
            );
            todo!("also prepare search for modules and sources in parent, should also tell from which module it is required");
        }
        ana.resolve()
    };
    let insertion = stores.node_store.prepare_insertion(&hashable, eq);
    let hashs = hbuilder.build();
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
                compo::Size(size),
                compo::Height(height),
                compo::SizeNoSpaces(size_no_spaces),
                CS(acc.children_names.into_boxed_slice()), // TODO extract dir names
                CS(acc.children.into_boxed_slice()),
                BloomSize::Much,
            ),
        )
    };

    let metrics = SubTreeMetrics {
        size,
        height,
        hashs,
        size_no_spaces,
    };

    let full_node = (node_id.clone(), MD { metrics, ana });
    full_node
}

struct MavenModuleHelper {
    name: String,
    submodules: (bool, Vec<PathBuf>),
    source_directories: (bool, Vec<PathBuf>),
    test_source_directories: (bool, Vec<PathBuf>),
}

impl From<(&mut MavenModuleAcc, &[u8])> for MavenModuleHelper {
    fn from((parent_acc, name): (&mut MavenModuleAcc, &[u8])) -> Self {
        let process = |mut v: &mut Option<Vec<PathBuf>>| {
            let mut v = drain_filter_strip(&mut v, name);
            let c = v.drain_filter(|x| x.components().next().is_none()).count();
            (c > 0, v)
        };
        Self {
            name: std::str::from_utf8(name).unwrap().to_string(),
            submodules: process(&mut parent_acc.sub_modules),
            source_directories: process(&mut parent_acc.main_dirs),
            test_source_directories: process(&mut parent_acc.test_dirs),
        }
    }
}

impl From<MavenModuleHelper> for MavenModuleAcc {
    fn from(helper: MavenModuleHelper) -> Self {
        MavenModuleAcc::with_content(
            helper.name,
            helper.submodules.1,
            helper.source_directories.1,
            helper.test_source_directories.1,
        )
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

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool>
    MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
{
    pub fn prepare_dir_exploration<It>(tree: It) -> Vec<It::Item>
    where
        It: Iterator,
        It::Item: NamedObject + TypedObject,
    {
        let mut children_objects: Vec<_> = tree.collect();
        let p = children_objects.iter().position(|x| match x.r#type() {
            ObjectType::File => x.name().eq(b"pom.xml"),
            ObjectType::Dir => false,
        });
        if let Some(p) = p {
            children_objects.swap(0, p); // priority to pom.xml processing
            children_objects.reverse(); // we use it like a stack
        }
        children_objects
    }
}

/// sometimes order of files/dirs can be important, similarly to order of statement
/// exploration order for example
pub(crate) fn prepare_dir_exploration(
    tree: git2::Tree,
    dir_path: &mut Peekable<Components>,
) -> Vec<BasicGitObject> {
    let mut children_objects: Vec<BasicGitObject> = tree
        .iter()
        .map(TryInto::try_into)
        .filter_map(|x| x.ok())
        .collect();
    if dir_path.peek().is_none() {
        let p = children_objects.iter().position(|x| match x {
            BasicGitObject::Blob(_, n) => n.eq(b"pom.xml"),
            _ => false,
        });
        if let Some(p) = p {
            children_objects.swap(0, p); // priority to pom.xml processing
            children_objects.reverse(); // we use it like a stack
        }
    }
    children_objects
}
