///! store nodes in Vec<Rc<Node>> without compression
use std::{collections::HashMap, fmt::Debug, hash::Hash, rc::Rc, vec, borrow::Borrow};

use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, Type, VersionedNodeStore, Stored, VersionedNodeStoreMut,
};
use rusted_gumtree_core::tree::tree::NodeStoreMut as NodeStoreMutTrait;
use string_interner::{DefaultSymbol, StringInterner};
use tree_sitter::{Language, Parser, TreeCursor};

use crate::{
    full::FullNode,
    hashed::{HashedCompressedNode, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{self, CompressedNode, HashSize, SimpleNode1, Space},
    store::TypeStore,
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, hash_for_node, label_for_cursor,
        AccIndentation, Accumulator, Spaces, TreeGen,
    },
    utils,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct HashedNode(
    HashedCompressedNode<SyntaxNodeHashs<HashSize>, Rc<HashedNode>, LabelIdentifier>,
);

type NodeIdentifier = Rc<HashedNode>;

impl rusted_gumtree_core::tree::tree::Typed for HashedNode {
    type Type = Type;

    fn get_type(&self) -> Type {
        self.0.get_type()
    }
}

impl rusted_gumtree_core::tree::tree::Labeled for HashedNode {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.0.get_label()
    }
}

impl rusted_gumtree_core::tree::tree::Node for HashedNode {}

impl rusted_gumtree_core::tree::tree::Stored for HashedNode {
    type TreeId = NodeIdentifier;
}
impl rusted_gumtree_core::tree::tree::WithChildren for HashedNode {
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        let tmp = &self.0;
        tmp.child_count()
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0.get_child(idx)
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0.get_child_rev(idx)
    }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        self.0.get_children()
    }
}

impl rusted_gumtree_core::tree::tree::WithHashs for HashedNode {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> HashSize {
        self.0.hashs.hash(kind)
    }
}

impl rusted_gumtree_core::tree::tree::Tree for HashedNode {
    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_label(&self) -> bool {
        self.0.has_label()
    }
}

impl HashedNode {
    pub(crate) fn new(
        hashs: SyntaxNodeHashs<HashSize>,
        node: CompressedNode<NodeIdentifier, LabelIdentifier>,
    ) -> Self {
        Self(HashedCompressedNode::new(hashs, node))
    }
}

// pub type HashedNode<'a> = HashedCompressedNode<SyntaxNodeHashs<HashSize>,SymbolU32<&'a HashedNode>,LabelIdentifier>;

extern "C" {
    fn tree_sitter_java() -> Language;
}

type MyLabel = str;
type LabelIdentifier = DefaultSymbol;

pub struct JavaTreeGen {
    pub line_break: Vec<u8>,
    pub stores: SimpleStores,
}

// type SpacesStoreD = SpacesStore<u16, 4>;

pub struct LabelStore {
    count: usize,
    internal: StringInterner, //VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl Debug for LabelStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LabelStore")
            .field("count", &self.count)
            .field("internal_len", &self.internal.len())
            .field("internal", &self.internal)
            .finish()
    }
}

impl LabelStoreTrait<MyLabel> for LabelStore {
    type I = LabelIdentifier;
    fn get_or_insert<T: Borrow<MyLabel>>(&mut self, node: T) -> Self::I {
        self.count += 1;
        self.internal.get_or_intern(node.borrow())
    }

    fn resolve(&self, id: &Self::I) -> &MyLabel {
        self.internal.resolve(*id).unwrap()
    }
}

pub struct NodeStore {
    count: usize,
    // internal: Vec<Rc<HashedNode>>,
    roots: HashMap<(u8, u8, u8), NodeIdentifier>,
}

impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeStore")
            .field("count", &self.count)
            // .field("internal_len", &self.internal.len())
            // .field("internal", &self.internal)
            .finish()
    }
}

impl<'a> NodeStoreTrait<'a, NodeIdentifier, NodeIdentifier> for NodeStore {
    fn resolve(&'a self, id: &NodeIdentifier) -> NodeIdentifier {
        id.clone()
    }
}

impl<'a> NodeStoreMutTrait<'a, HashedNode, NodeIdentifier> for NodeStore {
}

impl<'a> NodeStore {
    fn get_or_insert(&mut self, node: HashedNode) -> NodeIdentifier {
        self.count += 1;
        Rc::new(node)
    }
}

impl<'a> VersionedNodeStore<'a, NodeIdentifier, NodeIdentifier> for NodeStore {
    fn resolve_root(&self, version: (u8, u8, u8), node: <HashedNode as Stored>::TreeId) {
        todo!()
    }
}

impl<'a> VersionedNodeStoreMut<'a, HashedNode, NodeIdentifier> for NodeStore {
    fn as_root(&mut self, version: (u8, u8, u8), id: NodeIdentifier) {
        assert!(self.roots.insert(version, id).is_none());
    }

    fn insert_as_root(&mut self, version: (u8, u8, u8), node: HashedNode) -> <HashedNode as Stored>::TreeId {
        let r = self.get_or_insert(node);
        self.as_root(version, r.clone());
        r
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
    type Node1 = SimpleNode1<NodeIdentifier, String>;
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
            let label = label_for_cursor(text, &node)
                .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
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

            (HashedNode::new(hashs, node), metrics)
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
            let node = CompressedNode::<NodeIdentifier, _>::Spaces(relativized.into_boxed_slice());
            let spaces_leaf = HashedNode::new(hashs, node);
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
                node_store: NodeStore::new(),
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
    ) -> CompressedNode<NodeIdentifier, DefaultSymbol> {
        let label_id = match n1.label {
            Some(l) => Some(label_store.get_or_insert(l)),
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
                node_store: NodeStore::new(),
            },
        };
        let _full_node = java_tree_gen.generate_default(text, tree.walk());

        java_tree_gen
            .stores
            .node_store
            .as_root((0, 1, 0), _full_node.local.compressed_node.clone());

        // print_tree_structure(
        //     &java_tree_gen.stores.node_store,
        //     &_full_node.local.compressed_node,
        // );

        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate_default(text, tree.walk());
    }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    nodes::print_tree_structure(
        |id| -> _ {
            node_store.resolve(id).0.node.clone()
         },
        id,
    )
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store.resolve(id).0.node.clone()
         },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store.resolve(id).0.node.clone()
         },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn serialize<W: std::fmt::Write>(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    nodes::serialize(
        |id| -> _ {
            node_store.resolve(id).0.node.clone()
         },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
        out,
        parent_indent,
    )
}

impl NodeStore {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            roots: Default::default(),
            // internal: VecMapStore::new(),
        }
    }
}

impl LabelStore {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            internal: Default::default(),
        }
    }
}
