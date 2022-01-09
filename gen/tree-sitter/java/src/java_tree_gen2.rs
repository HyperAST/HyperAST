///! second attempt at compressing subtrees
///! does not compressed due to first attempt of vec_map_store unable to correctly compare nodes
use std::{cell::Ref, fmt::Debug, vec, borrow::Borrow};

use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, NodeStoreMut as NodeStoreMutTrait, OwnedLabel, Type,
};
use tree_sitter::{Language, Parser, TreeCursor};

use crate::{
    full::FullNode,
    hashed::{HashedCompressedNode, HashedNode, NodeHashs, SyntaxNodeHashs},
    nodes::{CompressedNode, LabelIdentifier, NodeIdentifier, SimpleNode1, Space},
    store::TypeStore,
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, hash_for_node, label_for_cursor, Accumulator,
        AccIndentation, Spaces, TreeGen,
    },
    utils,
    vec_map_store::VecMapStore,
};

extern "C" {
    fn tree_sitter_java() -> Language;
}

pub struct JavaTreeGen {
    pub line_break: Vec<u8>,
    pub stores: SimpleStores,
}

/// DONE replaced with string-interner in second generator
#[derive(Debug)]
pub struct LabelStore {
    count: usize,
    // internal: VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl LabelStoreTrait<OwnedLabel> for LabelStore {
    type I = LabelIdentifier;

    fn get_or_insert<T: Borrow<OwnedLabel>>(&mut self, _node: T) -> Self::I {
        self.count += 1;
        // self.internal.get_or_insert(node)
        todo!()
    }

    fn resolve(&self, _id: &Self::I) -> &OwnedLabel {
        // self.internal.resolve(id)
        todo!()
    }
}

#[derive(Debug)]
pub struct NodeStore {
    count: usize,
    internal: VecMapStore<HashedNode, NodeIdentifier>,
}

impl<'a> NodeStoreTrait<'a, NodeIdentifier,Ref<'a, HashedNode>> for NodeStore {

    fn resolve(&'a self, id: &NodeIdentifier) -> Ref<'a, HashedNode> {
        self.internal.resolve(id)
    }
}
impl<'a> NodeStoreMutTrait<'a, HashedNode,Ref<'a, HashedNode>> for NodeStore {
}
impl<'a> NodeStore {
    fn get_or_insert(&mut self, node: HashedNode) -> NodeIdentifier {
        self.internal.get_or_insert(node)
    }
}

#[derive(Debug)]
pub struct Global {
    pub(crate) depth: usize,
    pub(crate) position: usize,
}

#[derive(Debug)]
pub struct Local {
    pub(crate) compressed_node: NodeIdentifier,
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

#[derive(Default, Debug)]
pub struct SubTreeMetrics<U: NodeHashs> {
    pub(crate) hashs: U,
    pub(crate) size: u32,
    pub(crate) height: u32,
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.hashs.acc(&other.hashs);
    }
}

pub struct BasicAccumulator {
    kind: Type,
    children: Vec<NodeIdentifier>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

pub struct AccumulatorWithIndentation {
    simple: BasicAccumulator,
    padding_start: usize,
    indentation: Spaces,
}

impl BasicAccumulator {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            kind,
            children: vec![],
            metrics: Default::default(),
        }
    }
}

impl Accumulator for BasicAccumulator {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        self.children.push(full_node.local.compressed_node);
        self.metrics.acc(full_node.local.metrics)
    }
}

impl AccumulatorWithIndentation {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            simple: BasicAccumulator::new(kind),
            padding_start: 0,
            indentation: Space::format_indentation(&"\n".as_bytes().to_vec()),
        }
    }
}

impl Accumulator for AccumulatorWithIndentation {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        self.simple.push(full_node);
    }
}

impl AccIndentation for AccumulatorWithIndentation {
    fn indentation<'a>(&'a self) -> &'a Spaces {
        &self.indentation
    }
}

pub struct SimpleStores {
    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
}

