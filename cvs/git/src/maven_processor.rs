use std::{
    iter::Peekable,
    marker::PhantomData,
    path::{Components, PathBuf},
};

use git2::{Oid, Repository};
use hyper_ast::{store::defaults::NodeIdentifier, types::LabelStore};
use hyper_ast_gen_ts_xml::types::Type;

use crate::{
    git::{BasicGitObject, NamedObject, ObjectType, TypedObject},
    maven::{MavenModuleAcc, MD},
    preprocessed::RepositoryProcessor,
    processing::{erased::ParametrizedCommitProc2, CacheHolding, InFiles, ObjectName},
    Processor, SimpleStores,
};

/// RMS: Resursive Module Search
/// FFWD: Fast ForWarD to java directories without looking at maven stuff
pub struct MavenProcessor<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc> {
    prepro: &'b mut RepositoryProcessor,
    repository: &'a Repository,
    stack: Vec<(Oid, Vec<BasicGitObject>, Acc)>,
    dir_path: &'c mut Peekable<Components<'c>>,
    handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<MavenProc>,
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
        let h = prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>();
        let handle =
            <MavenProc as crate::processing::erased::CommitProcExt>::register_param(h, Parameter);
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree, &mut dir_path);
        let name = std::str::from_utf8(&name).unwrap().to_string();
        let stack = vec![(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            handle,
        }
    }
}

