///! fully compress all subtrees
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    vec, rc::Rc, borrow::Borrow,
};

use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, Type, Typed, VersionedNodeStore, VersionedNodeStoreMut, Stored,
};
use string_interner::{DefaultSymbol, StringInterner};
use tree_sitter::{Language, Parser, TreeCursor};

use crate::{
    filter::{Bloom, BloomResult, BF},
    full::FullNode,
    hashed::{HashedCompressedNode, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{CompressedNode, HashSize, SimpleNode1, Space, RefContainer, self},
    store::{
        vec_map_store::{SymbolU32, VecMapStore, AsNodeEntityRef, AsNodeEntityRefSelf},
        TypeStore,
    },
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, hash_for_node, label_for_cursor,
        AccIndentation, Accumulator, BasicAccumulator, Spaces, TreeGen,
    },
    utils,
};
use rusted_gumtree_core::tree::tree::NodeStoreMut as NodeStoreMutTrait;

pub struct HashedSubtreePlus {
    refs: Bloom<&'static [u8], u16>,
    content: HashedCompressedNode<
        SyntaxNodeHashs<HashSize>,
        SymbolU32<HashedSubtreePlus>,
        LabelIdentifier,
    >,
}

impl RefContainer for HashedSubtreePlus {
    type Ref = [u8];
    type Result = crate::filter::BloomResult;

    fn check<U:Borrow<Self::Ref>+AsRef<[u8]>>(&self, rf: U) -> Self::Result {
        self.refs.check(0, rf)
    }
}

impl Hash for HashedSubtreePlus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.content.hash(state);
    }
}

impl PartialEq for HashedSubtreePlus {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl Eq for HashedSubtreePlus {}

impl Debug for HashedSubtreePlus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedNodePlus")
            .field("content", &self.content)
            .finish()
    }
}

impl AsRef<HashedSubtreePlus> for HashedSubtreePlus {
    fn as_ref(&self) -> &HashedSubtreePlus {
        self
    }
}

type NodeIdentifier = SymbolU32<HashedSubtreePlus>;

impl rusted_gumtree_core::tree::tree::Typed for HashedSubtreePlus {
    type Type = Type;

    fn get_type(&self) -> Type {
        self.content.get_type()
    }
}

impl rusted_gumtree_core::tree::tree::Labeled for HashedSubtreePlus {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.content.get_label()
    }
}

impl rusted_gumtree_core::tree::tree::Node for HashedSubtreePlus {}

impl rusted_gumtree_core::tree::tree::Stored for HashedSubtreePlus {
    type TreeId = NodeIdentifier;
}
impl rusted_gumtree_core::tree::tree::WithChildren for HashedSubtreePlus {
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.content.child_count()
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.content.get_child(idx)
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.content.get_child_rev(idx)
    }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        self.content.get_children()
    }
}

impl rusted_gumtree_core::tree::tree::WithHashs for HashedSubtreePlus {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> HashSize {
        self.content.hashs.hash(kind)
    }
}

impl rusted_gumtree_core::tree::tree::Tree for HashedSubtreePlus {
    fn has_children(&self) -> bool {
        self.content.has_children()
    }

    fn has_label(&self) -> bool {
        self.content.has_label()
    }
}

impl HashedSubtreePlus {
    pub(crate) fn new(
        hashs: SyntaxNodeHashs<HashSize>,
        node: CompressedNode<NodeIdentifier, LabelIdentifier>,
    ) -> Self {
        Self::new_with_refs(hashs, node, Default::default())
    }
    pub(crate) fn new_with_refs(
        hashs: SyntaxNodeHashs<HashSize>,
        node: CompressedNode<NodeIdentifier, LabelIdentifier>,
        refs:Bloom<&'static[u8], u16>,
    ) -> Self {
        Self {
            refs,
            content: HashedCompressedNode::new(hashs, node),
        }
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

impl AsNodeEntityRef for HashedSubtreePlus {
    type Ref<'a> = &'a HashedSubtreePlus;

    fn eq(&self, other: &Self::Ref<'_>) -> bool {
        PartialEq::eq(&self, other)
    }
}

impl AsNodeEntityRefSelf for HashedSubtreePlus {
    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
}

pub struct NodeStore {
    count: usize,
    errors: usize,
    roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    internal: VecMapStore<HashedSubtreePlus, NodeIdentifier>,
}

impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeStore")
            .field("count", &self.count)
            .field("errors", &self.errors)
            .field("internal_len", &self.internal.len())
            // .field("internal", &self.internal)
            .finish()
    }
}

impl<'a> NodeStoreTrait<'a, NodeIdentifier,&'a HashedSubtreePlus> for NodeStore {
    fn resolve(&'a self, id: &NodeIdentifier) -> &'a HashedSubtreePlus {
        self.internal.resolve(id)
    }
}
impl<'a> NodeStoreMutTrait<'a, HashedSubtreePlus,&'a HashedSubtreePlus> for NodeStore {
}

