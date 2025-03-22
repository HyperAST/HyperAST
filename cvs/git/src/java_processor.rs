use std::ops::Deref;
use std::sync::Arc;
use std::{iter::Peekable, path::Components};

use git2::{Oid, Repository};
use hyperast::hashed::{IndexingHashBuilder, MetaDataHashsBuilder};
use hyperast::store::nodes::legion::RawHAST;
use hyperast::tree_gen::add_md_precomp_queries;
use hyperast_gen_ts_java::legion_with_refs::{self, Acc};
use hyperast_gen_ts_java::types::{TStore, Type};

use crate::processing::erased::ParametrizedCommitProc2;
use crate::StackEle;
use crate::{
    git::BasicGitObject,
    java::JavaAcc,
    preprocessed::{IsSkippedAna, RepositoryProcessor},
    processing::{CacheHolding, InFiles, ObjectName},
    Processor,
};

pub(crate) fn prepare_dir_exploration(tree: git2::Tree) -> Vec<BasicGitObject> {
    tree.iter()
        .rev()
        .map(TryInto::try_into)
        .filter_map(|x| x.ok())
        .collect()
}

pub type SimpleStores = hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore>;

pub struct JavaProcessor<'repo, 'prepro, 'd, 'c, Acc> {
    repository: &'repo Repository,
    prepro: &'prepro mut RepositoryProcessor,
    stack: Vec<StackEle<Acc>>,
    pub dir_path: &'d mut Peekable<Components<'c>>,
    handle: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
}

impl<'repo, 'b, 'd, 'c> JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
    pub(crate) fn new(
        repository: &'repo Repository,
        prepro: &'b mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
        name: &ObjectName,
        oid: git2::Oid,
        handle: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree);
        let name = name.try_into().unwrap();
        let prep_scripting = prep_scripting(prepro, handle.0);
        use hyperast::tree_gen::Prepro;
        let scripting_acc = prep_scripting.map(|x| {
            hyperast::scripting::Prepro::<SimpleStores, &Acc>::from(x.clone())
                .preprocessing(Type::Directory)
                .unwrap()
        });
        let acc = JavaAcc::new(name, scripting_acc);
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

