use std::{
    fmt::{self, Debug},
    ops::AddAssign,
    path::PathBuf,
};

use hyper_ast::{
    hashed::SyntaxNodeHashs,
    position::{StructuralPosition, TreePath},
    store::defaults::{LabelIdentifier, NodeIdentifier},
    tree_gen::SubTreeMetrics,
};
use hyper_ast_gen_ts_cpp::legion as cpp_tree_gen;
use hyper_ast_gen_ts_xml::legion::XmlTreeGen;

use crate::{
    processing::ObjectName, Accumulator, DefaultMetrics, SimpleStores, TStore,
    PROPAGATE_ERROR_ON_BAD_CST_NODE,
};

pub(crate) fn handle_makefile_file<'a>(
    tree_gen: &mut XmlTreeGen<'a, TStore>,
    name: &ObjectName,
    text: &'a [u8],
) -> Result<MakeFile, ()> {
    let tree = match XmlTreeGen::<TStore>::tree_sitter_parse(b"<proj></proj>") {
        Ok(tree) => tree,
        Err(tree) => {
            log::warn!("bad CST");
            log::debug!("{:?}", name.try_str());
            log::debug!("{}", tree.root_node().to_sexp());
            if PROPAGATE_ERROR_ON_BAD_CST_NODE {
                return Err(());
            } else {
                tree
            }
        }
    };
    let x = tree_gen
        .generate_file(name.as_bytes(), b"<proj></proj>", tree.walk())
        .local;
    // TODO extract submodules, dependencies and directories. maybe even more ie. artefact id, ...
    let x = MakeFile {
        compressed_node: x.compressed_node,
        metrics: x.metrics,
        submodules: vec![],
        source_dirs: vec![".".to_owned()],
        test_source_dirs: vec!["../tests".to_owned()],
    };
    Ok(x)
}

#[derive(Debug, Clone)]
pub struct MakeFile {
    pub compressed_node: NodeIdentifier,
    pub metrics: DefaultMetrics,
    submodules: Vec<String>,
    source_dirs: Vec<String>,
    test_source_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MD {
    pub(crate) metrics: DefaultMetrics,
    #[allow(unused)] // TODO needed for scalable module level reference analysis
    pub(crate) ana: MakePartialAnalysis,
}

pub struct MakeModuleAcc {
    pub(crate) name: String,
    pub(crate) children_names: Vec<LabelIdentifier>,
    pub(crate) children: Vec<NodeIdentifier>,
    pub(crate) metrics: DefaultMetrics,
    pub(crate) ana: MakePartialAnalysis,
    pub(crate) sub_modules: Option<Vec<PathBuf>>,
    pub(crate) main_dirs: Option<Vec<PathBuf>>,
    pub(crate) test_dirs: Option<Vec<PathBuf>>,
}

impl From<String> for MakeModuleAcc {
    fn from(name: String) -> Self {
        Self {
            name,
            children_names: Default::default(),
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: MakePartialAnalysis::new(),
            sub_modules: None,
            main_dirs: None,
            test_dirs: None,
        }
    }
}

impl MakeModuleAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            children_names: Default::default(),
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: MakePartialAnalysis::new(),
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
            children_names: Default::default(),
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: MakePartialAnalysis::new(),
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

impl MakeModuleAcc {
    pub(crate) fn push_makefile(&mut self, name: LabelIdentifier, full_node: MakeFile) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.main_dirs = Some(full_node.source_dirs.iter().map(|x| x.into()).collect());
        self.test_dirs = Some(
            full_node
                .test_source_dirs
                .iter()
                .map(|x| x.into())
                .collect(),
        );
        self.sub_modules = Some(full_node.submodules.iter().map(|x| x.into()).collect());
        self.metrics.acc(full_node.metrics);
        // TODO
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub fn push_submodule(&mut self, name: LabelIdentifier, full_node: (NodeIdentifier, MD)) {
        self.children.push(full_node.0);
        self.children_names.push(name);
        self.metrics.acc(full_node.1.metrics);
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_source_file(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
        skiped_ana: bool,
    ) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
            size_no_spaces: full_node.metrics.size_no_spaces,
            line_count: 0,
        });
    }
    pub(crate) fn push_source_directory(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
    ) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
            size_no_spaces: full_node.metrics.size_no_spaces,
            line_count: 0,
        });
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_test_source_directory(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
    ) {
        self.children.push(full_node.compressed_node);
        self.children_names.push(name);
        self.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
            size_no_spaces: full_node.metrics.size_no_spaces,
            line_count: 0,
        });
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MakePartialAnalysis {
    submodules: Vec<()>,
    main_dirs: Vec<()>,
    test_dirs: Vec<()>,
}

impl MakePartialAnalysis {
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

pub struct IterMavenModules<'a, T: TreePath<NodeIdentifier>> {
    stores: &'a SimpleStores,
    path: T,
    stack: Vec<(NodeIdentifier, usize, Option<Vec<NodeIdentifier>>)>,
}

