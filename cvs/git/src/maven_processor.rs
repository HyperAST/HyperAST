use crate::processing::erased::{
    CommitProcessorHandle, ParametrizedCommitProcessor2Handle as PCP2Handle,
};
use crate::processing::ParametrizedCommitProcessorHandle;
use crate::StackEle;
use crate::{
    git::{BasicGitObject, NamedObject, ObjectType, TypedObject},
    maven::{MavenModuleAcc, MD},
    preprocessed::RepositoryProcessor,
    processing::{erased::ParametrizedCommitProc2, CacheHolding, InFiles, ObjectName},
    Processor,
};
use git2::{Oid, Repository};
use hyperast::store::nodes::legion::RawHAST;
use hyperast::types::ETypeStore as _;
use hyperast::{
    hashed::MetaDataHashsBuilder,
    store::{defaults::NodeIdentifier, nodes::EntityBuilder},
    tree_gen::Accumulator,
    types::LabelStore,
};
use hyperast_gen_ts_xml::types::{Type, XmlEnabledTypeStore as _};
use std::{
    iter::Peekable,
    marker::PhantomData,
    path::{Components, PathBuf},
};
pub type SimpleStores = hyperast::store::SimpleStores<hyperast_gen_ts_xml::types::TStore>;

/// RMS: Resursive Module Search
/// FFWD: Fast ForWarD to java directories without looking at maven stuff
pub struct MavenProcessor<'a, 'b, 'c, const RMS: bool, const FFWD: bool, Acc> {
    prepro: &'b mut RepositoryProcessor,
    repository: &'a Repository,
    stack: Vec<StackEle<Acc>>,
    dir_path: &'c mut Peekable<Components<'c>>,
    handle: ParametrizedCommitProcessorHandle,
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool>
    MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
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
        let acc = MavenModuleAcc::new(name);
        let prep_scripting = prep_scripting(prepro, handle.1);
        let acc = acc.init_scripting(
            prep_scripting
                .map(|x| {
                    hyperast::scripting::Prepro::<
                        RawHAST<hyperast_gen_ts_java::types::TStore>,
                        &hyperast_gen_ts_java::legion_with_refs::Acc,
                    >::from(x.clone())
                })
                .as_ref(),
        );
        let stack = vec![StackEle::new(oid, prepared, acc)];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            handle,
        }
    }
}