impl<'repo, 'b, 'd, 'c> Processor<JavaAcc> for JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
    fn pre(&mut self, current_object: BasicGitObject) {
        match current_object {
            BasicGitObject::Tree(oid, name) => {
                if let Some(
                    // (already, skiped_ana)
                    already,
                ) = self
                    .prepro
                    .processing_systems
                    .mut_or_default::<JavaProcessorHolder>()
                    .with_parameters(self.handle.0) //.with_parameters(self.parameters.0)
                    .cache
                    .object_map
                    .get(&(oid, name.clone()))
                {
                    // reinit already computed node for post order
                    let full_node = already.clone();
                    // let skiped_ana = *skiped_ana;
                    let id = full_node.0.compressed_node;
                    let w = &mut self.stack.last_mut().unwrap().acc;
                    let name = self.prepro.intern_object_name(&name);
                    assert!(!w.primary.children_names.contains(&name));
                    hyperast::tree_gen::Accumulator::push(w, (name, full_node));
                    // w.push(name, full_node, skiped_ana);
                    if let Some(acc) = &mut w.scripting_acc {
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
                log::info!("tree {:?}", name.try_str());
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared: Vec<BasicGitObject> = prepare_dir_exploration(tree);

                let prepro_acc = if let Some(more) = prep_scripting(&self.prepro, self.handle.0) {
                    use hyperast::tree_gen::Prepro;
                    Some(
                        hyperast::scripting::Prepro::<RawHAST<TStore>, &Acc>::from(more.clone())
                            .preprocessing(Type::Directory)
                            .unwrap(),
                    )
                } else {
                    None
                };
                let acc = JavaAcc::new(name.try_into().unwrap(), prepro_acc);
                self.stack.push(StackEle::new(oid, prepared, acc));
            }
            BasicGitObject::Blob(oid, name) => {
                if crate::processing::file_sys::Java::matches(&name) {
                    self.prepro
                        .help_handle_java_file(
                            oid,
                            &mut self.stack.last_mut().unwrap().acc,
                            &name,
                            self.repository,
                            *self.handle,
                        )
                        .unwrap();
                } else {
                    log::debug!("not java source file {:?}", name.try_str());
                }
            }
        }
    }

    fn post(&mut self, oid: Oid, acc: JavaAcc) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
        let skiped_ana = acc.skiped_ana;
        let name = &acc.primary.name;
        let key = (oid, name.as_bytes().into());
        let name = self.prepro.get_or_insert_label(name);
        let full_node = make(acc, self.prepro.main_stores_mut().mut_with_ts());
        self.prepro
            .processing_systems
            .mut_or_default::<JavaProcessorHolder>()
            .with_parameters_mut(self.handle.0) //.with_parameters(self.parameters.0)
            .cache
            .object_map
            .insert(key, (full_node.clone(), skiped_ana));
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
            let id = full_node.compressed_node;
            w.push(name, full_node.clone(), skiped_ana);

            if let Some(acc) = &mut w.scripting_acc {
                // SAFETY: this side should be fine, issue when unerasing
                let store = unsafe { self.prepro.main_stores.erase_ts_unchecked() };
                acc.acc::<_, hyperast_gen_ts_java::types::TType, _>(
                    store,
                    Type::Directory,
                    id.into(),
                )
                .unwrap();
            }
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<StackEle<JavaAcc>> {
        &mut self.stack
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
        .get::<JavaProcessorHolder>()
        .as_ref()?
        .with_parameters(handle)
        .parameter
        .prepro
        .as_ref()
}

fn make(acc: JavaAcc, stores: &mut SimpleStores) -> hyperast_gen_ts_java::legion_with_refs::Local {
    use hyperast::{
        cyclomatic::Mcc,
        store::nodes::legion::{eq_node, NodeStore},
        types::LabelStore,
    };
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    let kind = Type::Directory;
    use hyperast::types::ETypeStore;
    let interned_kind = hyperast_gen_ts_java::types::TStore::intern(kind);
    let label_id = label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));

    let hashable = primary.metrics.hashs.most_discriminating();

    let eq = eq_node(&interned_kind, Some(&label_id), &primary.children);

    let insertion = node_store.prepare_insertion(&hashable, eq);

    let compute_ana = || {
        #[cfg(feature = "impact")]
        {
            let ana = acc.ana;
            let ana = if acc.skiped_ana {
                log::info!(
                    "show ana with at least {} refs",
                    ana.lower_estimate_refs_count()
                );
                None
            } else {
                log::info!(
                    "ref count lower estimate in dir {}",
                    ana.lower_estimate_refs_count()
                );
                log::debug!("refs in directory");
                for x in ana.display_refs(label_store) {
                    log::debug!("    {}", x);
                }
                log::debug!("decls in directory");
                for x in ana.display_decls(label_store) {
                    log::debug!("    {}", x);
                }
                let c = ana.estimated_refs_count();
                if c < crate::MAX_REFS {
                    Some(ana.resolve())
                } else {
                    Some(ana)
                }
            };
            // log::info!(
            //     "ref count in dir after resolver {}",
            //     ana.lower_estimate_refs_count()
            // );
            // log::debug!("refs in directory after resolve: ");
            // for x in ana.display_refs(label_store) {
            //     log::debug!("    {}", x);
            // }
            ana
        }
        #[cfg(not(feature = "impact"))]
        {
            None
        }
    };

    // Guard to avoid computing metadata for an already present subtree
    if let Some(id) = insertion.occupied_id() {
        // TODO add (debug) assertions to detect non-local metadata
        // TODO use the cache ?
        // this branch should be really cold
        let ana = compute_ana();
        let metrics = primary
            .metrics
            .map_hashs(|h| MetaDataHashsBuilder::build(h));
        return legion_with_refs::Local {
            compressed_node: id,
            metrics,
            ana,
            mcc: Mcc::new(&Type::Directory),
            role: None,
            precomp_queries: Default::default(),
        };
    }

    let ana = compute_ana();

    let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
    let children_is_empty = primary.children.is_empty();
    if acc.skiped_ana {
        use hyperast::store::nodes::EntityBuilder;
        dyn_builder.add(hyperast::filter::BloomSize::None);
    } else {
        #[cfg(feature = "impact")]
        hyperast_gen_ts_java::legion_with_refs::add_md_ref_ana(
            &mut dyn_builder,
            children_is_empty,
            ana.as_ref(),
        );
    }
    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    if let Some(acc) = acc.scripting_acc {
        let subtr = hyperast::scripting::lua_scripting::Subtr(kind, &dyn_builder);
        let ss = acc.finish(&subtr).unwrap();
        log::error!("dir {:?}", ss.0);
        use hyperast::store::nodes::EntityBuilder;
        dyn_builder.add(ss);
    };

    let vacant = insertion.vacant();
    let node_id = NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

    let full_node = legion_with_refs::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana,
        mcc: Mcc::new(&kind),
        role: None,
        precomp_queries: acc.precomp_queries,
    };
    full_node
}

