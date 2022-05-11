use crate::{MAX_REFS, PROPAGATE_ERROR_ON_BAD_CST_NODE};

use hyper_ast::{hashed::SyntaxNodeHashs, store::labels::DefaultLabelIdentifier, types::Type};
use rusted_gumtree_gen_ts_java::impact::partial_analysis::PartialAnalysis;

use rusted_gumtree_gen_ts_java::legion_with_refs as java_tree_gen;

pub(crate) fn handle_java_file<'a,'b:'a>(
    tree_gen: &mut java_tree_gen::JavaTreeGen<'a>,
    name: &[u8],
    text: &'b [u8],
) -> Result<java_tree_gen::FNode, ()> {
    let tree = match java_tree_gen::JavaTreeGen::tree_sitter_parse(text) {
        Ok(tree) => tree,
        Err(tree) => {
            log::warn!("bad CST");
            // println!("{}", name);
            log::debug!("{}", tree.root_node().to_sexp());
            if PROPAGATE_ERROR_ON_BAD_CST_NODE {
                return Err(());
            } else {
                tree
            }
        }
    };
    Ok(tree_gen.generate_file(&name, text, tree.walk()))
}

pub struct JavaAcc {
    pub(crate) name: String,
    pub(crate) children_names: Vec<DefaultLabelIdentifier>,
    pub(crate) children: Vec<hyper_ast::store::nodes::DefaultNodeIdentifier>,
    pub(crate) metrics: java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) ana: PartialAnalysis,
    pub(crate) skiped_ana: bool,
}

impl JavaAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            children_names: Default::default(),
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: PartialAnalysis::init(&Type::Directory, None, |_| panic!()),
            skiped_ana: false,
        }
    }
}

impl JavaAcc {
    pub(crate) fn push_file(
        &mut self,
        name: DefaultLabelIdentifier,
        full_node: java_tree_gen::FNode,
    ) {
        self.children.push(full_node.local.compressed_node.clone());
        self.children_names.push(name);
        self.metrics.acc(full_node.local.metrics);
        full_node
            .local
            .ana
            .unwrap()
            .acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push(&mut self, name: DefaultLabelIdentifier, full_node: java_tree_gen::Local) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.estimated_refs_count() < MAX_REFS && self.skiped_ana == false {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
    pub(crate) fn push_dir(
        &mut self,
        name: DefaultLabelIdentifier,
        full_node: java_tree_gen::Local,
        skiped_ana: bool,
    ) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(full_node.metrics);

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