impl<'a, 'b, 'c, const RMS: bool, const FFWD: bool> Processor<MavenModuleAcc>
    for MavenProcessor<'a, 'b, 'c, RMS, FFWD, MavenModuleAcc>
{
    fn pre(&mut self, current_dir: BasicGitObject) {
        match current_dir {
            BasicGitObject::Tree(oid, name) => {
                self.handle_tree_cached(name, oid);
            }
            BasicGitObject::Blob(oid, name)
                if !FFWD
                    && !self.dir_path.peek().is_some()
                    && crate::processing::file_sys::Pom::matches(&name) =>
            {
                let parent_acc = &mut self.stack.last_mut().unwrap().acc;
                // TODO find a better conversion, ie. first safe conv to MavenProc handle then into()
                let parameters = PCP2Handle(self.handle.1, PhantomData);
                if let Err(err) =
                    self.prepro
                        .handle_pom(oid, parent_acc, name, &self.repository, parameters)
                {
                    log::debug!("{:?}", err);
                }
            }
            _ => {}
        }
    }
    fn post(&mut self, oid: Oid, acc: MavenModuleAcc) -> Option<(NodeIdentifier, MD)> {
        let name = acc.primary.name.clone();
        let full_node = Self::make(acc, self.prepro.main_stores_mut().mut_with_ts());
        self.prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>()
            .with_parameters_mut(self.handle.1)
            .get_caches_mut()
            .object_map
            .insert(oid, full_node.clone());
        let name = self.prepro.intern_label(&name);
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
            let id = full_node.0;
            if full_node
                .1
                .status
                .contains(crate::maven::SemFlags::IsMavenModule)
            {
                w.push_submodule(name, full_node);
            } else {
                w.push((name, full_node));
            }

            if let Some(acc) = &mut w.scripting_acc {
                // SAFETY: this side should be fine, issue when unerasing
                let store = unsafe { self.prepro.main_stores.erase_ts_unchecked() };
                let child: hyperast::scripting::lua_scripting::SubtreeHandle<
                    hyperast_gen_ts_xml::types::TType,
                > = id.into();
                acc.acc(store, Type::Directory, child).unwrap();
            }
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<StackEle<MavenModuleAcc>> {
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
                .eq(std::ffi::OsStr::as_encoded_bytes(s.as_os_str()))
            {
                self.dir_path.next();
                self.stack.last_mut().expect("never empty").cs.clear();
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared = prepare_dir_exploration(tree, &mut self.dir_path);
                let acc = MavenModuleAcc::new(name.try_into().unwrap());
                let prep_scripting = prep_scripting(&self.prepro, self.handle.1);
                let acc = acc.init_scripting(
                    prep_scripting
                        .map(|x| {
                            hyperast::scripting::Prepro::<
                                RawHAST<hyperast_gen_ts_java::types::TStore>,
                                &hyperast_gen_ts_java::legion_with_refs::Acc,
                            >::from(x.clone())
                        })
                        .as_ref(),
                );
                self.stack.push(StackEle::new(oid, prepared, acc));
                return;
            } else {
                return;
            }
        }
        let maven_proc = self
            .prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>()
            .with_parameters_mut(self.handle.1);
        let java_handle = maven_proc.parameter.java_handle;
        if let Some(already) = maven_proc.get_caches_mut().object_map.get(&oid) {
            // reinit already computed node for post order
            let full_node = already.clone();
            let w = &mut self.stack.last_mut().unwrap().acc;
            let name = self.prepro.intern_object_name(&name);
            assert!(!w.primary.children_names.contains(&name));
            let id = full_node.0;
            if full_node
                .1
                .status
                .contains(crate::maven::SemFlags::IsMavenModule)
            {
                w.push_submodule(name, full_node);
            } else {
                w.push((name, full_node));
            }
            if let Some(acc) = &mut w.scripting_acc {
                // SAFETY: this side should be fine, issue when unerasing
                let store = unsafe { self.prepro.main_stores.erase_ts_unchecked() };
                acc.acc::<_, hyperast_gen_ts_xml::types::TType, _>(
                    store,
                    Type::Directory,
                    id.into(),
                )
                .unwrap();
            }
            return;
        }

        log::debug!("maven tree {:?}", name.try_str());
        let parent_acc = &mut self.stack.last_mut().unwrap().acc;
        if FFWD {
            let (name, (full_node, _)) = self.prepro.help_handle_java_folder(
                &self.repository,
                &mut self.dir_path,
                oid,
                &name,
                java_handle,
            );
            let id = full_node.compressed_node;
            assert!(!parent_acc.primary.children_names.contains(&name));
            parent_acc.push_source_directory(name, full_node);
            if let Some(acc) = &mut parent_acc.scripting_acc {
                // SAFETY: this side should be fine, issue when unerasing
                let store = unsafe { self.prepro.main_stores.erase_ts_unchecked() };
                acc.acc::<_, hyperast_gen_ts_java::types::TType, _>(
                    store,
                    Type::Directory,
                    id.into(),
                )
                .unwrap();
            }
            return;
        }
        let helper = MavenModuleHelper::from((parent_acc, &name));
        if helper.source_directories.0 || helper.test_source_directories.0 {
            // handle as source dir
            let (name, (full_node, _)) = self.prepro.help_handle_java_folder(
                &self.repository,
                self.dir_path,
                oid,
                &name,
                java_handle,
            );
            let id = full_node.compressed_node;
            let parent_acc = &mut self.stack.last_mut().unwrap().acc;
            assert!(!parent_acc.primary.children_names.contains(&name));
            if helper.source_directories.0 {
                parent_acc.push_source_directory(name, full_node);
            } else {
                // test_source_folders.0
                parent_acc.push_test_source_directory(name, full_node);
            }

            if let Some(acc) = &mut parent_acc.scripting_acc {
                // SAFETY: this side should be fine, issue when unerasing
                let store = unsafe { self.prepro.main_stores.erase_ts_unchecked() };
                acc.acc::<_, hyperast_gen_ts_java::types::TType, _>(
                    store,
                    Type::Directory,
                    id.into(),
                )
                .unwrap();
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
            let prep_scripting = prep_scripting(&self.prepro, self.handle.1);
            if helper.submodules.0 {
                // handle as maven module
                let acc = helper.into_acc().init_scripting(
                    prep_scripting
                        .map(|x| hyperast::scripting::Prepro::from(x.clone()))
                        .as_ref(),
                );
                self.stack.push(StackEle::new(oid, prepared, acc));
            } else {
                // search further inside
                let acc = helper.into_acc().init_scripting(
                    prep_scripting
                        .map(|x| hyperast::scripting::Prepro::from(x.clone()))
                        .as_ref(),
                );
                self.stack.push(StackEle::new(oid, prepared, acc));
            };
        } else if RMS && !(helper.source_directories.0 || helper.test_source_directories.0) {
            let tree = self.repository.find_tree(oid).unwrap();
            // anyway try to find maven modules, but maybe can do better
            let prepared = prepare_dir_exploration(tree, &mut self.dir_path);

            let prep_scripting = prep_scripting(&self.prepro, self.handle.1);
            let acc = helper.into_acc().init_scripting(
                prep_scripting
                    .map(|x| hyperast::scripting::Prepro::from(x.clone()))
                    .as_ref(),
            );
            self.stack.push(StackEle::new(oid, prepared, acc));
        }
    }
}

impl MavenModuleAcc {
    fn init_scripting(
        mut self,
        prep_scripting: Option<
            &hyperast::scripting::Prepro<
                RawHAST<hyperast_gen_ts_java::types::TStore>,
                &hyperast_gen_ts_java::legion_with_refs::Acc,
            >,
        >,
    ) -> Self {
        if let Some(more) = prep_scripting {
            log::info!("prep_scripting");
            use hyperast::tree_gen::Prepro;
            match more.preprocessing(hyperast_gen_ts_java::types::Type::Directory) {
                Ok(acc) => self.scripting_acc = Some(acc),
                Err(err) => {
                    log::error!("error when handling maven modules {}", err);
                }
            }
        } else {
            log::trace!("no prep_scripting");
        };
        self
    }
}

// TODO generalize and factor similar preps
// and use the type in ParametrizedCommitProcessor2Handle to get the Holder
fn prep_scripting(
    prepro: &RepositoryProcessor,
    handle: crate::processing::erased::ConfigParametersHandle,
) -> Option<&std::sync::Arc<str>> {
    prepro
        .processing_systems
        // it is fine but could do better and kind of use MavenHolder
        .get::<crate::java_processor::JavaProcessorHolder>()
        .as_ref()?
        .with_parameters(handle)
        .parameter
        .prepro
        .as_ref()
}

pub(crate) fn make(mut acc: MavenModuleAcc, stores: &mut SimpleStores) -> (NodeIdentifier, MD) {
    use hyperast::hashed::IndexingHashBuilder;
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    use hyperast::store::nodes::legion::eq_node;
    let kind = Type::MavenDirectory;
    let interned_kind = hyperast_gen_ts_xml::types::TStore::intern(kind);
    let label_id = label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));

    let hashable = primary.metrics.hashs.most_discriminating();

    let eq = eq_node(&interned_kind, Some(&label_id), &primary.children);

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
    let insertion = node_store.prepare_insertion(&hashable, eq);

    if let Some(id) = insertion.occupied_id() {
        let metrics = primary.metrics.map_hashs(|h| h.build());
        let status = acc.status;
        let md = MD {
            metrics,
            ana,
            status,
        };
        return (id, md);
    }
    use hyperast::store::nodes::legion::NodeStore;

    log::info!("make mm {} {}", &primary.name, primary.children.len());
    assert_eq!(primary.children_names.len(), primary.children.len());
    let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();
    let children_is_empty = primary.children.is_empty();
    if !acc.status.is_empty() {
        dyn_builder.add(acc.status);
    }
    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    if let Some(acc) = acc.scripting_acc {
        let subtr = hyperast::scripting::lua_scripting::Subtr(kind, &dyn_builder);
        let ss = acc.finish(&subtr).unwrap();
        log::error!("mm {:?}", ss.0);
        use hyperast::store::nodes::EntityBuilder;
        dyn_builder.add(ss);
    };

    let vacant = insertion.vacant();
    let node_id = NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

    let status = acc.status;

    let md = MD {
        metrics,
        ana,
        status,
    };
    (node_id.clone(), md)
}

