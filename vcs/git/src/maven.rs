use crate::{
    Accumulator, BasicDirAcc, DefaultMetrics, PROPAGATE_ERROR_ON_BAD_CST_NODE, ParseErr,
    SimpleStores, processing::ObjectName,
};
use enumset::EnumSet;
use hyperast::{
    position::{StructuralPosition, TreePath, TreePathMut},
    store::defaults::{LabelIdentifier, NodeIdentifier},
    tree_gen::SubTreeMetrics,
    types::{Childrn, LabelStore as _, Labeled, Tree, Typed, WithChildren},
};
use hyperast_gen_ts_java::legion_with_refs as java_tree_gen;
use hyperast_gen_ts_xml::{
    legion::XmlTreeGen,
    types::{TStore, Type},
};
use num::ToPrimitive;
use std::{
    fmt::{self, Debug},
    ops::AddAssign,
    path::PathBuf,
};

pub(crate) fn handle_pom_file<'a>(
    tree_gen: &mut XmlTreeGen<'a, TStore>,
    name: &ObjectName,
    text: &'a [u8],
) -> Result<POM, ParseErr> {
    let tree = match XmlTreeGen::<TStore>::tree_sitter_parse(text) {
        Ok(tree) => tree,
        Err(tree) => {
            log::warn!("bad CST: {:?}", name.try_str());
            log::debug!("{}", tree.root_node().to_sexp());
            if PROPAGATE_ERROR_ON_BAD_CST_NODE {
                return Err(ParseErr::IllFormed);
            } else {
                tree
            }
        }
    };
    let x = tree_gen
        .generate_file(name.as_bytes(), text, tree.walk())
        .local;
    // TODO extract submodules, dependencies and directories. maybe even more ie. artefact id, ...
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
    pub metrics: DefaultMetrics,
    submodules: Vec<String>,
    source_dirs: Vec<String>,
    test_source_dirs: Vec<String>,
}

pub struct IterMavenModules2<'a> {
    stores: &'a SimpleStores,
    parents: Vec<NodeIdentifier>,
    offsets: Vec<u16>,
    /// to tell that we need to pop a parent, we could also use a bitvec instead of Option::None
    remaining: Vec<Option<NodeIdentifier>>,
}

impl<'a> Debug for IterMavenModules2<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterMavenModules")
            .field("parents", &self.parents())
            .field("offsets", &self.offsets())
            .field("remaining", &self.remaining)
            .finish()
    }
}

impl<'a> Iterator for IterMavenModules2<'a> {
    type Item = StructuralPosition;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node()
            .map(|x| (self.parents().to_vec(), self.offsets().to_vec(), x).into())
    }
}

impl<'a> IterMavenModules2<'a> {
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
    pub fn offsets(&self) -> &[u16] {
        &self.offsets[..self.offsets.len() - 1]
    }