// impl<'a, T: TreePath<NodeIdentifier>> Debug for IterMavenModules<'a, T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         todo!()
//     }
// }

// impl<'a, T: TreePath<NodeIdentifier> + Debug + Clone> Iterator for IterMavenModules<'a, T> {
//     type Item = T;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             let (node, offset, children) = self.stack.pop()?;
//             if let Some(children) = children {
//                 if offset < children.len() {
//                     let child = children[offset];
//                     self.path.check(&self.stores).unwrap();
//                     {
//                         let b = self.stores.node_store.resolve(node);
//                         if b.has_children() {
//                             let len = b.child_count();
//                             let cs = b.children().unwrap();
//                             // println!("children: {:?} {} {:?}", node,cs.len(),cs);
//                             assert!(offset < len as usize);
//                             assert_eq!(child, cs[offset as u16]);
//                         } else {
//                             panic!()
//                         }
//                     }
//                     if offset == 0 {
//                         match self.path.node() {
//                             Some(x) => assert_eq!(*x, node),
//                             None => {}
//                         }
//                         self.path.goto(child, offset);
//                         self.path.check(&self.stores).unwrap();
//                     } else {
//                         match self.path.node() {
//                             Some(x) => assert_eq!(*x, children[offset - 1]),
//                             None => {}
//                         }
//                         self.path.inc(child);
//                         assert_eq!(*self.path.offset().unwrap(), offset + 1);
//                         self.path.check(&self.stores).expect(&format!(
//                             "{:?} {} {:?} {:?} {:?}",
//                             node, offset, child, children, self.path
//                         ));
//                     }
//                     self.stack.push((node, offset + 1, Some(children)));
//                     self.stack.push((child, 0, None));
//                     continue;
//                 } else {
//                     self.path.check(&self.stores).unwrap();
//                     self.path.pop().expect("should not go higher than root");
//                     self.path.check(&self.stores).unwrap();
//                     continue;
//                 }
//             } else {
//                 let b = self.stores.node_store.resolve(node);

//                 if self.is_dead_end(&b) {
//                     continue;
//                 }

//                 if b.has_children() {
//                     let children = b.children();
//                     self.stack.push((
//                         node,
//                         0,
//                         Some(children.unwrap().iter_children().cloned().collect()),
//                     ));
//                 }

//                 if self.is_matching(&b) {
//                     self.path.check(&self.stores).unwrap();
//                     return Some(self.path.clone());
//                 }
//             }
//         }
//     }
// }

// impl<'a, T: TreePath<NodeIdentifier>> IterMavenModules<'a, T> {
//     pub fn new(stores: &'a SimpleStores, path: T, root: NodeIdentifier) -> Self {
//         let stack = vec![(root, 0, None)];
//         Self {
//             stores,
//             path,
//             stack,
//         }
//     }

//     fn is_dead_end(&self, b: &hyper_ast::store::nodes::legion::HashedNodeRef) -> bool {
//         let t = b.get_type();
//         let is_src = if b.has_label() {
//             self.stores.label_store.resolve(b.get_label()).eq("src")
//         } else {
//             false
//         };

//         is_src || t != Type::MavenDirectory
//     }
//     fn is_matching(&self, b: &hyper_ast::store::nodes::legion::HashedNodeRef) -> bool {
//         let contains_pom = b
//             .children()
//             .unwrap()
//             .iter_children()
//             .find(|x| {
//                 if let Some(n) = self.stores.node_store.try_resolve(**x) {
//                     log::debug!("f {:?}", n.get_type());
//                     n.get_type().eq(&Type::xml_SourceFile)
//                         && if n.has_label() {
//                             log::debug!(
//                                 "f name: {:?}",
//                                 self.stores.label_store.resolve(n.get_label())
//                             );
//                             self.stores.label_store.resolve(n.get_label()).eq("pom.xml")
//                         } else {
//                             false
//                         }
//                 } else {
//                     false
//                 }
//             })
//             .is_some();
//         contains_pom
//     }
// }

impl hyper_ast::tree_gen::Accumulator for MakeModuleAcc {
    type Node = (LabelIdentifier, (NodeIdentifier, MD));
    fn push(&mut self, (name, full_node): Self::Node) {
        self.children.push(full_node.0);
        self.children_names.push(name);
        self.metrics.acc(full_node.1.metrics);
        // TODO ana
        // full_node.2.acc(&Type::Directory, &mut self.ana);
    }

    // fn push(
    //     &mut self,
    //     _full_node: (NodeIdentifier, MD),
    // ) {
    //     panic!()
    // }
}

impl Accumulator for MakeModuleAcc {
    type Unlabeled = (NodeIdentifier, MD);
    // fn push(
    //     &mut self,
    //     name: LabelIdentifier,
    //     full_node: (NodeIdentifier, MD),
    // ) {
    //     self.children.push(full_node.0);
    //     self.children_names.push(name);
    //     self.metrics.acc(full_node.1.metrics);
    //     // TODO ana
    //     // full_node.2.acc(&Type::Directory, &mut self.ana);
    // }
}