type Caches = <crate::processing::file_sys::Maven as crate::processing::CachesHolding>::Caches;

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool> Processor<MavenModuleAcc>
    for MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
{
    fn pre(&mut self, current_dir: BasicGitObject) {
        match current_dir {
            BasicGitObject::Tree(oid, name) => {
                self.handle_tree_cached(name, oid);
            }
            BasicGitObject::Blob(oid, name) => {
                if FFWD {
                    return;
                }
                if self.dir_path.peek().is_some() {
                    return;
                }
                if crate::processing::file_sys::Pom::matches(&name) {
                    self.prepro
                        .handle_pom(
                            oid,
                            &mut self.stack.last_mut().unwrap().2,
                            name,
                            &self.repository,
                            self.handle.into(),
                        )
                        .unwrap()
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: MavenModuleAcc) -> Option<(NodeIdentifier, MD)> {
        let name = acc.name.clone();
        let full_node = Self::make(acc, self.prepro.main_stores_mut());
        self.prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>()
            .get_caches_mut()
            .object_map
            .insert(oid, full_node.clone());

        let name = self.prepro.intern_label(&name);
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

    fn handle_tree_cached(&mut self, name: ObjectName, oid: Oid) {
        if let Some(s) = self.dir_path.peek() {
            if name
                .as_bytes()
                .eq(std::os::unix::prelude::OsStrExt::as_bytes(s.as_os_str()))
            {
                self.dir_path.next();
                self.stack.last_mut().expect("never empty").1.clear();
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                self.stack
                    .push((oid, prepared, MavenModuleAcc::new(name.try_into().unwrap())));
                return;
            } else {
                return;
            }
        }
        if let Some(already) = self
            .prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>()
            .get_caches_mut()
            .object_map
            .get(&oid)
        {
            // reinit already computed node for post order
            let full_node = already.clone();

            let w = &mut self.stack.last_mut().unwrap().2;
            let name = self.prepro.intern_object_name(&name);
            assert!(!w.children_names.contains(&name));
            w.push_submodule(name, full_node);
            return;
        }
        log::debug!("mm tree {:?}", name.try_str());
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
        let helper = MavenModuleHelper::from((parent_acc, &name));
        if helper.source_directories.0 || helper.test_source_directories.0 {
            // handle as source dir
            let (name, (full_node, _)) =
                self.prepro
                    .help_handle_java_folder(&self.repository, self.dir_path, oid, &name);
            let parent_acc = &mut self.stack.last_mut().unwrap().2;
            assert!(!parent_acc.children_names.contains(&name));
            if helper.source_directories.0 {
                parent_acc.push_source_directory(name, full_node);
            } else {
                // test_source_folders.0
                parent_acc.push_test_source_directory(name, full_node);
            }
        }
        // check if module or src/main/java or src/test/java
        // TODO use maven pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
        // TODO maybe at some point try to handle maven modules and source dirs that reference parent directory in their path

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
        } else if RMS && !(helper.source_directories.0 || helper.test_source_directories.0) {
            let tree = self.repository.find_tree(oid).unwrap();
            // anyway try to find maven modules, but maybe can do better
            let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
            self.stack.push((oid, prepared, helper.into()));
        }
    }
}

pub(crate) fn make(mut acc: MavenModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
    use hyper_ast::{
        filter::BloomSize,
        hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder},
        store::nodes::legion::{compo, compo::CS, NodeStore},
        tree_gen::SubTreeMetrics,
    };
    use hyper_ast_gen_ts_java::legion_with_refs::{eq_node, hash32};
    let dir_hash: u32 = hash32(&Type::MavenDirectory); // FIXME should be MavenDirectory ?
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
        if !new_sub_modules.is_empty() || !new_main_dirs.is_empty() || !new_test_dirs.is_empty() {
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

use hyper_ast_gen_ts_xml::legion::XmlTreeGen;
impl RepositoryProcessor {
    fn handle_pom(
        &mut self,
        oid: Oid,
        parent_acc: &mut MavenModuleAcc,
        name: ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<PomProc>,
    ) -> Result<(), crate::ParseErr> {
        let x = self
            .processing_systems
            .caching_blob_handler::<crate::processing::file_sys::Pom>()
            .handle(oid, repository, &name, parameters, |c, n, t| {
                crate::maven::handle_pom_file(
                    &mut XmlTreeGen {
                        line_break: "\n".as_bytes().to_vec(),
                        stores: &mut self.main_stores,
                    },
                    n,
                    t,
                )
            })?;
        // type Caches = <crate::processing::file_sys::Pom as crate::processing::CachesHolder>::Caches;
        // if let Some(already) = self
        //     .processing_systems
        //     .get::<Caches>()
        //     .and_then(|c| c.object_map.get(&oid))
        // {
        //     //.object_map_pom.get(&oid) {
        //     // TODO reinit already computed node for post order
        //     let full_node = already.clone();
        //     let name = self.intern_label(std::str::from_utf8(&name).unwrap());
        //     assert!(!parent_acc.children_names.contains(&name));
        //     parent_acc.push_pom(name, full_node);
        //     return;
        // }
        // log::info!("blob {:?}", std::str::from_utf8(&name));
        // let blob = repository.find_blob(oid).unwrap();
        // if std::str::from_utf8(blob.content()).is_err() {
        //     return;
        // }
        // let text = blob.content();
        // let full_node = self.handle_pom_file(&name, text);
        // let x = full_node.unwrap();
        // self.processing_systems
        //     .mut_or_default::<Caches>()
        //     .object_map
        //     .insert(oid, x.clone());
        let name = self.intern_object_name(&name);
        assert!(!parent_acc.children_names.contains(&name));
        parent_acc.push_pom(name, x);
        Ok(())
    }

    fn handle_pom_file(
        &mut self,
        name: &ObjectName,
        text: &[u8],
    ) -> Result<crate::maven::POM, crate::ParseErr> {
        crate::maven::handle_pom_file(&mut self.xml_generator(), name, text)
    }

    pub(crate) fn xml_generator(&mut self) -> XmlTreeGen<crate::TStore> {
        XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut self.main_stores,
        }
    }
}

struct MavenModuleHelper {
    name: String,
    submodules: (bool, Vec<PathBuf>),
    source_directories: (bool, Vec<PathBuf>),
    test_source_directories: (bool, Vec<PathBuf>),
}

impl From<(&mut MavenModuleAcc, &ObjectName)> for MavenModuleHelper {
    fn from((parent_acc, name): (&mut MavenModuleAcc, &ObjectName)) -> Self {
        let process = |mut v: &mut Option<Vec<PathBuf>>| {
            let mut v = drain_filter_strip(&mut v, name.as_bytes());
            let c = v.extract_if(|x| x.components().next().is_none()).count();
            (c > 0, v)
        };
        Self {
            name: name.try_into().unwrap(),
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
            .extract_if(|x| x.starts_with(name))
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
            ObjectType::File => crate::processing::file_sys::Pom::matches(x.name()),
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
            BasicGitObject::Blob(_, n) => crate::processing::file_sys::Pom::matches(n),
            _ => false,
        });
        if let Some(p) = p {
            children_objects.swap(0, p); // priority to pom.xml processing
            children_objects.reverse(); // we use it like a stack
        }
    }
    children_objects
}

// # Pom

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter;
impl From<crate::processing::erased::ParametrizedCommitProcessor2Handle<MavenProc>>
    for crate::processing::erased::ParametrizedCommitProcessor2Handle<PomProc>
{
    fn from(
        value: crate::processing::erased::ParametrizedCommitProcessor2Handle<MavenProc>,
    ) -> Self {
        crate::processing::erased::ParametrizedCommitProcessor2Handle(value.0, PhantomData)
    }
}
// #[derive(Default)]
struct PomProcessorHolder(Option<PomProc>);
impl Default for PomProcessorHolder {
    fn default() -> Self {
        Self(Some(PomProc {
            parameter: Parameter,
            cache: Default::default(),
        }))
    }
}
struct PomProc {
    parameter: Parameter,
    cache: crate::processing::caches::Pom,
}
impl crate::processing::erased::Parametrized for PomProcessorHolder {
    type T = Parameter;
    fn register_param(
        &mut self,
        t: Self::T,
    ) -> crate::processing::erased::ParametrizedCommitProcessorHandle {
        let l = self
            .0
            .iter()
            .position(|x| &x.parameter == &t)
            .unwrap_or_else(|| {
                let l = 0; //self.0.len();
                           // self.0.push(PomProc(t));
                self.0 = Some(PomProc {
                    parameter: t,
                    cache: Default::default(),
                });
                l
            });
        use crate::processing::erased::ConfigParametersHandle;
        use crate::processing::erased::ParametrizedCommitProc;
        use crate::processing::erased::ParametrizedCommitProcessorHandle;
        ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    }
}
impl crate::processing::erased::CommitProc for PomProc {
    fn process_root_tree(
        &mut self,
        repository: &git2::Repository,
        tree_oid: &git2::Oid,
    ) -> hyper_ast::store::defaults::NodeIdentifier {
        unimplemented!()
    }

    fn prepare_processing(
        &self,
        repository: &git2::Repository,
        commit_builder: crate::preprocessed::CommitBuilder,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc> {
        unimplemented!()
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        unimplemented!()
    }
}

impl crate::processing::erased::CommitProcExt for PomProc {
    type Holder = PomProcessorHolder;
}
impl crate::processing::erased::ParametrizedCommitProc2 for PomProcessorHolder {
    type Proc = PomProc;

    fn with_parameters_mut(
        &mut self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &mut Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_mut().unwrap()
    }

    fn with_parameters(
        &self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_ref().unwrap()
    }
}
impl CacheHolding<crate::processing::caches::Pom> for PomProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Pom {
        &mut self.cache
    }

    fn get_caches(&self) -> &crate::processing::caches::Pom {
        &self.cache
    }
}
impl CacheHolding<crate::processing::caches::Pom> for PomProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Pom {
        &mut self.0.as_mut().unwrap().cache
    }

    fn get_caches(&self) -> &crate::processing::caches::Pom {
        &self.0.as_ref().unwrap().cache
    }
}

// # Maven
#[derive(Default)]
pub struct MavenProcessorHolder(Option<MavenProc>);
pub struct MavenProc {
    parameter: Parameter,
    cache: crate::processing::caches::Maven,
    commits: std::collections::HashMap<git2::Oid, crate::Commit>,
}
impl crate::processing::erased::Parametrized for MavenProcessorHolder {
    type T = Parameter;
    fn register_param(
        &mut self,
        t: Self::T,
    ) -> crate::processing::erased::ParametrizedCommitProcessorHandle {
        let l = self
            .0
            .iter()
            .position(|x| &x.parameter == &t)
            .unwrap_or_else(|| {
                let l = 0; //self.0.len();
                           // self.0.push(MavenProc(t));
                self.0 = Some(MavenProc {
                    parameter: t,
                    cache: Default::default(),
                    commits: Default::default(),
                });
                l
            });
        use crate::processing::erased::ConfigParametersHandle;
        use crate::processing::erased::ParametrizedCommitProc;
        use crate::processing::erased::ParametrizedCommitProcessorHandle;
        ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    }
}

struct PreparedMavenCommitProc<'repo> {
    repository: &'repo git2::Repository,
    commit_builder: crate::preprocessed::CommitBuilder,
}
impl<'repo> crate::processing::erased::PreparedCommitProc for PreparedMavenCommitProc<'repo> {
    fn process(
        self: Box<PreparedMavenCommitProc<'repo>>,
        prepro: &mut RepositoryProcessor,
    ) -> hyper_ast::store::defaults::NodeIdentifier {
        let dir_path = PathBuf::from("");
        let mut dir_path = dir_path.components().peekable();
        let name = b"";
        // TODO check parameter in self to know it is a recusive module search
        let root_full_node = MavenProcessor::<true, false, MavenModuleAcc>::new(
            self.repository,
            prepro,
            &mut dir_path,
            name,
            self.commit_builder.tree_oid(),
        )
        .process();
        let h = prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>();
        let handle =
            <MavenProc as crate::processing::erased::CommitProcExt>::register_param(h, Parameter);
        let commit_oid = self.commit_builder.commit_oid();
        let commit = self.commit_builder.finish(root_full_node.0);
        h.with_parameters_mut(handle.0)
            .commits
            .insert(commit_oid, commit);
        root_full_node.0
    }
}
impl crate::processing::erased::CommitProc for MavenProc {
    fn process_root_tree(
        &mut self,
        repository: &git2::Repository,
        tree_oid: &git2::Oid,
    ) -> hyper_ast::store::defaults::NodeIdentifier {
        let dir_path = PathBuf::from("");
        let mut dir_path = dir_path.components().peekable();
        let name = b"";
        // TODO check parameter in self to know it is a recusive module search
        // let root_full_node = MavenProcessor::<true, false, MavenModuleAcc>::new(
        //     repository,
        //     self.prepro,
        //     &mut dir_path,
        //     name,
        //     tree_oid,
        // )
        // .process();
        // root_full_node.0
        unimplemented!("cannot access retrieve RepositoryProcessor as a CommitProc is likely part of it, double mutable borrow RIP")
    }

    fn prepare_processing<'repo>(
        &self,
        repository: &'repo git2::Repository,
        oids: crate::preprocessed::CommitBuilder,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc + 'repo> {
        Box::new(PreparedMavenCommitProc {
            repository,
            commit_builder: oids,
        })
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }
}

impl crate::processing::erased::CommitProcExt for MavenProc {
    type Holder = MavenProcessorHolder;
}
impl crate::processing::erased::ParametrizedCommitProc2 for MavenProcessorHolder {
    type Proc = MavenProc;

    fn with_parameters_mut(
        &mut self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &mut Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_mut().unwrap()
    }

    fn with_parameters(
        &self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_ref().unwrap()
    }
}
impl CacheHolding<crate::processing::caches::Maven> for MavenProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Maven {
        &mut self.cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Maven {
        &self.cache
    }
}
impl CacheHolding<crate::processing::caches::Maven> for MavenProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Maven {
        &mut self.0.as_mut().unwrap().cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Maven {
        &self.0.as_ref().unwrap().cache
    }
}
