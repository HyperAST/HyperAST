use crate::{
    cpp::CppAcc,
    git::BasicGitObject,
    make::MakeModuleAcc,
    preprocessed::{IsSkippedAna, RepositoryProcessor},
    processing::{erased::CommitProcExt, CacheHolding, InFiles, ObjectName},
    Processor, SimpleStores,
};
use git2::{Oid, Repository};
use hyper_ast::{store::nodes::legion::eq_node, types::LabelStore};
use hyper_ast_gen_ts_cpp::{legion as cpp_gen, types::Type};
use std::{iter::Peekable, path::Components};

pub(crate) fn prepare_dir_exploration(tree: git2::Tree) -> Vec<BasicGitObject> {
    tree.iter()
        .rev()
        .map(TryInto::try_into)
        .filter_map(|x| x.ok())
        .collect()
}

pub struct CppProcessor<'repo, 'prepro, 'd, 'c, Acc> {
    repository: &'repo Repository,
    prepro: &'prepro mut RepositoryProcessor,
    stack: Vec<(Oid, Vec<BasicGitObject>, Acc)>,
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
        let stack = vec![(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            parameters,
        }
    }
}

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
                            &mut self.stack.last_mut().unwrap().2,
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
        let full_node = make(acc, self.prepro.main_stores_mut());
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
            let w = &mut self.stack.last_mut().unwrap().2;
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

    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, CppAcc)> {
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
            let w = &mut self.stack.last_mut().unwrap().2;
            let name = self.prepro.intern_object_name(&name);
            assert!(!w.primary.children_names.contains(&name));
            hyper_ast::tree_gen::Accumulator::push(w, (name, full_node));
            // w.push(name, full_node, skiped_ana);
        } else {
            log::info!("tree {:?}", name.try_str());
            let tree = self.repository.find_tree(oid).unwrap();
            let prepared: Vec<BasicGitObject> = prepare_dir_exploration(tree);
            self.stack
                .push((oid, prepared, CppAcc::new(name.try_into().unwrap())));
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter;
#[derive(Default)]
pub(crate) struct CppProcessorHolder(Option<CppProc>);
pub(crate) struct CppProc {
    parameter: Parameter,
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
                self.0 = Some(CppProc {
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
impl crate::processing::erased::CommitProc for CppProc {
    fn prepare_processing(
        &self,
        _repository: &git2::Repository,
        _builder: crate::preprocessed::CommitBuilder,
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
                let line_break = if t.contains(&b'\r') {
                    "\r\n".as_bytes().to_vec()
                } else {
                    "\n".as_bytes().to_vec()
                };
                crate::cpp::handle_cpp_file(
                    &mut cpp_gen::CppTreeGen {
                        line_break,
                        stores: self.main_stores.mut_with_ts(),
                        md_cache: &mut c
                            .mut_or_default::<CppProcessorHolder>()
                            .get_caches_mut()
                            .md_cache, //cpp_md_cache,
                    },
                    n,
                    t,
                )
                .map_err(|_| crate::ParseErr::IllFormed)
                .map(|x| (x.local.clone(), false))
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
    ) -> (cpp_gen::Local, IsSkippedAna) {
        let h = self
            .processing_systems
            .mut_or_default::<CppProcessorHolder>();

        let handle = CppProc::register_param(h, Parameter);
        CppProcessor::<CppAcc>::new(repository, self, dir_path, name, oid, &handle).process()
    }

    pub(crate) fn help_handle_cpp_folder<'a, 'b, 'c, 'd: 'c>(
        &'a mut self,
        repository: &'b Repository,
        dir_path: &'c mut Peekable<Components<'d>>,
        oid: Oid,
        name: &ObjectName,
    ) -> <CppAcc as hyper_ast::tree_gen::Accumulator>::Node {
        let full_node = self.handle_cpp_directory(repository, dir_path, name, oid);
        let name = self.intern_object_name(name);
        (name, full_node)
    }
}

fn make(acc: CppAcc, stores: &mut SimpleStores) -> cpp_gen::Local {
    use hyper_ast::hashed::{IndexingHashBuilder, MetaDataHashsBuilder};
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    let interned_kind = Type::Directory;
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
        };
    }

    let mut dyn_builder = hyper_ast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    let ana = None;

    let children_is_empty = primary.children.is_empty();

    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    let vacant = insertion.vacant();
    let node_id = hyper_ast::store::nodes::legion::NodeStore::insert_built_after_prepare(
        vacant,
        dyn_builder.build(),
    );

    let full_node = cpp_gen::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana,
    };
    full_node
}
