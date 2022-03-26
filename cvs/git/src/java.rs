use crate::{FAIL_ON_BAD_CST_NODE, MAX_REFS};

use hyper_ast::{
    hashed::{SyntaxNodeHashs},
    types::Type,
};
use rusted_gumtree_gen_ts_java::impact::partial_analysis::PartialAnalysis;

use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;

pub(crate) fn handle_java_file(
    tree_gen: &mut java_tree_gen::JavaTreeGen,
    name: &[u8],
    text: &[u8],
) -> Result<java_tree_gen::FNode, ()> {
    use tree_sitter::{Language, Parser};

    let mut parser = Parser::new();

    extern "C" {
        fn tree_sitter_java() -> Language;
    }
    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }

    let tree = parser.parse(text, None).unwrap();
    if tree.root_node().has_error() {
        println!("bad CST");
        // println!("{}", name);
        println!("{}", tree.root_node().to_sexp());
        // {
        //     let mut fe = PathBuf::new();
        //     fe.extend(&[
        //         "/home/quentin/resources/file_error",
        //         repo_name,
        //         &oid.to_string(),
        //         x,
        //     ]);
        //     std::fs::create_dir_all(&fe).unwrap();
        //     fe.extend(&[&y.name().unwrap()]);
        //     let mut fe = fs::File::create(&fe).unwrap();
        //     fe.write(a.content()).unwrap();

        //     let mut fe = PathBuf::new();
        //     fe.extend(&[
        //         "/home/quentin/resources/tree_error",
        //         repo_name,
        //         &oid.to_string(),
        //         x,
        //     ]);
        //     std::fs::create_dir_all(&fe).unwrap();
        //     fe.extend(&[&y.name().unwrap()]);
        //     let mut fe = fs::File::create(&fe).unwrap();
        //     fe.write(tree.root_node().to_sexp().as_bytes()).unwrap();
        // }
        if FAIL_ON_BAD_CST_NODE {
            return Err(());
        }
    }
    Ok(tree_gen.generate_file(&name, text, tree.walk()))
}

pub struct JavaAcc {
    pub(crate) name: String,
    pub(crate) children: Vec<hyper_ast::store::nodes::DefaultNodeIdentifier>,
    pub(crate) metrics: java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) ana: PartialAnalysis,
    pub(crate) skiped_ana: bool,
}

impl JavaAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: PartialAnalysis::init(&Type::Directory, None, |x| panic!()),
            skiped_ana: false,
        }
    }
}

impl JavaAcc {
    pub(crate) fn push_file(&mut self, full_node: java_tree_gen::FNode) {
        self.children.push(full_node.local.compressed_node.clone());
        self.metrics.acc(full_node.local.metrics);
        full_node
            .local
            .ana
            .unwrap()
            .acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push(&mut self, full_node: java_tree_gen::Local) {
        self.children.push(full_node.compressed_node);
        self.metrics.acc(full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.refs_count() < MAX_REFS && self.skiped_ana == false {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
    pub(crate) fn push_dir(&mut self, full_node: java_tree_gen::Local, skiped_ana: bool) {
        self.children.push(full_node.compressed_node);
        self.metrics.acc(full_node.metrics);

        if let Some(ana) = full_node.ana {
            if ana.refs_count() < MAX_REFS && skiped_ana == false && self.skiped_ana == false {
                ana.acc(&Type::Directory, &mut self.ana);
            } else {
                self.skiped_ana = true;
            }
        }
    }
}