use hyperast_gen_ts_java::legion_with_refs as java_tree_gen;

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub query: Option<hyperast_tsquery::ZeroSepArrayStr>,
    pub tsg: Option<std::sync::Arc<str>>,
    pub prepro: Option<std::sync::Arc<str>>,
}

#[doc(hidden)]
pub static PREPRO: &str = r#"
local size = 1 -- init

function acc(c)
    size += c.size
end

function finish()
    return {size = size}
end
"#;

impl Parameter {
    pub fn faster() -> Self {
        let query = None;
        let tsg = None;
        let prepro = None;
        Self { query, tsg, prepro }
    }
    pub fn fast() -> Self {
        let query = Some(crate::java_processor::SUB_QUERIES.into());
        let tsg = None;
        let prepro = None;
        Self { query, tsg, prepro }
    }
    pub fn stable() -> Self {
        let query = Some(crate::java_processor::SUB_QUERIES.into());
        let tsg = None;
        let prepro = Some(crate::java_processor::PREPRO.into());
        Self { query, tsg, prepro }
    }

    pub fn nightly() -> Self {
        let query = Some(crate::java_processor::SUB_QUERIES.into());
        let tsg = Some(crate::java_processor::TSG.into());
        let prepro = Some(crate::java_processor::PREPRO.into());
        Self { query, tsg, prepro }
    }
}

#[derive(Default)]
pub struct JavaProcessorHolder(Vec<JavaProc>);
pub struct JavaProc {
    pub(crate) parameter: Parameter,
    pub height_counts: Vec<u32>,
    query: Option<Query>,
    #[cfg(feature = "tsg")]
    tsg: Option<(ErazedTSG, ErazedFcts)>,
    cache: crate::processing::caches::Java,
    commits: std::collections::HashMap<git2::Oid, crate::Commit>,
}

type ErazedFcts = Arc<dyn std::any::Any + Send + Sync>;
type ErazedTSG = Box<dyn std::any::Any + Send + Sync>;

impl crate::processing::erased::Parametrized for JavaProcessorHolder {
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
                assert!(l <= 1);
                let query = if let Some(q) = &t.query {
                    use hyperast_tsquery::ArrayStr;
                    Some(Query::new(q.iter()))
                } else {
                    None
                };

