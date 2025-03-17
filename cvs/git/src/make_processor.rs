use crate::processing::erased::ParametrizedCommitProcessor2Handle as PCP2Handle;
use crate::StackEle;
use crate::{
    git::{BasicGitObject, NamedObject, ObjectType, TypedObject},
    make::{MakeModuleAcc, MakePartialAnalysis, MD},
    preprocessed::RepositoryProcessor,
    processing::{
        erased::ParametrizedCommitProc2, CacheHolding, InFiles, ObjectName,
        ParametrizedCommitProcessorHandle,
    },
    Processor,
};
use git2::{Oid, Repository};
use hyperast::types::ETypeStore as _;
use hyperast::{
    hashed::{IndexingHashBuilder, MetaDataHashsBuilder},
    store::{defaults::NodeIdentifier, nodes::legion::eq_node},
    types::LabelStore,
};
use hyperast_gen_ts_xml::types::Type;
use std::{
    iter::Peekable,
    path::{Components, PathBuf},
};

pub type SimpleStores = hyperast::store::SimpleStores<hyperast_gen_ts_xml::types::TStore>;

pub struct MakeProcessor<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc> {
    prepro: &'b mut RepositoryProcessor,
    repository: &'a Repository,
    stack: Vec<StackEle<Acc>>,
    dir_path: &'c mut Peekable<Components<'c>>,
    handle: ParametrizedCommitProcessorHandle,
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc: From<String>>
    MakeProcessor<'a, 'b, 'c, RMS, FFWD, Acc>
{
    pub fn new(
        repository: &'a Repository,
        prepro: &'b mut RepositoryProcessor,
        mut dir_path: &'c mut Peekable<Components<'c>>,
        name: &[u8],
        oid: git2::Oid,
        handle: ParametrizedCommitProcessorHandle,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree, &mut dir_path);
        let name = std::str::from_utf8(&name).unwrap().to_string();
        let stack = vec![StackEle::new(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            handle,
        }
    }
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool> Processor<MakeModuleAcc>
    for MakeProcessor<'a, 'b, 'c, RMS, FFWD, MakeModuleAcc>
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
                if crate::processing::file_sys::MakeFile::matches(&name) {
                    self.prepro
                        .help_handle_makefile(
                            oid,
                            &mut self.stack.last_mut().unwrap().acc,
                            name,
                            &self.repository,
                            PCP2Handle(self.handle.1, std::marker::PhantomData),
                        )
                        .unwrap();
                } else if crate::processing::file_sys::Cpp::matches(&name) {
                    self.prepro
                        .help_handle_cpp_file2(
                            oid,
                            &mut self.stack.last_mut().unwrap().acc,
                            &name,
                            self.repository,
                            PCP2Handle(self.handle.1, std::marker::PhantomData),
                        )
                        .unwrap();
                // } else if name.ends_with(b".h") || name.ends_with(b".hpp") {
                //     self.prepro.help_handle_cpp_file2(
                //         oid,
                //         &mut self.stack.last_mut().unwrap().acc,
                //         name,
                //         self.repository,
                //     )
                } else {
                    log::debug!("not cpp source file {:?}", name.try_str());
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: MakeModuleAcc) -> Option<(NodeIdentifier, MD)> {
        let name = acc.primary.name.clone();
        let full_node = Self::make(acc, self.prepro.main_stores_mut().mut_with_ts());
        self.prepro
            .processing_systems
            .mut_or_default::<MakeProcessorHolder>()
            .get_caches_mut()
            .object_map
            .insert(oid, full_node.clone());

        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some(full_node)
        } else {
            let w = &mut self.stack.last_mut().unwrap().acc;
            assert!(
                !w.primary.children_names.contains(&name),
                "{:?} {:?}",
                w.primary.children_names,
                name
            );
            w.push_submodule(name, full_node);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<StackEle<MakeModuleAcc>> {
        &mut self.stack
    }
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool>
    MakeProcessor<'a, 'b, 'c, RMS, FFWD, MakeModuleAcc>
{
    fn make(acc: MakeModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
        make(acc, stores)
    }

    fn handle_tree_cached(&mut self, name: ObjectName, oid: Oid) {
        if let Some(s) = self.dir_path.peek() {
            if name
                .as_bytes()
                .eq(std::ffi::OsStr::as_encoded_bytes(s.as_os_str()))
            {
                self.dir_path.next();
                self.stack.last_mut().expect("never empty").cs.clear();
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                self.stack
                    .push(StackEle::new(oid, prepared, MakeModuleAcc::new(name.try_into().unwrap())));
                return;
            } else {
                return;
            }
        }
        let mut make_proc = self
            .prepro
            .processing_systems
            .mut_or_default::<MakeProcessorHolder>()
            .with_parameters_mut(self.handle.1);
        let cpp_handle = make_proc.parameter.cpp_handle;
        if let Some(already) = make_proc.get_caches_mut().object_map.get(&oid) {
            // reinit already computed node for post order
            let full_node = already.clone();
            let w = &mut self.stack.last_mut().unwrap().acc;
            let name = self.prepro.intern_object_name(name);
            assert!(!w.primary.children_names.contains(&name));
            w.push_submodule(name, full_node);
            return;
        }
        log::debug!("make tree {:?}", name.try_str());
        let parent_acc = &mut self.stack.last_mut().unwrap().acc;
        if true {
            // TODO also try to handle nested Makefiles
            let (name, (full_node, _)) = self.prepro.help_handle_cpp_folder(
                &self.repository,
                &mut self.dir_path,
                oid,
                &name,
                cpp_handle,
            );
            assert!(!parent_acc.primary.children_names.contains(&name));
            parent_acc.push_source_directory(name, full_node);
            return;
        }
        let helper = MakeModuleHelper::from((parent_acc, &name));
        if helper.source_directories.0 || helper.test_source_directories.0 {
            // handle as source dir
            let (name, (full_node, _)) = self.prepro.help_handle_cpp_folder(
                &self.repository,
                self.dir_path,
                oid,
                &name,
                cpp_handle,
            );
            let parent_acc = &mut self.stack.last_mut().unwrap().acc;
            assert!(!parent_acc.primary.children_names.contains(&name));
            if helper.source_directories.0 {
                parent_acc.push_source_directory(name, full_node);
            } else {
                // test_source_folders.0
                parent_acc.push_test_source_directory(name, full_node);
            }
        }
        // check if module or src/main/java or src/test/java
        // TODO use Make pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
        // TODO maybe at some point try to handle Make modules and source dirs that reference parent directory in their path

        // TODO check it we can use more info from context and prepare analysis more specifically
        if helper.submodules.0
            || !helper.submodules.1.is_empty()
            || !helper.source_directories.1.is_empty()
            || !helper.test_source_directories.1.is_empty()
        {
            let tree = self.repository.find_tree(oid).unwrap();
            let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
            if helper.submodules.0 {
                // handle as Make module
                self.stack.push(StackEle::new(oid, prepared, helper.into()));
            } else {
                // search further inside
                self.stack.push(StackEle::new(oid, prepared, helper.into()));
            };
        } else if RMS && !(helper.source_directories.0 || helper.test_source_directories.0) {
            let tree = self.repository.find_tree(oid).unwrap();
            // anyway try to find Make modules, but maybe can do better
            let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
            self.stack.push(StackEle::new(oid, prepared, helper.into()));
        }
    }
}

pub(crate) fn make(acc: MakeModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
    let kind = Type::Directory;
    let interned_kind = hyperast_gen_ts_xml::types::TStore::intern(kind);
    let label_id = stores.label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));

    let hashable = primary.metrics.hashs.most_discriminating();

    let eq = eq_node(&interned_kind, Some(&label_id), &primary.children);

    assert_eq!(primary.children_names.len(), primary.children.len());
    let ana = MakePartialAnalysis::new();

    let insertion = stores.node_store.prepare_insertion(&hashable, eq);
    if let Some(id) = insertion.occupied_id() {
        let metrics = primary
            .metrics
            .map_hashs(|h| MetaDataHashsBuilder::build(h));
        return (id, MD { metrics, ana });
    }

    log::info!("make mm {} {}", &primary.name, primary.children.len());

    let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    let children_is_empty = primary.children.is_empty();

    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    let vacant = insertion.vacant();
    let node_id = hyperast::store::nodes::legion::NodeStore::insert_built_after_prepare(
        vacant,
        dyn_builder.build(),
    );

    let full_node = (node_id.clone(), MD { metrics, ana });
    full_node
}

use hyperast_gen_ts_xml::{legion::XmlTreeGen, types::XmlEnabledTypeStore as _};
impl RepositoryProcessor {
    fn help_handle_makefile(
        &mut self,
        oid: Oid,
        parent_acc: &mut MakeModuleAcc,
        name: ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<MakefileProc>,
    ) -> Result<(), crate::ParseErr> {
        let x = self
            .processing_systems
            .caching_blob_handler::<crate::processing::file_sys::MakeFile>()
            .handle(oid, repository, &name, parameters, |c, n, t| {
                crate::make::handle_makefile_file(
                    &mut XmlTreeGen {
                        line_break: "\n".as_bytes().to_vec(),
                        stores: self.main_stores.mut_with_ts(),
                    },
                    n,
                    t,
                )
                .map_err(|_| crate::ParseErr::IllFormed)
            })?;
        let name = self.intern_object_name(&name);
        assert!(!parent_acc.primary.children_names.contains(&name));
        parent_acc.push_makefile(name, x);
        Ok(())
    }
}

struct MakeModuleHelper {
    name: String,
    submodules: (bool, Vec<PathBuf>),
    source_directories: (bool, Vec<PathBuf>),
    test_source_directories: (bool, Vec<PathBuf>),
}

impl From<(&mut MakeModuleAcc, &ObjectName)> for MakeModuleHelper {
    fn from((parent_acc, name): (&mut MakeModuleAcc, &ObjectName)) -> Self {
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

impl From<MakeModuleHelper> for MakeModuleAcc {
    fn from(helper: MakeModuleHelper) -> Self {
        MakeModuleAcc::with_content(
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
    MakeProcessor<'a, 'b, 'c, RMS, FFWD, MakeModuleAcc>
{
    pub fn prepare_dir_exploration<It>(tree: It) -> Vec<It::Item>
    where
        It: Iterator,
        It::Item: NamedObject + TypedObject,
    {
        let mut children_objects: Vec<_> = tree.collect();
        let p = children_objects.iter().position(|x| match x.r#type() {
            ObjectType::File => crate::processing::file_sys::MakeFile::matches(x.name()),
            ObjectType::Dir => false,
        });
        if let Some(p) = p {
            children_objects.swap(0, p); // priority to config file processing
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
            BasicGitObject::Blob(_, n) => crate::processing::file_sys::MakeFile::matches(n),
            _ => false,
        });
        if let Some(p) = p {
            children_objects.swap(0, p); // priority to config file processing
            children_objects.reverse(); // we use it like a stack
        }
    }
    children_objects
}

// # Pom

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub(crate) cpp_handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<
        crate::cpp_processor::CppProc,
    >,
}
impl From<crate::processing::erased::ParametrizedCommitProcessor2Handle<MakeProc>>
    for crate::processing::erased::ParametrizedCommitProcessor2Handle<MakefileProc>
{
    fn from(
        value: crate::processing::erased::ParametrizedCommitProcessor2Handle<MakeProc>,
    ) -> Self {
        crate::processing::erased::ParametrizedCommitProcessor2Handle(
            value.0,
            std::marker::PhantomData,
        )
    }
}
impl From<crate::processing::erased::ParametrizedCommitProcessor2Handle<MakeProc>>
    for crate::processing::erased::ParametrizedCommitProcessor2Handle<crate::cpp_processor::CppProc>
{
    fn from(
        value: crate::processing::erased::ParametrizedCommitProcessor2Handle<MakeProc>,
    ) -> Self {
        crate::processing::erased::ParametrizedCommitProcessor2Handle(
            value.0,
            std::marker::PhantomData,
        )
    }
}
// #[derive(Default)]
struct MakefileProcessorHolder(Option<MakefileProc>);
impl Default for MakefileProcessorHolder {
    fn default() -> Self {
        Self(Some(MakefileProc(None, Default::default())))
    }
}

struct MakefileProc(Option<Parameter>, crate::processing::caches::Makefile);

impl crate::processing::erased::Parametrized for MakefileProcessorHolder {
    type T = Parameter;
    fn register_param(
        &mut self,
        t: Self::T,
    ) -> crate::processing::erased::ParametrizedCommitProcessorHandle {
        let l = self
            .0
            .iter()
            .position(|x| x.0.as_ref() == Some(&t))
            .unwrap_or_else(|| {
                let l = 0; //self.0.len();
                           // self.0.push(MakefileProc(t));
                self.0 = Some(MakefileProc(Some(t), Default::default()));
                l
            });
        use crate::processing::erased::ConfigParametersHandle;
        use crate::processing::erased::ParametrizedCommitProc;
        use crate::processing::erased::ParametrizedCommitProcessorHandle;
        ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    }
}

// TODO should not have to impl this trait
impl crate::processing::erased::CommitProc for MakefileProc {
    fn prepare_processing(
        &self,
        repository: &git2::Repository,
        commit_builder: crate::preprocessed::CommitBuilder,
        param_handle: crate::processing::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc> {
        unimplemented!("required for processing at the root of a project")
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        unimplemented!("required for processing at the root of a project")
    }
}

impl crate::processing::erased::CommitProcExt for MakefileProc {
    type Holder = MakefileProcessorHolder;
}

impl crate::processing::erased::ParametrizedCommitProc2 for MakefileProcessorHolder {
    type Proc = MakefileProc;

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
impl CacheHolding<crate::processing::caches::Makefile> for MakefileProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Makefile {
        &mut self.1
    }
    fn get_caches(&self) -> &crate::processing::caches::Makefile {
        &self.1
    }
}
impl CacheHolding<crate::processing::caches::Makefile> for MakefileProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Makefile {
        &mut self.0.as_mut().unwrap().1
    }
    fn get_caches(&self) -> &crate::processing::caches::Makefile {
        &self.0.as_ref().unwrap().1
    }
}

// # Make
#[derive(Default)]
pub(crate) struct MakeProcessorHolder(Option<MakeProc>);
pub(crate) struct MakeProc {
    parameter: Parameter,
    cache: crate::processing::caches::Make,
    commits: std::collections::HashMap<git2::Oid, crate::Commit>,
}
impl crate::processing::erased::Parametrized for MakeProcessorHolder {
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
                           // self.0.push(MakeProc(t));
                self.0 = Some(MakeProc {
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
struct PreparedMakeCommitProc<'repo> {
    repository: &'repo git2::Repository,
    commit_builder: crate::preprocessed::CommitBuilder,
    pub(crate) handle: ParametrizedCommitProcessorHandle,
}
impl<'repo> crate::processing::erased::PreparedCommitProc for PreparedMakeCommitProc<'repo> {
    fn process(
        self: Box<PreparedMakeCommitProc<'repo>>,
        prepro: &mut RepositoryProcessor,
    ) -> hyperast::store::defaults::NodeIdentifier {
        let dir_path = PathBuf::from("");
        let mut dir_path = dir_path.components().peekable();
        let name = b"";
        // TODO check parameter in self to know it is a recusive module search
        let root_full_node = MakeProcessor::<true, false, MakeModuleAcc>::new(
            self.repository,
            prepro,
            &mut dir_path,
            name,
            self.commit_builder.tree_oid(),
            self.handle,
        )
        .process();
        let h = prepro
            .processing_systems
            .mut_or_default::<MakeProcessorHolder>();
        let handle = self.handle;
        let commit_oid = self.commit_builder.commit_oid();
        let commit = self.commit_builder.finish(root_full_node.0);
        h.with_parameters_mut(handle.1)
            .commits
            .insert(commit_oid, commit);
        root_full_node.0
    }
}

impl crate::processing::erased::CommitProc for MakeProc {
    fn prepare_processing<'repo>(
        &self,
        repository: &'repo git2::Repository,
        commit_builder: crate::preprocessed::CommitBuilder,
        handle: crate::processing::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc + 'repo> {
        Box::new(PreparedMakeCommitProc {
            repository,
            commit_builder,
            handle,
        })
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }
}

impl crate::processing::erased::CommitProcExt for MakeProc {
    type Holder = MakeProcessorHolder;
}

impl crate::processing::erased::ParametrizedCommitProc2 for MakeProcessorHolder {
    type Proc = MakeProc;

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

impl CacheHolding<crate::processing::caches::Make> for MakeProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Make {
        &mut self.cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Make {
        &self.cache
    }
}

impl CacheHolding<crate::processing::caches::Make> for MakeProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Make {
        &mut self.0.as_mut().unwrap().cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Make {
        &self.0.as_ref().unwrap().cache
    }
}