    pub(crate) fn next_node(&mut self) -> Option<NodeIdentifier> {
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

        let b = self
            .stores
            .node_store
            .try_resolve_typed::<XmlIdN>(&x)
            .unwrap()
            .0;
        let t = b.get_type();

        let is_src = if b.has_label() {
            self.stores
                .label_store
                .resolve(b.get_label_unchecked())
                .eq("src")
        } else {
            false
        };

        if is_src {
            return self.next_node();
        } else if t != Type::MavenDirectory {
            return self.next_node();
        }

        self.parents.push(x);
        self.offsets.push(0);
        self.remaining.push(None);
        if b.has_children() {
            self.remaining
                .extend(b.children().unwrap().iter_children().rev().map(|x| Some(x)));
        }

        let contains_pom = b
            .children()
            .unwrap_or_default()
            .iter_children()
            .find(|x| {
                if let Some(n) = self.stores.node_store.try_resolve_typed::<XmlIdN>(x) {
                    let n = n.0;
                    log::debug!("f {:?}", n.get_type());
                    n.get_type().eq(&Type::Document)
                        && if n.has_label() {
                            log::debug!(
                                "f name: {:?}",
                                self.stores.label_store.resolve(n.get_label_unchecked())
                            );
                            self.stores
                                .label_store
                                .resolve(n.get_label_unchecked())
                                .eq("pom.xml")
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
                if let Some(x) = self.next_node() {
                    return Some(x);
                }
            }
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct MD {
    pub(crate) metrics: DefaultMetrics,
    #[allow(unused)] // TODO needed for scalable module level reference analysis
    pub(crate) ana: MavenPartialAnalysis,
    pub(crate) status: EnumSet<SemFlag>,
}

pub struct MavenModuleAcc {
    pub(crate) primary: BasicDirAcc<NodeIdentifier, LabelIdentifier, DefaultMetrics>,
    pub(crate) ana: MavenPartialAnalysis,
    pub(crate) sub_modules: Option<Vec<PathBuf>>,
    pub(crate) main_dirs: Option<Vec<PathBuf>>,
    pub(crate) test_dirs: Option<Vec<PathBuf>>,
    pub(crate) status: EnumSet<SemFlag>,
    pub(crate) scripting_acc: std::option::Option<hyperast::scripting::Acc>,
}

impl From<String> for MavenModuleAcc {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl MavenModuleAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
            ana: MavenPartialAnalysis::new(),
            sub_modules: None,
            main_dirs: None,
            test_dirs: None,
            status: Default::default(),
            scripting_acc: None,
        }
    }
    pub(crate) fn with_content(
        name: String,
        sub_modules: Vec<PathBuf>,
        main_dirs: Vec<PathBuf>,
        test_dirs: Vec<PathBuf>,
    ) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
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
            status: Default::default(),
            scripting_acc: None,
        }
    }
}

#[derive(enumset::EnumSetType, Debug)]
pub enum SemFlag {
    IsMavenModule,
    HoldMainFolder,
    HoldTestFolder,
    HoldMavenSubModule,
}

impl MavenModuleAcc {
    pub(crate) fn push_pom(&mut self, name: LabelIdentifier, full_node: POM) {
        self.status |= SemFlag::IsMavenModule;
        assert!(!self.primary.children_names.contains(&name));
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.main_dirs = Some(full_node.source_dirs.iter().map(|x| x.into()).collect());
        self.test_dirs = Some(
            full_node
                .test_source_dirs
                .iter()
                .map(|x| x.into())
                .collect(),
        );
        self.sub_modules = Some(full_node.submodules.iter().map(|x| x.into()).collect());
        self.primary.metrics.acc(full_node.metrics);
    }
    pub fn push_submodule(&mut self, name: LabelIdentifier, full_node: (NodeIdentifier, MD)) {
        if full_node.1.status.contains(SemFlag::HoldMavenSubModule)
            || full_node.1.status.contains(SemFlag::IsMavenModule)
        {
            self.status |= SemFlag::HoldMavenSubModule;
        }
        self.primary.children.push(full_node.0);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(full_node.1.metrics);
    }
    pub(crate) fn push_source_directory(
        &mut self,
        name: LabelIdentifier,
        full_node: java_tree_gen::Local,
    ) {
        self.status |= SemFlag::HoldMainFolder;
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(SubTreeMetrics {
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
        full_node: java_tree_gen::Local,
    ) {
        self.status |= SemFlag::HoldTestFolder;
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(SubTreeMetrics {
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

pub struct IterMavenModules<'a, T: TreePath<NodeIdentifier>> {
    stores: &'a SimpleStores,
    path: T,
    stack: Vec<(NodeIdentifier, u16, Option<Vec<NodeIdentifier>>)>,
}

impl<'a, T: TreePathMut<NodeIdentifier, u16> + Debug + Clone> Iterator for IterMavenModules<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, offset, children) = self.stack.pop()?;
            if let Some(children) = children {
                if offset.to_usize().unwrap() < children.len() {
                    let child = children[offset.to_usize().unwrap()];
                    self.path.check(self.stores).unwrap();
                    {
                        let b = self
                            .stores
                            .node_store
                            .try_resolve_typed::<XmlIdN>(&node)
                            .unwrap()
                            .0;
                        if b.has_children() {
                            let len = b.child_count();
                            let cs = b.children().unwrap();
                            // println!("children: {:?} {} {:?}", node,cs.len(),cs);
                            assert!(offset < len);
                            assert_eq!(child, cs[offset]);
                        } else {
                            panic!()
                        }
                    }
                    if offset == 0 {
                        match self.path.node() {
                            Some(x) => assert_eq!(*x, node),
                            None => {}
                        }
                        self.path.goto(child, offset);
                        self.path.check(self.stores).unwrap();
                    } else {
                        match self.path.node() {
                            Some(x) => assert_eq!(*x, children[offset.to_usize().unwrap() - 1]),
                            None => {}
                        }
                        self.path.inc(child);
                        assert_eq!(*self.path.offset().unwrap(), offset + 1);
                        self.path.check(self.stores).expect(&format!(
                            "{:?} {} {:?} {:?} {:?}",
                            node, offset, child, children, self.path
                        ));
                    }
                    self.stack.push((node, offset + 1, Some(children)));
                    self.stack.push((child, 0, None));
                    continue;
                } else {
                    self.path.check(self.stores).unwrap();
                    self.path.pop().expect("should not go higher than root");
                    self.path.check(self.stores).unwrap();
                    continue;
                }
            } else {
                let b = self
                    .stores
                    .node_store
                    .try_resolve_typed::<XmlIdN>(&node)
                    .unwrap()
                    .0;

                if self.is_dead_end(&b) {
                    continue;
                }

                if b.has_children() {
                    let children = b.children();
                    self.stack
                        .push((node, 0, Some(children.unwrap().iter_children().collect())));
                }

                if self.is_matching(&b) {
                    self.path.check(self.stores).unwrap();
                    return Some(self.path.clone());
                }
            }
        }
    }
}

type XmlIdN = hyperast_gen_ts_xml::types::TIdN<NodeIdentifier>;
type XmlNode<'a> = hyperast::store::nodes::legion::HashedNodeRef<'a, XmlIdN>;

impl<'a, T: TreePath<NodeIdentifier>> IterMavenModules<'a, T> {
    pub fn new(stores: &'a SimpleStores, path: T, root: NodeIdentifier) -> Self {
        let stack = vec![(root, 0, None)];
        Self {
            stores,
            path,
            stack,
        }
    }

    fn is_dead_end(&self, b: &XmlNode<'a>) -> bool {
        let t = b.get_type();
        let is_src = if b.has_label() {
            self.stores
                .label_store
                .resolve(b.get_label_unchecked())
                .eq("src")
        } else {
            false
        };

        is_src || t != Type::MavenDirectory
    }
    fn is_matching(&self, b: &XmlNode<'a>) -> bool {
        let contains_pom = b
            .children()
            .unwrap()
            .iter_children()
            .find(|x| {
                if let Some(n) = self.stores.node_store.try_resolve_typed::<XmlIdN>(x) {
                    let n = n.0;
                    log::debug!("f {:?}", n.get_type());
                    n.get_type().eq(&Type::Document)
                        && if n.has_label() {
                            log::debug!(
                                "f name: {:?}",
                                self.stores.label_store.resolve(n.get_label_unchecked())
                            );
                            self.stores
                                .label_store
                                .resolve(n.get_label_unchecked())
                                .eq("pom.xml")
                        } else {
                            false
                        }
                } else {
                    false
                }
            })
            .is_some();
        contains_pom
    }
}

impl hyperast::tree_gen::Accumulator for MavenModuleAcc {
    type Node = (LabelIdentifier, (NodeIdentifier, MD));
    fn push(&mut self, (name, full_node): Self::Node) {
        let s = full_node.1.status - SemFlag::IsMavenModule;
        assert!(!s.contains(SemFlag::IsMavenModule));
        self.status |= s;
        self.primary.children.push(full_node.0);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(full_node.1.metrics);
    }
}

impl Accumulator for MavenModuleAcc {
    type Unlabeled = (NodeIdentifier, MD);
}