                #[cfg(feature = "tsg")]
                let tsg = if let Some(q) = &t.tsg {
                    let tsg = q.deref();
                    type M<'hast, TS, Acc> = hyperast_tsquery::QueryMatcher<TS, Acc>;
                    type ExtQ<'hast, TS, Acc> = hyperast_tsquery::ExtendingStringQuery<
                        M<'hast, TS, Acc>,
                        tree_sitter::Language,
                    >;

                    let source: &str = tsg;
                    let language = hyperast_gen_ts_java::language();

                    let mut file = tree_sitter_graph::ast::File::<M<&SimpleStores, &Acc>>::new(
                        language.clone(),
                    );

                    // let mty: &[_] = &[];
                    // let query_source = ExtQ::new(language.clone(), Box::new(mty), source.len());
                    let query_source = if let Some(p) = &t.query {
                        ExtQ::new(language.clone(), Box::new(p.clone()), source.len())
                    } else {
                        let x: &[&str] = &[];
                        ExtQ::new(language.clone(), Box::new(x), source.len())
                    };
                    tree_sitter_graph::parser::Parser::<ExtQ<_, _>>::with_ext(query_source, source)
                        .parse_into_file(&mut file)
                        .unwrap();
                    use tree_sitter_graph::GenQuery;

                    M::check(&mut file).unwrap();

                    let mut functions = tree_sitter_graph::functions::Functions::<
                        tree_sitter_graph::graph::Graph<
                            hyperast_tsquery::stepped_query_imm::Node<
                                hyperast::store::SimpleStores<
                                    TStore,
                                    &hyperast::store::nodes::legion::NodeStoreInner,
                                    &hyperast::store::labels::LabelStore,
                                >,
                                &Acc,
                            >,
                        >,
                    >::essentials();
                    // TODO port those path functions to the generified variant in my fork
                    // hyperast_tsquery::add_path_functions(&mut functions);
                    let functions = functions.as_any();

                    Some((file.as_any(), functions))
                } else {
                    // crate::java_processor::TSG
                    None
                };
                let r = JavaProc {
                    parameter: t,
                    height_counts: vec![],
                    query,
                    #[cfg(feature = "tsg")]
                    tsg,
                    cache: Default::default(),
                    commits: Default::default(),
                };
                self.0.push(r);

                l
            });
        use crate::processing::erased::{
            ConfigParametersHandle, ParametrizedCommitProc, ParametrizedCommitProcessorHandle,
        };
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

// impl Default for Query {
//     fn default() -> Self {
//         let precomputeds = crate::java_processor::SUB_QUERIES;
//         Query::new(precomputeds.into_iter().map(|x| x.as_ref()))
//     }
// }

impl Query {
    fn new<'a>(precomputeds: impl Iterator<Item = &'a str>) -> Self {
        static DQ: &str = "(_)";
        let precomputeds = precomputeds.collect::<Vec<_>>();
        let (precomp, _) = hyperast_tsquery::Query::with_precomputed(
            DQ,
            hyperast_gen_ts_java::language(),
            precomputeds.as_slice(),
        )
        .unwrap();
        Self(precomp.into(), precomputeds.join("\n").into())
    }
}

impl crate::processing::erased::CommitProc for JavaProc {
    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }

    fn get_precomp_query(&self) -> Option<hyperast_tsquery::ZeroSepArrayStr> {
        dbg!(&self.parameter.query);
        // if self.parameter.query.is_none() {
        //     let s = crate::java_processor::SUB_QUERIES;
        //     let s: Vec<_> = s.iter().map(|x| x.to_string()).collect();
        //     return Some(s.into());
        // }
        self.parameter.query.clone()
    }

    fn prepare_processing<'repo>(
        &self,
        _repository: &'repo git2::Repository,
        _commit_builder: crate::preprocessed::CommitBuilder,
        _handle: crate::processing::ParametrizedCommitProcessorHandle,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc + 'repo> {
        unimplemented!("required for processing java at the root of project")
    }
}

impl crate::processing::erased::CommitProcExt for JavaProc {
    type Holder = JavaProcessorHolder;
}
impl crate::processing::erased::ParametrizedCommitProc2 for JavaProcessorHolder {
    type Proc = JavaProc;

