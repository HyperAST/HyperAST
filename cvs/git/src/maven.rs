use std::{
    fmt::{self, Debug},
    ops::AddAssign,
    path::PathBuf,
};

use hyper_ast::{
    hashed::SyntaxNodeHashs,
    store::nodes::DefaultNodeIdentifier as NodeIdentifier,
    tree_gen::SubTreeMetrics,
    types::{LabelStore as _, Labeled, Tree, Type, Typed, WithChildren},
};
use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;
use rusted_gumtree_gen_ts_xml::xml_tree_gen::XmlTreeGen;

use crate::{SimpleStores, FAIL_ON_BAD_CST_NODE};

pub(crate) fn handle_pom_file(
    tree_gen: &mut XmlTreeGen,
    name: &[u8],
    text: &[u8],
) -> Result<POM, ()> {
    use tree_sitter::{Language, Parser};

    let mut parser = Parser::new();

    extern "C" {
        fn tree_sitter_xml() -> Language;
    }
    {
        let language = unsafe { tree_sitter_xml() };
        parser.set_language(language).unwrap();
    }

    let tree = parser.parse(text, None).unwrap();
    if tree.root_node().has_error() {
        println!("bad CST");
        // println!("{}", name);
        println!("{}", tree.root_node().to_sexp());

        if FAIL_ON_BAD_CST_NODE {
            return Err(());
        }
    }
    let x = tree_gen.generate_file(&name, text, tree.walk()).local;
    let x = POM {
        compressed_node: x.compressed_node,
        metrics: x.metrics,
        submodules: vec![],
        source_dirs: vec!["src/main/java".to_owned()],
        test_source_dirs: vec!["src/test/java".to_owned()],
    };
    Ok(x)
}

#[derive(Debug, Clone)]
pub struct POM {
    pub compressed_node: NodeIdentifier,
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    submodules: Vec<String>,
    source_dirs: Vec<String>,
    test_source_dirs: Vec<String>,
}

pub struct IterMavenModules<'a> {
    stores: &'a SimpleStores,
    parents: Vec<NodeIdentifier>,
    offsets: Vec<usize>,
    /// to tell that we need to pop a parent, we could also use a bitvec instead of Option::None
    remaining: Vec<Option<NodeIdentifier>>,
}

impl<'a> Debug for IterMavenModules<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterMavenModules")
            .field("parents", &self.parents())
            .field("offsets", &self.offsets())
            .field("remaining", &self.remaining)
            .finish()
    }
}

impl<'a> Iterator for IterMavenModules<'a> {
    type Item = NodeIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        let x;
        loop {
            if let Some(c) = self.remaining.pop()? {
                self.offsets.last_mut().unwrap().add_assign(1);
                x = c;
                break;
            } else {
                self.offsets.pop();
                self.parents.pop();
            }
        }

        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();

        let is_src = if b.has_label() {
            self.stores.label_store.resolve(b.get_label()).eq("src")
        } else {
            false
        };

        if is_src {
            return self.next();
        } else if t != Type::MavenDirectory {
            return self.next();
        }

        self.parents.push(x);
        self.offsets.push(0);
        self.remaining.push(None);
        if b.has_children() {
            self.remaining
                .extend(b.get_children().iter().rev().map(|x| Some(*x)));
        }

        let contains_pom = b
            .get_children()
            .iter()
            .find(|x| {
                if let Some(n) = self.stores.node_store.try_resolve(**x) {
                    println!("f {:?}", n.get_type());
                    n.get_type().eq(&Type::xml_SourceFile)
                        && if n.has_label() {
                            println!(
                                "f name: {:?}",
                                self.stores.label_store.resolve(n.get_label())
                            );
                            self.stores.label_store.resolve(n.get_label()).eq("pom.xml")
                        } else {
                            false
                        }
                } else {
                    false
                }
            })
            .is_some();

        if contains_pom {
            Some(x)
        } else {
            while !self.remaining.is_empty() {
                if let Some(x) = self.next() {
                    return Some(x);
                }
            }
            None
        }
    }
}