use hyperast_gen_ts_xml::legion::XmlTreeGen;
impl RepositoryProcessor {
    fn handle_pom(
        &mut self,
        oid: Oid,
        parent_acc: &mut MavenModuleAcc,
        name: ObjectName,
        repository: &Repository,
        parameters: PCP2Handle<PomProc>,
    ) -> Result<(), crate::ParseErr> {
        let x = self
            .processing_systems
            .caching_blob_handler::<crate::processing::file_sys::Pom>()
            .handle(oid, repository, &name, parameters, |c, n, t| {
                // let caches = c.mut_or_default::<PomProcessorHolder>().get_caches_mut();
                crate::maven::handle_pom_file(
                    &mut XmlTreeGen {
                        line_break: "\n".as_bytes().to_vec(),
                        stores: self.main_stores.mut_with_ts(),
                    },
                    n,
                    t,
                )
            })?;
        let name = self.intern_object_name(&name);
        assert!(!parent_acc.primary.children_names.contains(&name));
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

    pub(crate) fn xml_generator(&mut self) -> XmlTreeGen<hyperast_gen_ts_xml::types::TStore> {
        XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: self.main_stores.mut_with_ts(),
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

impl MavenModuleHelper {
    pub fn into_acc(self) -> MavenModuleAcc {
        MavenModuleAcc::with_content(
            self.name,
            self.submodules.1,
            self.source_directories.1,
            self.test_source_directories.1,
        )
    }
    pub fn into_acc_with_scripting(self, prepro_acc: hyperast::scripting::Acc) -> MavenModuleAcc {
        let mut r = MavenModuleAcc::with_content(
            self.name,
            self.submodules.1,
            self.source_directories.1,
            self.test_source_directories.1,
        );
        r.scripting_acc = Some(prepro_acc);
        r
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
            BasicGitObject::Blob(_, n) => crate::processing::file_sys::Pom::matches(&n),
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
pub struct Parameter {
    // pub(crate) query: Option<std::sync::Arc<[String]>>,
    // pub(crate) prepo: Option<hyperast_scripting::Prepro>,
    pub java_handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<
        crate::java_processor::JavaProc,
    >,
}

impl From<PCP2Handle<MavenProc>> for PCP2Handle<PomProc> {
    fn from(value: PCP2Handle<MavenProc>) -> Self {
        PCP2Handle(value.0, PhantomData)
    }
}
struct PomProcessorHolder(Option<PomProc>);
impl Default for PomProcessorHolder {
    fn default() -> Self {
        Self(Some(PomProc {
            parameter: None,
            cache: Default::default(),
        }))
    }
}
struct PomProc {
    parameter: Option<Parameter>,
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
            .position(|x| x.parameter.as_ref() == Some(&t))
            .unwrap_or_else(|| {
                let l = 0; //self.0.len();
                           // self.0.push(PomProc(t));
                self.0 = Some(PomProc {
                    parameter: Some(t),
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
    fn prepare_processing(
        &self,
        _repository: &git2::Repository,
        _commit_builder: crate::preprocessed::CommitBuilder,
        _param_handle: crate::processing::erased::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc> {
        unimplemented!("required for processing at the root of a project")
    }

    fn get_commit(&self, _commit_oid: git2::Oid) -> Option<&crate::Commit> {
        unimplemented!("required for processing at the root of a project")
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
pub struct MavenProcessorHolder(Vec<MavenProc>);
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
                let l = self.0.len();
                self.0.push(MavenProc {
                    parameter: t,
                    cache: Default::default(),
                    commits: Default::default(),
                });
                l
            });

        use crate::processing::erased::ConfigParametersHandle;
        use crate::processing::erased::ParametrizedCommitProc;
        ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    }
}

struct PreparedMavenCommitProc<'repo> {
    repository: &'repo git2::Repository,
    commit_builder: crate::preprocessed::CommitBuilder,
    pub(crate) handle: ParametrizedCommitProcessorHandle,
}

impl<'repo> crate::processing::erased::PreparedCommitProc for PreparedMavenCommitProc<'repo> {
    fn process(
        self: Box<PreparedMavenCommitProc<'repo>>,
        prepro: &mut RepositoryProcessor,
    ) -> hyperast::store::defaults::NodeIdentifier {
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
            self.handle,
        )
        .process();
        let h = prepro
            .processing_systems
            .mut_or_default::<MavenProcessorHolder>();
        let handle = self.handle;
        let commit_oid = self.commit_builder.commit_oid();
        let commit = self.commit_builder.finish(root_full_node.0);
        h.with_parameters_mut(handle.1)
            .commits
            .insert(commit_oid, commit);
        root_full_node.0
    }
}

impl crate::processing::erased::CommitProc for MavenProc {
    fn prepare_processing<'repo>(
        &self,
        repository: &'repo git2::Repository,
        commit_builder: crate::preprocessed::CommitBuilder,
        handle: crate::processing::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc + 'repo> {
        Box::new(PreparedMavenCommitProc {
            repository,
            commit_builder,
            handle,
        })
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }

    fn get_lang_handle(&self, lang: &str) -> Option<ParametrizedCommitProcessorHandle> {
        if lang.eq_ignore_ascii_case("java") {
            Some(ParametrizedCommitProcessorHandle(
                CommitProcessorHandle(std::any::TypeId::of::<
                    crate::java_processor::JavaProcessorHolder,
                >()),
                self.parameter.java_handle.0,
            ))
        } else if lang.eq_ignore_ascii_case("cpp") {
            if cfg!(debug_assertions) {
                unimplemented!()
            }
            None
        } else {
            None
        }
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
        &mut self.0[parameters.0]
    }

    fn with_parameters(
        &self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &Self::Proc {
        &self.0[parameters.0]
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
        &mut self.0[0].cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Maven {
        &self.0[0].cache
    }
}