    fn with_parameters_mut(
        &mut self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &mut Self::Proc {
        assert_eq!(0, parameters.0);
        &mut self.0[parameters.0]
    }

    fn with_parameters(
        &self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &Self::Proc {
        assert_eq!(0, parameters.0);
        &self.0[parameters.0]
    }
}
impl CacheHolding<crate::processing::caches::Java> for JavaProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Java {
        &mut self.cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Java {
        &self.cache
    }
}

// impl CacheHolding<crate::processing::caches::Java> for JavaProcessorHolder {
//     fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Java {
//         &mut self.0.as_mut().unwrap().cache
//     }
//     fn get_caches(&self) -> &crate::processing::caches::Java {
//         &self.0.as_ref().unwrap().cache
//     }
// }

/// WARN be cautious about mutating that
/// TODO make something safer
#[doc(hidden)]
pub static SUB_QUERIES: &[&str] = &[
    r#"(method_invocation
    (identifier) (#EQ? "fail")
)"#,
    r#"(try_statement
    (block)
    (catch_clause)
)"#,
    r#"(marker_annotation 
    name: (identifier) (#EQ? "Test")
)"#,
    "(constructor_declaration)",
    "(class_declaration)",
    "(interface_declaration)",
    r#"(method_invocation
        name: (identifier) (#EQ? "sleep")
    )"#,
    r#"(marker_annotation
        name: (identifier) (#EQ? "Ignored")
    )"#,
    r#"(block
        "{"
        .
        "}"
    )"#,
    r#"(method_invocation
        (identifier) (#EQ? "assertEquals")
    )"#,
    r#"(method_invocation
        (identifier) (#EQ? "assertSame")
    )"#,
    r#"(method_invocation
        (identifier) (#EQ? "assertThat")
    )"#,
    r#"(program)"#,
];

#[doc(hidden)]
pub static TSG: &str = r#"
(program)@prog {
    node @prog.defs
    node @prog.lexical_scope
}
(class_declaration name:(_)@name)@class {
    node @class.defs
    attr (@class.defs) name = (source-text @name)
}
"#;

pub fn sub_queries() -> &'static [&'static str] {
    SUB_QUERIES
}

impl RepositoryProcessor {
    pub(crate) fn handle_java_file(
        &mut self,
        name: &ObjectName,
        text: &[u8],
    ) -> Result<java_tree_gen::FNode, ()> {
        todo!() // not used much anyway apart from  check_random_files_reserialization
                // crate::java::handle_java_file(&mut self.java_generator(text), name, text)
    }

    fn java_generator(
        &mut self,
        text: &[u8],
    ) -> java_tree_gen::JavaTreeGen<crate::TStore, SimpleStores, hyperast_tsquery::Query> {
        let line_break = if text.contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };

