use crate::{
    cpp::CppAcc,
    git::BasicGitObject,
    make::MakeModuleAcc,
    preprocessed::{IsSkippedAna, RepositoryProcessor},
    processing::{erased::CommitProcExt, CacheHolding, InFiles, ObjectName},
    Processor, StackEle,
};
use git2::{Oid, Repository};
use hyperast::{
    store::nodes::legion::eq_node,
    types::{ETypeStore as _, LabelStore},
};
use hyperast_gen_ts_cpp::{
    legion as cpp_gen,
    types::{CppEnabledTypeStore as _, Type},
};
use std::{iter::Peekable, path::Components, sync::Arc};

pub(crate) fn prepare_dir_exploration(tree: git2::Tree) -> Vec<BasicGitObject> {
    tree.iter()
        .rev()
        .map(TryInto::try_into)
        .filter_map(|x| x.ok())
        .collect()
}

pub type SimpleStores = hyperast::store::SimpleStores<hyperast_gen_ts_cpp::types::TStore>;

pub struct CppProcessor<'repo, 'prepro, 'd, 'c, Acc> {
    repository: &'repo Repository,
    prepro: &'prepro mut RepositoryProcessor,
    stack: Vec<StackEle<Acc>>,
    pub dir_path: &'d mut Peekable<Components<'c>>,
    parameters: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
}

impl<'repo, 'b, 'd, 'c, Acc: From<String>> CppProcessor<'repo, 'b, 'd, 'c, Acc> {
    pub(crate) fn new(
        repository: &'repo Repository,
        prepro: &'b mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
        name: &ObjectName,
        oid: git2::Oid,
        parameters: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree);
        let name = name.try_into().unwrap();
        let stack = vec![StackEle::new(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            parameters,
        }
    }
}