impl<'a> TreeGen for JavaTreeGen {
    type Node1 = SimpleNode1<NodeIdentifier, OwnedLabel>;
    type Acc = AccumulatorWithIndentation;
    type Stores = SimpleStores;
    type Text = [u8];

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn pre(
        &mut self,
        text: &[u8],
        node: &tree_sitter::Node,
        stack: &Vec<Self::Acc>,
        sum_byte_length: usize,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = type_store.get(node.kind());

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            sum_byte_length,
            &parent_indentation,
        );
        AccumulatorWithIndentation {
            simple: BasicAccumulator {
                kind,
                children: vec![],
                metrics: Default::default(),
            },
            padding_start: sum_byte_length,
            indentation: indent,
        }
    }
    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        depth: usize,
        position: usize,
        text: &[u8],
        node: &tree_sitter::Node,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;

        println!("{}", node.kind());

        Self::handle_spacing(
            acc.padding_start,
            node.start_byte(),
            text,
            node_store,
            &(depth + 1),
            position,
            parent,
        );

        let (node, metrics) = {
            let label = label_for_cursor(text, &node);
            let acc = acc.simple;
            let node = SimpleNode1 {
                kind: acc.kind,
                label,
                children: acc.children,
            };
            let metrics = acc.metrics;
            (node, metrics)
        };
        let (compressible_node, metrics) = {
            let hashs = hash_for_node(&metrics.hashs, &metrics.size, &node);

            let metrics = SubTreeMetrics {
                size: metrics.size + 1,
                height: metrics.height + 1,
                hashs,
            };

            let node = Self::compress_label(label_store, node);

            (HashedCompressedNode::new(hashs, node), metrics)
        };

        let compressed_node = node_store.get_or_insert(compressible_node);
        let full_node = FullNode {
            global: Global { depth, position },
            local: Local {
                compressed_node,
                metrics,
            },
        };
        full_node
    }

    fn init_val(&mut self, text: &[u8], node: &tree_sitter::Node) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get(node.kind());

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &Space::format_indentation(&self.line_break),
        );
        AccumulatorWithIndentation {
            simple: BasicAccumulator {
                kind,
                children: vec![],
                metrics: Default::default(),
            },
            padding_start: 0,
            indentation: indent,
        }
    }
}

impl JavaTreeGen {
    fn handle_spacing(
        padding_start: usize,
        pos: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        depth: &usize,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        let tmp = get_spacing(padding_start, pos, text, parent.indentation());
        if let Some(relativized) = tmp {
            let hashs = SyntaxNodeHashs {
                structt: 0,
                label: 0,
                syntax: utils::clamp_u64_to_u32(&utils::hash(&relativized)),
            };
            let node = CompressedNode::Spaces(relativized.into_boxed_slice());
            let spaces_leaf = HashedCompressedNode::new(hashs, node);
            let compressed_node = node_store.get_or_insert(spaces_leaf);
            let full_spaces_node = FullNode {
                global: Global {
                    depth: *depth,
                    position,
                },
                local: Local {
                    compressed_node,
                    metrics: SubTreeMetrics {
                        size: 1,
                        height: 1,
                        hashs,
                    },
                },
            };
            parent.push(full_spaces_node);
        };
    }

    /// end of tree but not end of file,
    /// thus to be a bijection, we need to get the last spaces
    fn handle_final_space(
        depth: &usize,
        sum_byte_length: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        if has_final_space(depth, sum_byte_length, text) {
            Self::handle_spacing(
                sum_byte_length,
                text.len(),
                text,
                node_store,
                depth,
                position,
                parent,
            )
        }
    }