        let (precomp, _) = hyperast_tsquery::Query::with_precomputed(
            "(_)",
            hyperast_gen_ts_java::language(),
            sub_queries(),
        )
        .unwrap();
        unimplemented!()
        // java_tree_gen::JavaTreeGen {
        //     line_break,
        //     stores: self.main_stores.mut_with_ts(),
        //     md_cache: &mut self
        //         .processing_systems
        //         .mut_or_default::<JavaProcessorHolder>()
        //         .get_caches_mut()
        //         .md_cache, //java_md_cache,
        //     more: precomp,
        //     _p: Default::default(),
        // }
    }

    pub(crate) fn help_handle_java_folder<'a, 'b, 'c, 'd: 'c>(
        &'a mut self,
        repository: &'b Repository,
        dir_path: &'c mut Peekable<Components<'d>>,
        oid: Oid,
        name: &ObjectName,
        handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> <JavaAcc as hyperast::tree_gen::Accumulator>::Node {
        let full_node = self.handle_java_directory(repository, dir_path, name, oid, handle);
        let name = self.intern_object_name(name);
        (name, full_node)
    }

    fn handle_java_blob(
        &mut self,
        oid: Oid,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Result<(java_tree_gen::Local, IsSkippedAna), crate::ParseErr> {
        self.processing_systems
            .caching_blob_handler::<crate::processing::file_sys::Java>()
            .handle2(oid, repository, name, parameters, |c, n, t| {
                let line_break = if t.contains(&b'\r') {
                    "\r\n".as_bytes().to_vec()
                } else {
                    "\n".as_bytes().to_vec()
                };

                let holder = c.mut_or_default::<JavaProcessorHolder>();
                let java_proc = holder.with_parameters_mut(parameters.0);
                let md_cache = &mut java_proc.cache.md_cache;
                let stores = self
                    .main_stores
                    .mut_with_ts::<hyperast_gen_ts_java::types::TStore>();
                // let java_tree_gen =
                //     java_tree_gen::JavaTreeGen::new(stores, md_cache).with_line_break(line_break);
                #[cfg(not(feature = "tsg"))]
                let tsg: Option<()> = None;
                #[cfg(feature = "tsg")]
                let tsg = java_proc.tsg.as_ref();
                let r = if let Some(tsg) = tsg {
                    #[cfg(not(feature = "tsg"))]
                    panic!();
                    #[cfg(feature = "tsg")]
                    {
                        let spec: &tree_sitter_graph::ast::File<
                            hyperast_tsquery::QueryMatcher<_, &Acc>,
                        > = tsg.0.downcast_ref().unwrap();
                        let query = java_proc.query.as_ref().map(|x| &x.0);
                        let functions = tsg.1.clone();
                        let more = hyperast_tsquery::PreparedOverlay {
                            query,
                            overlayer: spec,
                            functions,
                        };
                        let mut java_tree_gen = java_tree_gen::JavaTreeGen::<
                            hyperast_gen_ts_java::types::TStore,
                            _,
                            _,
                        >::with_preprocessing(
                            stores, md_cache, more
                        )
                        .with_line_break(line_break);
                        crate::java::handle_java_file(&mut java_tree_gen, n, t)
                    }
                } else if let Some(precomp) = &java_proc.parameter.prepro {
                    let more = hyperast::scripting::Prepro::<_, _>::from_arc(precomp.clone());
                    // let mut java_tree_gen = java_tree_gen.with_more(more);
                    let mut java_tree_gen =
                        java_tree_gen::JavaTreeGen::with_preprocessing(stores, md_cache, more)
                            .with_line_break(line_break);
                    crate::java::handle_java_file(&mut java_tree_gen, n, t)
                } else if let Some(more) = &java_proc.query {
                    let more = &more.0;
                    let more: hyperast_tsquery::PreparedQuerying<_, _, _> = more.into();
                    let mut java_tree_gen =
                        java_tree_gen::JavaTreeGen::with_preprocessing(stores, md_cache, more)
                            .with_line_break(line_break);
                    crate::java::handle_java_file::<_>(&mut java_tree_gen, n, t)
                } else {
                    let mut java_tree_gen = java_tree_gen::JavaTreeGen::new(stores, md_cache)
                        .with_line_break(line_break);
                    crate::java::handle_java_file(&mut java_tree_gen, n, t)
                }
                .map_err(|_| crate::ParseErr::IllFormed)?;

                self.parsing_time += r.parsing_time;
                self.processing_time += r.processing_time;
                log::info!(
                    "parsing, processing, n, f: {} {} {} {}",
                    self.parsing_time.as_secs(),
                    self.processing_time.as_secs(),
                    java_proc.cache.md_cache.len(),
                    java_proc.cache.object_map.len()
                );

                let r = r.node;

                #[cfg(debug_assertions)]
                if let Ok(dd) = stores
                    .node_store
                    .resolve(r.local.compressed_node)
                    .get_component::<hyperast::scripting::lua_scripting::DerivedData>()
                {
                    log::info!("native: {:?} {:?}", r.local.mcc, r.local.metrics);
                    log::info!("script: {:?}", dd.0);
                }
                Ok((r.local.clone(), false))
            })
    }

    fn help_handle_java_file(
        &mut self,
        oid: Oid,
        w: &mut JavaAcc,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Result<(), crate::ParseErr> {
        let (full_node, skiped_ana) = self.handle_java_blob(oid, name, repository, parameters)?;
        let name = self.intern_object_name(name);
        assert!(!w.primary.children_names.contains(&name));
        let id = full_node.compressed_node;
        w.push(name, full_node, skiped_ana);
        if let Some(acc) = &mut w.scripting_acc {
            // SAFETY: this side should be fine, issue when unerasing
            let store = unsafe { self.main_stores.erase_ts_unchecked() };
            acc.acc::<_, hyperast_gen_ts_java::types::TType, _>(store, Type::Directory, id.into())
                .unwrap();
            // prepro_acc(
            //     acc,
            //     self.main_stores
            //         .mut_with_ts::<hyperast_gen_ts_java::types::TStore>(),
            //     &full_node,
            // );
        }
        Ok(())
    }

    /// oid : Oid of a dir such that */src/main/java/ or */src/test/java/
    fn handle_java_directory<'b, 'd: 'b>(
        &mut self,
        repository: &Repository,
        dir_path: &'b mut Peekable<Components<'d>>,
        name: &ObjectName,
        oid: git2::Oid,
        handle: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> (java_tree_gen::Local, IsSkippedAna) {
        JavaProcessor::<JavaAcc>::new(repository, self, dir_path, name, oid, &handle).process()
    }
}

// TODO try to separate processing from caching from git
#[cfg(test)]
#[allow(unused)]
mod experiments {
    use super::*;
    use crate::{
        git::{NamedObject, ObjectType, TypedObject, UniqueObject},
        processing::InFiles,
        Accumulator,
    };

    pub(crate) struct GitProcessorMiddleWare<'repo, 'prepro, 'd, 'c> {
        repository: &'repo Repository,
        prepro: &'prepro mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
    }

    impl<'repo, 'b, 'd, 'c> GitProcessorMiddleWare<'repo, 'b, 'd, 'c> {
        pub(crate) fn prepare_dir_exploration<It>(&self, current_object: It::Item) -> Vec<It::Item>
        where
            It: Iterator,
            It::Item: NamedObject + UniqueObject<Id = Oid>,
        {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            tree.iter()
                .rev()
                .map(|_| todo!())
                // .filter_map(|x| x.ok())
                .collect()
        }
    }

    impl<'repo, 'b, 'd, 'c> JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
        pub(crate) fn prepare_dir_exploration<T>(&self, current_object: &T) -> Vec<T>
        where
            T: NamedObject + UniqueObject<Id = git2::Oid>,
        {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            todo!()
        }
        pub(crate) fn stack(
            &mut self,
            current_object: BasicGitObject,
            prepared: Vec<BasicGitObject>,
            acc: JavaAcc,
        ) {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            self.stack
                .push(StackEle::new(*current_object.id(), prepared, acc));
        }
        fn pre(
            &mut self,
            current_object: BasicGitObject,
            already: Option<<JavaAcc as Accumulator>::Unlabeled>,
        ) -> Option<<JavaAcc as Accumulator>::Unlabeled> {
            match current_object.r#type() {
                ObjectType::Dir => {
                    if let Some(already) = already {
                        let full_node = already.clone();
                        return Some(full_node);
                    }
                    log::info!("tree {:?}", current_object.name().try_str());
                    let prepared: Vec<BasicGitObject> =
                        self.prepare_dir_exploration(&current_object);
                    let acc = JavaAcc::new(current_object.name().try_into().unwrap(), todo!());
                    self.stack(current_object, prepared, acc);
                    None
                }
                ObjectType::File => {
                    if crate::processing::file_sys::Java::matches(current_object.name()) {
                        self.prepro
                            .help_handle_java_file(
                                *current_object.id(),
                                &mut self.stack.last_mut().unwrap().acc,
                                current_object.name(),
                                self.repository,
                                *self.handle,
                            )
                            .unwrap();
                    } else {
                        log::debug!("not java source file {:?}", current_object.name().try_str());
                    }
                    None
                }
            }
        }
        fn post(
            &mut self,
            oid: Oid,
            acc: JavaAcc,
        ) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
            let skiped_ana = acc.skiped_ana;
            let name = &acc.primary.name;
            // let key = (oid, name.as_bytes().into());
            let name = self.prepro.intern_label(name);
            let full_node = make(acc, self.prepro.main_stores_mut().mut_with_ts());
            let full_node = (full_node, skiped_ana);
            todo!(
              // self.prepro
              // .processing_systems
              // .mut_or_default::<JavaProcessorHolder>()
              // .with_parameters_mut(?)
              // .cache.object_map
              // .insert(key, full_node.clone());
            );
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
                hyperast::tree_gen::Accumulator::push(w, (name, full_node));
                None
            }
        }
    }
}