pub static SUB_QUERIES: &[&str] = &[
    r#"(declaration
    type: (primitive_type) (#EQ? "char")
)"#,
    r#"(preproc_if)"#,
    r#"(call_expression
    (field_expression
        (call_expression
            (qualified_identifier
                (namespace_identifier) (#EQ? "base")
                (qualified_identifier
                    (namespace_identifier) (#EQ? "CommandLine")
                    (identifier) (#EQ? "ForCurrentProcess")
                )
            )
            (argument_list)
        )
        "->"
        (field_identifier) (#EQ? "HasSwitch")
    )
)"#,
    r#"(qualified_identifier
    (namespace_identifier) (#EQ? "switches")
    (identifier)
)"#,
];

impl<'repo, 'b, 'd, 'c> Processor<CppAcc> for CppProcessor<'repo, 'b, 'd, 'c, CppAcc> {
    fn pre(&mut self, current_object: BasicGitObject) {
        match current_object {
            BasicGitObject::Tree(oid, name) => {
                self.handle_tree_cached(oid, name);
            }
            BasicGitObject::Blob(oid, name) => {
                if crate::processing::file_sys::Cpp::matches(&name) {
                    self.prepro
                        .help_handle_cpp_file(
                            oid,
                            &mut self.stack.last_mut().unwrap().acc,
                            &name,
                            self.repository,
                            *self.parameters,
                        )
                        .unwrap();
                } else {
                    log::debug!("not cpp source file {:?}", name.try_str());
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: CppAcc) -> Option<(cpp_gen::Local, IsSkippedAna)> {
        let skiped_ana = true;
        let name = acc.primary.name.clone();
        let key = (oid, name.as_bytes().into());
        let full_node = make(acc, self.prepro.main_stores_mut().mut_with_ts());
        self.prepro
            .processing_systems
            .mut_or_default::<CppProcessorHolder>()
            .get_caches_mut()
            .object_map
            .insert(key, (full_node.clone(), skiped_ana));
        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some((full_node, skiped_ana))
        } else {
            let w = &mut self.stack.last_mut().unwrap().acc;
            assert!(
                !w.primary.children_names.contains(&name),
                "{:?} {:?}",
                w.primary.children_names,
                name
            );
            w.push(name, full_node.clone(), skiped_ana);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<StackEle<CppAcc>> {
        &mut self.stack
    }
}

impl<'repo, 'prepro, 'd, 'c> CppProcessor<'repo, 'prepro, 'd, 'c, CppAcc> {
    fn handle_tree_cached(&mut self, oid: Oid, name: ObjectName) {
        if let Some(
            // (already, skiped_ana)
            already,
        ) = self
            .prepro
            .processing_systems
            .mut_or_default::<CppProcessorHolder>()
            .get_caches_mut()
            .object_map
            .get(&(oid, name.clone()))
        {
            // reinit already computed node for post order
            let full_node = already.clone();
            // let skiped_ana = *skiped_ana;
            let w = &mut self.stack.last_mut().unwrap().acc;
            let name = self.prepro.intern_object_name(&name);
            assert!(!w.primary.children_names.contains(&name));
            hyperast::tree_gen::Accumulator::push(w, (name, full_node));
            // w.push(name, full_node, skiped_ana);
        } else {
            log::debug!("tree {:?}", name.try_str());
            let tree = self.repository.find_tree(oid).unwrap();
            let prepared: Vec<BasicGitObject> = prepare_dir_exploration(tree);
            self.stack.push(StackEle::new(
                oid,
                prepared,
                CppAcc::new(name.try_into().unwrap()),
            ));
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub(crate) query: Option<std::sync::Arc<[String]>>,
}
#[derive(Default)]
pub(crate) struct CppProcessorHolder(Option<CppProc>);
pub(crate) struct CppProc {
    parameter: Parameter,
    query: Query,
    cache: crate::processing::caches::Cpp,
    commits: std::collections::HashMap<git2::Oid, crate::Commit>,
}
impl crate::processing::erased::Parametrized for CppProcessorHolder {
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
                           // self.0.push(CppProc(t));
                let query = if let Some(q) = &t.query {
                    Query::new(q.iter().map(|x| x.as_str()))
                } else {
                    let precomputeds = crate::cpp_processor::SUB_QUERIES;
                    Query::new(precomputeds.into_iter().map(|x| x.as_ref()))
                };
                self.0 = Some(CppProc {
                    parameter: t,
                    query,
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

#[derive(Clone)]
pub(crate) struct Query(pub(crate) hyperast_tsquery::Query, Arc<str>);

impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for Query {}

impl Query {
    fn new<'a>(precomputeds: impl Iterator<Item = &'a str>) -> Self {
        static DQ: &str = "(_)";
        let precomputeds = precomputeds.collect::<Vec<_>>();
        let (precomp, _) = hyperast_tsquery::Query::with_precomputed(
            DQ,
            hyperast_gen_ts_cpp::language(),
            precomputeds.as_slice(),
        )
        .unwrap();
        Self(precomp.into(), precomputeds.join("\n").into())
    }
}

impl crate::processing::erased::CommitProc for CppProc {
    fn prepare_processing(
        &self,
        _repository: &git2::Repository,
        _builder: crate::preprocessed::CommitBuilder,
        _handle: crate::processing::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc> {
        unimplemented!("required for processing cpp at the root of a project")
    }

    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }
}

impl crate::processing::erased::CommitProcExt for CppProc {
    type Holder = CppProcessorHolder;
}
impl crate::processing::erased::ParametrizedCommitProc2 for CppProcessorHolder {
    type Proc = CppProc;

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
impl CacheHolding<crate::processing::caches::Cpp> for CppProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Cpp {
        &mut self.cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Cpp {
        &self.cache
    }
}

impl CacheHolding<crate::processing::caches::Cpp> for CppProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Cpp {
        &mut self.0.as_mut().unwrap().cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Cpp {
        &self.0.as_ref().unwrap().cache
    }
}

#[cfg(feature = "cpp")]
impl RepositoryProcessor {
    fn handle_cpp_blob(
        &mut self,
        oid: Oid,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> Result<(cpp_gen::Local, IsSkippedAna), crate::ParseErr> {
        self.processing_systems
            .caching_blob_handler::<crate::processing::file_sys::Cpp>()
            .handle2(oid, repository, &name, parameters, |c, n, t| {
                let line_break = if t.contains(&b'\r') { "\r\n" } else { "\n" }
                    .as_bytes()
                    .to_vec();
                let holder = c.mut_or_default::<CppProcessorHolder>();
                let cpp_proc = holder.0.as_mut().unwrap();
                let md_cache = &mut cpp_proc.cache.md_cache;
                let stores = self
                    .main_stores
                    .mut_with_ts::<hyperast_gen_ts_cpp::types::TStore>();
                let more = hyperast_tsquery::PreparedQuerying::<
                    _,
                    hyperast_gen_ts_cpp::types::TStore,
                    cpp_gen::Acc,
                >::from(&cpp_proc.query.0);
                let mut cpp_tree_gen = cpp_gen::CppTreeGen {
                    line_break,
                    stores,
                    md_cache,
                    more,
                };
                crate::cpp::handle_cpp_file(&mut cpp_tree_gen, n, t)
                    .map(|x| {
                        let local = x.node.local.clone();
                        self.parsing_time += x.parsing_time;
                        self.processing_time += x.processing_time;
                        log::debug!(
                            "parsing, processing, n, f: {} {} {} {}",
                            self.parsing_time.as_secs(),
                            self.processing_time.as_secs(),
                            cpp_proc.cache.md_cache.len(),
                            cpp_proc.cache.object_map.len()
                        );
                        (local, false)
                    })
                    .map_err(|_| crate::ParseErr::IllFormed)
            })
    }

    pub(crate) fn help_handle_cpp_file(
        &mut self,
        oid: Oid,
        parent: &mut CppAcc,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> Result<(), crate::ParseErr> {
        let (full_node, skiped_ana) = self.handle_cpp_blob(oid, name, repository, parameters)?;
        let name = self.intern_object_name(name);
        assert!(!parent.primary.children_names.contains(&name));

        parent.push(name, full_node, skiped_ana);
        Ok(())
    }
    pub(crate) fn help_handle_cpp_file2(
        &mut self,
        oid: Oid,
        parent: &mut MakeModuleAcc,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> Result<(), crate::ParseErr> {
        let (full_node, skiped_ana) = self.handle_cpp_blob(oid, name, repository, parameters)?;
        let name = self.intern_object_name(name);
        // assert!(!parent_acc.children_names.contains(&name));
        // parent_acc.push_pom(name, x);
        assert!(!parent.primary.children_names.contains(&name));

        parent.push_source_file(name, full_node, skiped_ana);
        Ok(())
    }

    pub(crate) fn handle_cpp_directory<'b, 'd: 'b>(
        &mut self,
        repository: &Repository,
        dir_path: &'b mut Peekable<Components<'d>>,
        name: &ObjectName,
        oid: git2::Oid,
        handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> (cpp_gen::Local, IsSkippedAna) {
        CppProcessor::<CppAcc>::new(repository, self, dir_path, name, oid, &handle).process()
    }

    pub(crate) fn help_handle_cpp_folder<'a, 'b, 'c, 'd: 'c>(
        &'a mut self,
        repository: &'b Repository,
        dir_path: &'c mut Peekable<Components<'d>>,
        oid: Oid,
        name: &ObjectName,
        handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<CppProc>,
    ) -> <CppAcc as hyperast::tree_gen::Accumulator>::Node {
        let full_node = self.handle_cpp_directory(repository, dir_path, name, oid, handle);
        let name = self.intern_object_name(name);
        (name, full_node)
    }
}

fn make(acc: CppAcc, stores: &mut SimpleStores) -> cpp_gen::Local {
    use hyperast::hashed::{IndexingHashBuilder, MetaDataHashsBuilder};
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    let kind = Type::Directory;
    let interned_kind = hyperast_gen_ts_cpp::types::TStore::intern(kind);
    let label_id = label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));
    let hashable = primary.metrics.hashs.most_discriminating();
    let eq = eq_node(&Type::Directory, Some(&label_id), &primary.children);
    let insertion = node_store.prepare_insertion(&hashable, eq);

    if let Some(id) = insertion.occupied_id() {
        // NOTE this cituation should not happen often, due to cache based on oids, so there is no point caching md.
        // If git objects are changed but ignored, then it goes through this branch.
        // TODO bench
        // TODO in the oid cache the values could be NodeIdentifiers, then current cache would be used with an indirection.

        let metrics = primary.metrics.map_hashs(|h| h.build());
        let ana = None;
        return cpp_gen::Local {
            compressed_node: id,
            metrics,
            ana,
            role: None,
            precomp_queries: Default::default(),
        };
    }

    let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    let ana = None;

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

    let full_node = cpp_gen::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana,
        role: None,
        precomp_queries: Default::default(),
    };
    full_node
}
