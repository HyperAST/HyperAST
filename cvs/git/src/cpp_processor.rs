use std::{iter::Peekable, path::Components};

use git2::{Oid, Repository};
use hyper_ast::{
    filter::{BloomSize},
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{compo, compo::CS, NodeStore, PendingInsert},
    },
    tree_gen::SubTreeMetrics,
    types::LabelStore,
};
use hyper_ast_gen_ts_cpp::{
    legion::{self, eq_node},
    types::Type,
};
use tuples::CombinConcat;

use crate::{
    cpp::CppAcc,
    git::BasicGitObject,
    preprocessed::{IsSkippedAna, RepositoryProcessor},
    Processor, SimpleStores, MAX_REFS,
};

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
}

impl<'repo, 'b, 'd, 'c, Acc: From<String>> CppProcessor<'repo, 'b, 'd, 'c, Acc> {
    pub(crate) fn new(
        repository: &'repo Repository,
        prepro: &'b mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
        name: &[u8],
        oid: git2::Oid,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree);
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

impl<'repo, 'b, 'd, 'c> Processor<CppAcc> for CppProcessor<'repo, 'b, 'd, 'c, CppAcc> {
    fn pre(&mut self, current_object: BasicGitObject) {
        match current_object {
            BasicGitObject::Tree(oid, name) => {
                if let Some(
                    // (already, skiped_ana)
                    already,
                ) = self.prepro.object_map_cpp.get(&(oid, name.clone()))
                {
                    // reinit already computed node for post order
                    let full_node = already.clone();
                    // let skiped_ana = *skiped_ana;
                    let w = &mut self.stack.last_mut().unwrap().2;
                    let name = self
                        .prepro
                        .intern_label(std::str::from_utf8(&name).unwrap());
                    assert!(!w.children_names.contains(&name));
                    hyper_ast::tree_gen::Accumulator::push(w, (name, full_node));
                    // w.push(name, full_node, skiped_ana);
                    return;
                }
                log::info!("tree {:?}", std::str::from_utf8(&name));
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared: Vec<BasicGitObject> = prepare_dir_exploration(tree);
                self.stack.push((
                    oid,
                    prepared,
                    CppAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
                ));
            }
            BasicGitObject::Blob(oid, name) => {
                if name.ends_with(b".cpp") {
                    self.prepro.help_handle_cpp_file(
                        oid,
                        &mut self.stack.last_mut().unwrap().2,
                        name,
                        self.repository,
                    )
                } else if name.ends_with(b".h") || name.ends_with(b".hpp") {
                    self.prepro.help_handle_cpp_file(
                        oid,
                        &mut self.stack.last_mut().unwrap().2,
                        name,
                        self.repository,
                    )
                } else {
                    log::debug!("not cpp source file {:?}", std::str::from_utf8(&name));
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: CppAcc) -> Option<(legion::Local, IsSkippedAna)> {
        let skiped_ana = true;
        let name = acc.name.clone();
        let key = (oid, name.as_bytes().to_vec());
        let full_node = make(acc, self.prepro.main_stores_mut());
        self.prepro
            .object_map_cpp
            .insert(key, (full_node.clone(), skiped_ana));
        let name = self.prepro.main_stores.label_store.get_or_insert(name);
        if self.stack.is_empty() {
            Some((full_node, skiped_ana))
        } else {
            let w = &mut self.stack.last_mut().unwrap().2;
            assert!(
                !w.children_names.contains(&name),
                "{:?} {:?}",
                w.children_names,
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

fn make(acc: CppAcc, stores: &mut SimpleStores) -> hyper_ast_gen_ts_cpp::legion::Local {
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;

    let hashs = acc.metrics.hashs;
    let size = acc.metrics.size + 1;
    let height = acc.metrics.height + 1;
    let size_no_spaces = acc.metrics.size_no_spaces + 1;
    let hbuilder = hashed::Builder::new(hashs, &Type::Directory, &acc.name, size_no_spaces);
    let hashable = &hbuilder.most_discriminating();
    let label_id = label_store.get_or_insert(acc.name.clone());

    let eq = eq_node(&Type::Directory, Some(&label_id), &acc.children);

    let insertion = node_store.prepare_insertion(&hashable, eq);

    let compute_md = || {
        let hashs = hbuilder.build();

        let metrics = SubTreeMetrics {
            size,
            height,
            size_no_spaces,
            hashs,
        };

        (None, metrics)
    };

    if let Some(id) = insertion.occupied_id() {
        let (ana, metrics) = compute_md();
        return legion::Local {
            compressed_node: id,
            metrics,
            ana,
        };
    }

    let (ana, metrics) = compute_md();
    let hashs = hbuilder.build();
    let node_id = compress(
        insertion,
        label_id,
        acc.children,
        acc.children_names,
        size,
        height,
        size_no_spaces,
        hashs,
        true,
        &Default::default(),
    );

    let full_node = legion::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana: Some(Default::default()),
    };
    full_node
}

fn compress(
    insertion: PendingInsert,
    label_id: LabelIdentifier,
    children: Vec<NodeIdentifier>,
    children_names: Vec<LabelIdentifier>,
    size: u32,
    height: u32,
    size_no_spaces: u32,
    hashs: SyntaxNodeHashs<u32>,
    skiped_ana: bool,
    ana: &legion::PartialAnalysis,
) -> NodeIdentifier {
    let vacant = insertion.vacant();
    macro_rules! insert {
        ( $c0:expr, $($c:expr),* $(,)? ) => {{
            let c = $c0;
            $(
                let c = c.concat($c);
            )*
            NodeStore::insert_after_prepare(vacant, c)
        }};
    }
    match children.len() {
        0 => insert!((Type::Directory, label_id, hashs, BloomSize::None),),
        _ => {
            assert_eq!(children_names.len(), children.len());
            let c = (
                Type::Directory,
                label_id,
                compo::Size(size),
                compo::Height(height),
                compo::SizeNoSpaces(size_no_spaces),
                hashs,
                CS(children_names.into_boxed_slice()),
                CS(children.into_boxed_slice()),
            );
            insert!(c, (BloomSize::Much,))
        }
    }
}

// TODO try to separate processing from caching from git
#[cfg(test)]
#[allow(unused)]
mod experiments {
    use crate::{
        git::{NamedObject, ObjectType, TypedObject, UniqueObject},
        Accumulator,
    };

    use super::*;

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

    impl<'repo, 'b, 'd, 'c> CppProcessor<'repo, 'b, 'd, 'c, CppAcc> {
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
            acc: CppAcc,
        ) {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            self.stack.push((*current_object.id(), prepared, acc));
        }
        pub(crate) fn help_handle_java_file(&mut self, current_object: BasicGitObject) {
            self.prepro.help_handle_cpp_file(
                *current_object.id(),
                &mut self.stack.last_mut().unwrap().2,
                current_object.name().to_vec(),
                self.repository,
            )
        }
        fn pre(
            &mut self,
            current_object: BasicGitObject,
            already: Option<<CppAcc as Accumulator>::Unlabeled>,
        ) -> Option<<CppAcc as Accumulator>::Unlabeled> {
            match current_object.r#type() {
                ObjectType::Dir => {
                    if let Some(already) = already {
                        let full_node = already.clone();
                        return Some(full_node);
                    }
                    log::info!("tree {:?}", std::str::from_utf8(current_object.name()));
                    let prepared: Vec<BasicGitObject> =
                        self.prepare_dir_exploration(&current_object);
                    let acc = CppAcc::new(
                        std::str::from_utf8(current_object.name())
                            .unwrap()
                            .to_string(),
                    );
                    self.stack(current_object, prepared, acc);
                    None
                }
                ObjectType::File => {
                    if current_object.name().ends_with(b".java") {
                        self.help_handle_java_file(current_object)
                    } else {
                        log::debug!(
                            "not java source file {:?}",
                            std::str::from_utf8(current_object.name())
                        );
                    }
                    None
                }
            }
        }
        fn post(&mut self, oid: Oid, acc: CppAcc) -> Option<(legion::Local, IsSkippedAna)> {
            let skiped_ana = true;
            let name = acc.name.clone();
            let key = (oid, name.as_bytes().to_vec());
            let full_node = make(acc, self.prepro.main_stores_mut());
            let full_node = (full_node, skiped_ana);
            self.prepro.object_map_cpp.insert(key, full_node.clone());
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
                hyper_ast::tree_gen::Accumulator::push(w, (name, full_node));
                None
            }
        }
    }
}