    pub fn new() -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(HashedCompressedNode::new(
                    SyntaxNodeHashs::default(),
                    CompressedNode::Spaces(vec![].into_boxed_slice()),
                )),
            },
        }
    }

    pub fn generate_default(
        &mut self,
        text: &[u8],
        mut cursor: TreeCursor,
    ) -> FullNode<Global, Local> {
        // self.generate(text, cursor)
        let mut stack = vec![];
        stack.push(self.init_val(text, &cursor.node()));
        cursor.goto_first_child();
        let sum_byte_length = self.gen(text, &mut stack, &mut cursor);

        let mut acc = stack.pop().unwrap();

        Self::handle_final_space(
            &0,
            sum_byte_length,
            text,
            &mut self.stores.node_store,
            acc.simple.metrics.size as usize + 1,
            &mut acc,
        );
        let mut r = AccumulatorWithIndentation::new(self.stores().type_store.get("file"));

        let full_node = self.post(
            &mut r,
            0,
            acc.simple.metrics.size as usize,
            text,
            &cursor.node(),
            acc,
        );
        full_node
    }

    fn compress_label(
        label_store: &mut LabelStore,
        n1: <Self as TreeGen>::Node1,
    ) -> CompressedNode<NodeIdentifier, LabelIdentifier> {
        let label_id = match n1.label {
            Some(l) => Some(label_store.get_or_insert(&l)),
            None => None,
        };
        CompressedNode::new(n1.kind, label_id, n1.children)
    }

    pub fn main() {
        let mut parser = Parser::new();
        parser.set_language(unsafe { tree_sitter_java() }).unwrap();

        let text = {
            let source_code1 = "class A {void test() {}}";
            source_code1.as_bytes()
        };
        // let mut parser: Parser, old_tree: Option<&Tree>
        let tree = parser.parse(text, None).unwrap();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(HashedCompressedNode::new(
                    SyntaxNodeHashs {
                        structt: 0,
                        label: 0,
                        syntax: 0,
                    },
                    CompressedNode::Spaces(vec![].into_boxed_slice()),
                )),
            },
        };
        let _full_node = java_tree_gen.generate_default(text, tree.walk());

        print_tree_structure(
            &java_tree_gen.stores.node_store,
            &_full_node.local.compressed_node,
        );

        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate_default(text, tree.walk());
    }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label: _ } => {
            print!("({})", kind.to_string());
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_structure(node_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_structure(node_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.resolve(label);
            if s.len() > 20 {
                print!(
                    "({}='{}...')",
                    kind.to_string(),
                    std::str::from_utf8(&s[..20]).unwrap()
                );
            } else {
                print!(
                    "({}='{}')",
                    kind.to_string(),
                    std::str::from_utf8(s).unwrap()
                );
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_labels(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_labels(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.resolve(label);
            if s.len() > 20 {
                print!(
                    "({}='{}...')",
                    kind.to_string(),
                    std::str::from_utf8(&s[..20]).unwrap()
                );
            } else {
                print!(
                    "({}='{}')",
                    kind.to_string(),
                    std::str::from_utf8(s).unwrap()
                );
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_syntax(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_syntax(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(s) => {
            print!("(_ ");
            let a = &**s;
            a.iter().for_each(|a| print!("{:?}", a));
            print!(")");
        }
    };
}

pub fn serialize<W: std::fmt::Write>(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    let node = node_store.resolve(id);
    match &node.node {
        CompressedNode::Type(kind) => {
            out.write_str(&kind.to_string()).unwrap();
            // out.write_fmt(format_args!("{}",kind.to_string())).unwrap();
            None
        }
        CompressedNode::Label { kind: _, label } => {
            let s = &label_store.resolve(label);
            out.write_str(&std::str::from_utf8(s).unwrap()).unwrap();
            // write!(&mut out, "{}", std::str::from_utf8(s).unwrap()).unwrap();
            None
        }
        CompressedNode::Children2 { kind: _, children } => {
            let ind = serialize(node_store, label_store, &children[0], out, parent_indent)
                .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            serialize(node_store, label_store, &children[1], out, &ind);
            None
        }
        CompressedNode::Children { kind: _, children } => {
            let children = &(**children);
            // writeln!(out, "{:?}", children).unwrap();
            // writeln!(out, "{:?}", kind).unwrap();
            let mut it = children.iter();
            let mut ind = serialize(
                node_store,
                label_store,
                &it.next().unwrap(),
                out,
                parent_indent,
            )
            .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            for id in it {
                ind = serialize(node_store, label_store, &id, out, &ind)
                    .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            }
            None
        }
        CompressedNode::Spaces(s) => {
            let a = &**s;
            let mut b = String::new();
            // let mut b = format!("{:#?}", a);
            // fmt::format(args)
            a.iter()
                .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
            // std::io::Write::write_all(out, "<|".as_bytes()).unwrap();
            // std::io::Write::write_all(out, parent_indent.replace("\n", "n").as_bytes()).unwrap();
            // std::io::Write::write_all(out, "|>".as_bytes()).unwrap();
            out.write_str(&b).unwrap();
            Some(if b.contains("\n") {
                b
            } else {
                parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
            })
        }
    }
}

impl NodeStore {
    pub(crate) fn new(filling_element: HashedNode) -> Self {
        Self {
            count: 0,
            internal: VecMapStore::new(filling_element),
        }
    }
}

impl LabelStore {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            // internal: VecMapStore::new(vec![]),
        }
    }
}