impl<'a> IterMavenModules<'a> {
    pub fn new(stores: &'a SimpleStores, root: NodeIdentifier) -> Self {
        Self {
            stores,
            parents: vec![],
            offsets: vec![0],
            remaining: vec![Some(root)],
        }
    }
    pub fn parents(&self) -> &[NodeIdentifier] {
        &self.parents[..self.parents.len() - 1]
    }
    pub fn offsets(&self) -> &[usize] {
        &self.offsets[..self.offsets.len() - 1]
    }
}

#[derive(Debug, Clone)]
pub struct MD {
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) ana: MavenPartialAnalysis,
}

pub struct MavenModuleAcc {
    pub(crate) name: String,
    pub(crate) children: Vec<hyper_ast::store::nodes::DefaultNodeIdentifier>,
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>, //java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) ana: MavenPartialAnalysis,
    pub(crate) sub_modules: Option<Vec<PathBuf>>,
    pub(crate) main_dirs: Option<Vec<PathBuf>>,
    pub(crate) test_dirs: Option<Vec<PathBuf>>,
}

impl MavenModuleAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: MavenPartialAnalysis::new(),
            sub_modules: None,
            main_dirs: None,
            test_dirs: None,
        }
    }
    pub(crate) fn with_content(
        name: String,
        sub_modules: Vec<PathBuf>,
        main_dirs: Vec<PathBuf>,
        test_dirs: Vec<PathBuf>,
    ) -> Self {
        Self {
            name,
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: MavenPartialAnalysis::new(),
            sub_modules: if sub_modules.is_empty() {
                None
            } else {
                Some(sub_modules)
            },
            main_dirs: if main_dirs.is_empty() {
                None
            } else {
                Some(main_dirs)
            },
            test_dirs: if test_dirs.is_empty() {
                None
            } else {
                Some(test_dirs)
            },
        }
    }
}

impl MavenModuleAcc {
    // pub(crate) fn push_java(&mut self, full_node: java_tree_gen::FNode) {
    //     self.children.push(full_node.local.compressed_node.clone());
    //     self.metrics.acc(full_node.local.metrics);
    //     full_node
    //         .local
    //         .ana
    //         .unwrap()
    //         .acc(&Type::Directory, &mut self.ana);
    // }
    // pub(crate) fn push_xml(&mut self, full_node: xml_tree_gen::FNode) {
    //     self.children.push(full_node.local.compressed_node.clone());
    //     let m = full_node.local.metrics;
    //     let m = java_tree_gen::SubTreeMetrics {
    //         hashs: m.hashs,
    //         size: m.size,
    //         height: m.height,
    //     };
    //     self.metrics.acc(m);
    //     // full_node
    //     //     .local
    //     //     .ana
    //     //     .unwrap()
    //     //     .acc(&Type::Directory, &mut self.ana);
    // }
    pub(crate) fn push_pom(&mut self, full_node: POM) {
        self.children.push(full_node.compressed_node);
        self.main_dirs = Some(full_node.source_dirs.iter().map(|x| x.into()).collect());
        self.test_dirs = Some(
            full_node
                .test_source_dirs
                .iter()
                .map(|x| x.into())
                .collect(),
        );
        self.sub_modules = Some(full_node.submodules.iter().map(|x| x.into()).collect());
        // TODO
        // self.metrics.acc(full_node.1);
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_submodule(
        &mut self,
        full_node: (hyper_ast::store::nodes::DefaultNodeIdentifier, MD),
    ) {
        self.children.push(full_node.0);
        self.metrics.acc(full_node.1.metrics);
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_source_directory(&mut self, full_node: java_tree_gen::Local) {
        self.children.push(full_node.compressed_node);
        self.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
        });
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_test_source_directory(&mut self, full_node: java_tree_gen::Local) {
        self.children.push(full_node.compressed_node);
        self.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
        });
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MavenPartialAnalysis {
    submodules: Vec<()>,
    main_dirs: Vec<()>,
    test_dirs: Vec<()>,
}

impl MavenPartialAnalysis {
    pub(crate) fn new() -> Self {
        // TODO
        Self {
            submodules: vec![],
            main_dirs: vec![],
            test_dirs: vec![],
        }
    }
    pub(crate) fn resolve(&self) -> Self {
        Self {
            submodules: self.submodules.clone(),
            main_dirs: self.main_dirs.clone(),
            test_dirs: self.test_dirs.clone(),
        }
    }
}
