use std::time::Instant;

use crate::java_processor::SimpleStores;
use crate::{Accumulator, PROPAGATE_ERROR_ON_BAD_CST_NODE, processing::ObjectName};
use crate::{BasicDirAcc, FailedParsing, FileProcessingResult, SuccessProcessing};

use hyperast::store::defaults::NodeIdentifier;
use hyperast::tree_gen;
use hyperast::{
    hashed::SyntaxNodeHashs, store::defaults::LabelIdentifier, tree_gen::SubTreeMetrics,
};
use hyperast_gen_ts_java::types::TStore;
use hyperast_gen_ts_java::{legion_with_refs::PartialAnalysis, types::Type};

use hyperast_gen_ts_java::legion_with_refs as java_tree_gen;

pub(crate) fn handle_java_file<'stores, 'cache, 'b: 'stores, More>(
    tree_gen: &mut java_tree_gen::JavaTreeGen<
        'stores,
        'cache,
        TStore,
        hyperast::store::SimpleStores<TStore>,
        More,
    >,
    name: &ObjectName,
    text: &'b [u8],
) -> FileProcessingResult<java_tree_gen::FNode>
where
    More: tree_gen::Prepro<SimpleStores, Scope = hyperast::scripting::Acc>
        + tree_gen::PreproTSG<SimpleStores, Acc = java_tree_gen::Acc>,
{
    let time = Instant::now();
    let tree = java_tree_gen::tree_sitter_parse(text);
    let parsing_time = time.elapsed();
    let tree = match tree {
        Ok(tree) => tree,
        Err(tree) => {
            log::warn!("bad CST: {:?}", name.try_str());
            log::debug!("{}", tree.root_node().to_sexp());
            if PROPAGATE_ERROR_ON_BAD_CST_NODE {
                return Err(FailedParsing {
                    parsing_time,
                    tree,
                    error: "CST contains parsing errors",
                });
            } else {
                tree
            }
        }
    };
    let time = Instant::now();
    let node = tree_gen.generate_file(&name.as_bytes(), text, tree.walk());
    let processing_time = time.elapsed();
    Ok(SuccessProcessing {
        parsing_time,
        processing_time,
        node,
    })
}

type PrecompQueries = u16;

pub struct JavaAcc {
    /// Identifying elements and fundamental derived metrics used to accelerate deduplication.
    /// For example, hashing subtrees accelerates the deduplication process,
    /// but it requires to hash children and it can be done by accumulating hashes iteratively per child (see [`hyperast::hashed::inner_node_hash`]).
    pub primary: BasicDirAcc<NodeIdentifier, LabelIdentifier, SubTreeMetrics<SyntaxNodeHashs<u32>>>,
    pub skiped_ana: bool,
    pub ana: PartialAnalysis,
    pub precomp_queries: PrecompQueries,
    pub scripting_acc: Option<hyperast::scripting::Acc>,
}

impl JavaAcc {
    pub fn new(name: String, prepro: Option<hyperast::scripting::Acc>) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
            ana: PartialAnalysis::init(&Type::Directory, None, |_| panic!()),
            skiped_ana: false,
            precomp_queries: Default::default(),
            scripting_acc: prepro,
        }
    }
}

impl From<String> for JavaAcc {
    fn from(name: String) -> Self {
        Self::new(name, None)
    }
}

impl JavaAcc {
    // pub(crate) fn push_file(
    //     &mut self,
    //     name: LabelIdentifier,
    //     full_node: java_tree_gen::FNode,
    // ) {
    //     self.children.push(full_node.local.compressed_node.clone());
    //     self.children_names.push(name);
    //     self.metrics.acc(full_node.local.metrics);
    //     full_node
    //         .local
    //         .ana
    //         .unwrap()
    //         .acc(&Type::Directory, &mut self.ana);
    // }
    // pub(crate) fn push(&mut self, name: LabelIdentifier, full_node: java_tree_gen::Local) {
    //     self.children.push(full_node.compressed_node);
    //     self.children_names.push(name);
    //     self.metrics.acc(full_node.metrics);

    //     if let Some(ana) = full_node.ana {
    //         if ana.estimated_refs_count() < MAX_REFS && self.skiped_ana == false {
    //             ana.acc(&Type::Directory, &mut self.ana);
    //         } else {
    //             self.skiped_ana = true;
    //         }
    //     }
    // }
    pub fn push(&mut self, name: LabelIdentifier, full_node: java_tree_gen::Local) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);

        #[cfg(feature = "impact")]
        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < crate::MAX_REFS
                && skiped_ana == false
                && self.skiped_ana == false
            {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
        self.precomp_queries |= full_node.precomp_queries;
    }
}

impl hyperast::tree_gen::Accumulator for JavaAcc {
    type Node = (LabelIdentifier, (java_tree_gen::Local,));
    fn push(&mut self, (name, (full_node,)): Self::Node) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);

        #[cfg(feature = "impact")]
        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < crate::MAX_REFS
                && skiped_ana == false
                && self.skiped_ana == false
            {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
        self.precomp_queries |= full_node.precomp_queries;
    }
}

impl Accumulator for JavaAcc {
    type Unlabeled = (java_tree_gen::Local,);
}
