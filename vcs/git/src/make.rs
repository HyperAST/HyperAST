use std::{fmt::Debug, path::PathBuf};

use hyperast::{
    store::defaults::{LabelIdentifier, NodeIdentifier},
    tree_gen::SubTreeMetrics,
};
use hyperast_gen_ts_cpp::legion as cpp_tree_gen;
use hyperast_gen_ts_xml::{legion::XmlTreeGen, types::TStore};

use crate::{
    Accumulator, BasicDirAcc, DefaultMetrics, PROPAGATE_ERROR_ON_BAD_CST_NODE,
    processing::ObjectName,
};

pub(crate) fn handle_makefile_file<'a>(
    tree_gen: &mut XmlTreeGen<'a, TStore>,
    name: &ObjectName,
    text: &'a [u8],
) -> Result<MakeFile, ()> {
    log::trace!("not parsing {} bytes long Makefile", text.len()); // TODO parse the makefile
    let text = b"<proj></proj>";
    let tree = match hyperast_gen_ts_xml::legion::tree_sitter_parse_xml(text) {
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
        .generate_file(name.as_bytes(), text, tree.walk())
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
}

pub struct MakeModuleAcc {
    pub(crate) primary: BasicDirAcc<NodeIdentifier, LabelIdentifier, DefaultMetrics>,
    pub(crate) sub_modules: Option<Vec<PathBuf>>,
    pub(crate) main_dirs: Option<Vec<PathBuf>>,
    pub(crate) test_dirs: Option<Vec<PathBuf>>,
}

impl From<String> for MakeModuleAcc {
    fn from(name: String) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
            sub_modules: None,
            main_dirs: None,
            test_dirs: None,
        }
    }
}

impl MakeModuleAcc {
    pub(crate) fn new(name: String) -> Self {
        Self {
            primary: BasicDirAcc::new(name),
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
            primary: BasicDirAcc::new(name),
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
        self.primary.children.push(full_node.0);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(full_node.1.metrics);
    }
    pub(crate) fn push_source_file(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
    ) {
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(SubTreeMetrics {
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
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
            size_no_spaces: full_node.metrics.size_no_spaces,
            line_count: 0,
        });
    }
    pub(crate) fn push_test_source_directory(
        &mut self,
        name: LabelIdentifier,
        full_node: cpp_tree_gen::Local,
    ) {
        self.primary.children.push(full_node.compressed_node);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(SubTreeMetrics {
            hashs: full_node.metrics.hashs,
            size: full_node.metrics.size,
            height: full_node.metrics.height,
            size_no_spaces: full_node.metrics.size_no_spaces,
            line_count: 0,
        });
    }
}

impl hyperast::tree_gen::Accumulator for MakeModuleAcc {
    type Node = (LabelIdentifier, (NodeIdentifier, MD));
    fn push(&mut self, (name, full_node): Self::Node) {
        self.primary.children.push(full_node.0);
        self.primary.children_names.push(name);
        self.primary.metrics.acc(full_node.1.metrics);
    }
}

impl Accumulator for MakeModuleAcc {
    type Unlabeled = (NodeIdentifier, MD);
}
