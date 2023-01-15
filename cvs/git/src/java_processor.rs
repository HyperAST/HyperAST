use std::{iter::Peekable, path::Components};

use git2::{Oid, Repository};
use hyper_ast::{
    cyclomatic::Mcc,
    filter::{Bloom, BloomSize, BF},
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::legion::{compo, NodeStore, PendingInsert, CS},
    },
    tree_gen::SubTreeMetrics,
    types::{LabelStore, Type},
};
use hyper_ast_gen_ts_java::{
    impact::partial_analysis::PartialAnalysis,
    legion_with_refs::{self, eq_node, BulkHasher},
};
use tuples::CombinConcat;

use crate::{
    git::BasicGitObject,
    java::JavaAcc,
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

pub struct JavaProcessor<'repo, 'prepro, 'd, 'c, Acc> {
    repository: &'repo Repository,
    prepro: &'prepro mut RepositoryProcessor,
    stack: Vec<(Oid, Vec<BasicGitObject>, Acc)>,
    pub dir_path: &'d mut Peekable<Components<'c>>,
}

impl<'repo, 'b, 'd, 'c, Acc: From<String>> JavaProcessor<'repo, 'b, 'd, 'c, Acc> {
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

impl<'repo, 'b, 'd, 'c> Processor<JavaAcc> for JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
    fn pre(&mut self, current_object: BasicGitObject) {
        match current_object {
            BasicGitObject::Tree(oid, name) => {
                if let Some(
                    // (already, skiped_ana)
                    already,
                ) = self.prepro.object_map_java.get(&(oid, name.clone()))
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
                    JavaAcc::new(std::str::from_utf8(&name).unwrap().to_string()),
                ));
            }
            BasicGitObject::Blob(oid, name) => {
                if name.ends_with(b".java") {
                    self.prepro.help_handle_java_file(
                        oid,
                        &mut self.stack.last_mut().unwrap().2,
                        name,
                        self.repository,
                    )
                } else {
                    log::debug!("not java source file {:?}", std::str::from_utf8(&name));
                }
            }
        }
    }
    fn post(&mut self, oid: Oid, acc: JavaAcc) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
        let skiped_ana = acc.skiped_ana;
        let name = acc.name.clone();
        let key = (oid, name.as_bytes().to_vec());
        let full_node = make(acc, self.prepro.main_stores_mut());
        self.prepro
            .object_map_java
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

    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, JavaAcc)> {
        &mut self.stack
    }
}

fn make(acc: JavaAcc, stores: &mut SimpleStores) -> hyper_ast_gen_ts_java::legion_with_refs::Local {
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
        let ana = {
            let ana = acc.ana;
            let ana = if acc.skiped_ana {
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
                for x in ana.display_refs(label_store) {
                    log::debug!("    {}", x);
                }
                log::debug!("decls in directory");
                for x in ana.display_decls(label_store) {
                    log::debug!("    {}", x);
                }
                let c = ana.estimated_refs_count();
                if c < MAX_REFS {
                    ana.resolve()
                } else {
                    ana
                }
            };
            log::info!(
                "ref count in dir after resolver {}",
                ana.lower_estimate_refs_count()
            );
            log::debug!("refs in directory after resolve: ");
            for x in ana.display_refs(label_store) {
                log::debug!("    {}", x);
            }
            ana
        };

        let hashs = hbuilder.build();

        let metrics = SubTreeMetrics {
            size,
            height,
            size_no_spaces,
            hashs,
        };

        (ana, metrics)
    };

    if let Some(id) = insertion.occupied_id() {
        let (ana, metrics) = compute_md();
        return legion_with_refs::Local {
            compressed_node: id,
            metrics,
            ana: Some(ana),
            mcc: Mcc::new(&Type::Directory),
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
        acc.skiped_ana,
        &ana,
    );

    let full_node = legion_with_refs::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana: Some(ana.clone()),
        mcc: Mcc::new(&Type::Directory),
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
    ana: &PartialAnalysis,
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
    // NOTE needed as macro because I only implemented BulkHasher and Bloom for u8 and u16
    macro_rules! bloom {
        ( $t:ty ) => {{
            type B = $t;
            let it = ana.solver.iter_refs();
            let it = BulkHasher::<_, <B as BF<[u8]>>::S, <B as BF<[u8]>>::H>::from(it);
            let bloom = B::from(it);
            (B::SIZE, bloom)
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
            match ana.estimated_refs_count() {
                x if x > 2048 || skiped_ana => {
                    insert!(c, (BloomSize::Much,))
                }
                x if x > 1024 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 64]>))
                }
                x if x > 512 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 32]>))
                }
                x if x > 256 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 16]>))
                }
                x if x > 150 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 8]>))
                }
                x if x > 100 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 4]>))
                }
                x if x > 30 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], [u64; 2]>))
                }
                x if x > 15 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u64>))
                }
                x if x > 8 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u32>))
                }
                x if x > 0 => {
                    insert!(c, bloom!(Bloom::<&'static [u8], u16>))
                }
                _ => insert!(c, (BloomSize::None,)),
            }
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
            self.stack.push((*current_object.id(), prepared, acc));
        }
        pub(crate) fn help_handle_java_file(&mut self, current_object: BasicGitObject) {
            self.prepro.help_handle_java_file(
                *current_object.id(),
                &mut self.stack.last_mut().unwrap().2,
                current_object.name().to_vec(),
                self.repository,
            )
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
                    log::info!("tree {:?}", std::str::from_utf8(current_object.name()));
                    let prepared: Vec<BasicGitObject> =
                        self.prepare_dir_exploration(&current_object);
                    let acc = JavaAcc::new(
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
        fn post(
            &mut self,
            oid: Oid,
            acc: JavaAcc,
        ) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
            let skiped_ana = acc.skiped_ana;
            let name = acc.name.clone();
            let key = (oid, name.as_bytes().to_vec());
            let full_node = make(acc, self.prepro.main_stores_mut());
            let full_node = (full_node, skiped_ana);
            self.prepro.object_map_java.insert(key, full_node.clone());
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
