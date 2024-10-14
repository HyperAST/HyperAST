use crate::BasicDirAcc;
use crate::{
    preprocessed::IsSkippedAna, processing::ObjectName, Accumulator, MAX_REFS,
    PROPAGATE_ERROR_ON_BAD_CST_NODE,
};

use hyper_ast::store::defaults::NodeIdentifier;
use hyper_ast::{
    hashed::SyntaxNodeHashs, store::defaults::LabelIdentifier, tree_gen::SubTreeMetrics,
};
use hyper_ast_gen_ts_java::types::TStore;
use hyper_ast_gen_ts_java::{impact::partial_analysis::PartialAnalysis, types::Type};

use hyper_ast_gen_ts_java::legion_with_refs as java_tree_gen;

pub(crate) fn handle_java_file<'stores, 'cache, 'b: 'stores, More>(
    tree_gen: &mut java_tree_gen::JavaTreeGen<'stores, 'cache, TStore, More>,
    name: &ObjectName,
    text: &'b [u8],
) -> Result<java_tree_gen::FNode, ()>
where
    More: for<'a, 'c> hyper_ast_gen_ts_java::legion_with_refs::More<
        hyper_ast_gen_ts_java::legion_with_refs::MoreStore<'a, 'c, 'a, TStore>,
    >,
{
    let tree = match java_tree_gen::JavaTreeGen::<TStore>::tree_sitter_parse(text) {
        Ok(tree) => tree,
        Err(tree) => {
            log::warn!("bad CST: {:?}", name.try_str());
            log::debug!("{}", tree.root_node().to_sexp());
            if PROPAGATE_ERROR_ON_BAD_CST_NODE {
                return Err(());
            } else {
                tree
            }
        }
    };
    Ok(tree_gen.generate_file(&name.as_bytes(), text, tree.walk()))
}

type PrecompQueries = u16;

pub struct JavaAcc {
    /// Identifying elements and fundamental derived metrics used to accelerate deduplication.
    /// For example, hashing subtrees accelerates the deduplication process,
    /// but it requires to hash children and it can be done by accumulating hashes iteratively per child (see [`hyper_ast::hashed::inner_node_hash`]).
    pub primary: BasicDirAcc<NodeIdentifier, LabelIdentifier, SubTreeMetrics<SyntaxNodeHashs<u32>>>,
    pub skiped_ana: bool,
    pub ana: PartialAnalysis,
    pub precomp_queries: PrecompQueries,
}

impl JavaAcc {
    pub fn new(name: String) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
            ana: PartialAnalysis::init(&Type::Directory, None, |_| panic!()),
            skiped_ana: false,
            precomp_queries: Default::default(),
        }
    }
}

impl From<String> for JavaAcc {
    fn from(name: String) -> Self {
        Self::new(name)
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
    pub fn push(
        &mut self,
        name: LabelIdentifier,
        full_node: java_tree_gen::Local,
        skiped_ana: bool,
    ) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < MAX_REFS
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

impl hyper_ast::tree_gen::Accumulator for JavaAcc {
    type Node = (LabelIdentifier, (java_tree_gen::Local, IsSkippedAna));
    fn push(&mut self, (name, (full_node, skiped_ana)): Self::Node) {
        self.primary
            .push(name, full_node.compressed_node, full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < MAX_REFS
                && skiped_ana == false
                && self.skiped_ana == false
            {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
}

impl Accumulator for JavaAcc {
    type Unlabeled = (java_tree_gen::Local, IsSkippedAna);
}