impl<'a> NodeStore {
    fn get_or_insert(&mut self, node: HashedSubtreePlus) -> NodeIdentifier {
        self.count += 1;
        if node.get_type() == Type::Error
            && self.internal.get::<&HashedSubtreePlus>(&node).is_none()
        {
            self.errors += 1;
            println!("{:?}", &node);
        }
        self.internal.get_or_intern(node)
    }
}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl<'a> VersionedNodeStore<'a, NodeIdentifier,&'a HashedSubtreePlus> for NodeStore {
    fn resolve_root(&self, version: (u8, u8, u8), node: <HashedSubtreePlus as Stored>::TreeId) {
        todo!()
    }
}

impl<'a> VersionedNodeStoreMut<'a, HashedSubtreePlus,&'a HashedSubtreePlus> for NodeStore {
    fn as_root(&mut self, version: (u8, u8, u8), id: NodeIdentifier) {
        assert!(self.roots.insert(version, id).is_none());
    }

    fn insert_as_root(&mut self, version: (u8, u8, u8), node: HashedSubtreePlus) -> <HashedSubtreePlus as Stored>::TreeId {
        let r = self.get_or_insert(node);
        self.as_root(version, r);
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
    pub(crate) refs: HashSet<String>,
}

impl Local {
    fn acc(self, acc: &mut Acc) {
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);
        acc.refs.extend(self.refs);
    }
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

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    refs: HashSet<String>,
    padding_start: usize,
    indentation: Spaces,
}

impl Acc {
    pub(crate) fn new(kind: Type) -> Self {
        Self::new_with_indent(kind, Space::format_indentation(&"\n".as_bytes().to_vec()))
    }
    pub(crate) fn new_with_indent(kind: Type, indentation: Spaces) -> Self {
        Self {
            simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            refs: Default::default(),
            padding_start: 0,
            indentation,
        }
    }
}

impl Accumulator for Acc {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
        // self.simple.push(full_node.local.compressed_node);
        // self.metrics.acc(full_node.local.metrics);
    }
}

impl AccIndentation for Acc {
    fn indentation<'a>(&'a self) -> &'a Spaces {
        &self.indentation
    }
}

pub struct SimpleStores {
    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
}

impl Default for SimpleStores {
    fn default() -> Self {
        Self {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        }
    }
}

impl TreeGen for JavaTreeGen {
    type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Acc = Acc;
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

        Acc {
            simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            refs: Default::default(),
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
            let metrics = acc.metrics;
            let acc = acc.simple;
            let node = SimpleNode1 {
                kind: acc.kind,
                label,
                children: acc.children,
            };
            (node, metrics)
        };
        let mut refs = acc.refs;

        refs.insert("B".to_owned());
        let (compressible_node, metrics) = {
            let hashs = hash_for_node(&metrics.hashs, &metrics.size, &node);

            let metrics = SubTreeMetrics {
                size: metrics.size + 1,
                height: metrics.height + 1,
                hashs,
            };

            let node = Self::compress_label(label_store, node);

            let mut res = Bloom::default();
            for r in &refs {
                res.insert(0, r.as_bytes());
            }
            (HashedSubtreePlus::new_with_refs(hashs, node,res), metrics)
        };


        let compressed_node = node_store.get_or_insert(compressible_node);
        let full_node = FullNode {
            global: Global { depth, position },
            local: Local {
                compressed_node,
                metrics,
                refs: refs,
            },
        };
        full_node
    }

    fn init_val(&mut self, text: &[u8], node: &tree_sitter::Node) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get(node.kind());

        let indentation = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &Space::format_indentation(&self.line_break),
        );
        Acc::new_with_indent(kind, indentation)
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
            let spaces_leaf = HashedSubtreePlus::new(hashs, node);
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
                    refs: Default::default(),
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
            acc.metrics.size as usize + 1,
            &mut acc,
        );
        let mut r = Acc::new(self.stores().type_store.get("file"));

        let full_node = self.post(
            &mut r,
            0,
            acc.metrics.size as usize,
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
            let source_code1 = "class A {void test() { B.c; }}";
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

        // print_tree_structure(
        //     &java_tree_gen.stores.node_store,
        //     &_full_node.local.compressed_node,
        // );

        {// playing with refs
            let a = &_full_node.local.compressed_node;

            let b = java_tree_gen.stores.node_store.resolve(a);

            let d = _full_node.local.refs.iter().next().unwrap();

            let c = b.check(_full_node.local.refs.iter().next().unwrap().as_bytes());

            match c {
                BloomResult::MaybeContain => println!("Maybe contains {}", d),
                BloomResult::DoNotContain => println!("Do not contains {}", d),
            }
        }
        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate_default(text, tree.walk());
    }
}


pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    nodes::print_tree_structure(
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            // node_store.resolve(id).content.node;
            todo!()
        },
        id,
    )
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            // node_store.resolve(id).content.node;
            todo!()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            &node_store.resolve(id).content.node
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
            &node_store.resolve(id).content.node
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
            errors: 0,
            roots: Default::default(),
            internal: VecMapStore::new(),
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
