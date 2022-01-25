///! fully compress all subtrees
use std::{
    cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData,
    num::NonZeroU64, ops::Deref, vec, borrow::Borrow,
};

use legion::{
    storage::{Archetype, Component},
    world::{ComponentError, EntityAccessError, EntityLocation, EntryRef},
    EntityStore,
};
use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, Type, Typed, VersionedNodeStore, Stored,
};
use string_interner::{DefaultSymbol, StringInterner};
use tree_sitter::{Language, Parser, TreeCursor};
use rusted_gumtree_core::tree::tree::NodeStoreMut as NodeStoreMutTrait;

use crate::{
    full::FullNode,
    hashed::{HashedCompressedNode, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{CompressedNode, HashSize, SimpleNode1, Space, self},
    store::{
        vec_map_store::{AsNodeEntityRef, Backend, Symbol, SymbolU32, VecMapStore},
        TypeStore,
    },
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, hash_for_node, label_for_cursor,
        AccIndentation, Accumulator, Spaces, TreeGen,
    },
    utils,
};

// #[derive(PartialEq, Eq, Hash, Debug)]
// pub struct HashedNodeOld(
//     HashedCompressedNode<SyntaxNodeHashs<HashSize>, legion::Entity, LabelIdentifier>,
// );

type NodeIdentifier = legion::Entity;

struct HashedNodeRef<'a>(EntryRef<'a>);

struct HashedNode {
    node: CompressedNode<legion::Entity, LabelIdentifier>,
    hashs: SyntaxNodeHashs<u32>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

impl HashedNode {
    fn new(
        node: CompressedNode<legion::Entity, LabelIdentifier>,
        hashs: SyntaxNodeHashs<u32>,
        metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ) -> Self {
        Self {
            node,
            hashs,
            metrics,
        }
    }
}

impl<'a> PartialEq for HashedNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'a> Eq for HashedNode {}

impl<'a> Hash for HashedNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state)
    }
}

impl rusted_gumtree_core::tree::tree::Node for HashedNode {}
impl rusted_gumtree_core::tree::tree::Stored for HashedNode {
    type TreeId = NodeIdentifier;
}

impl Symbol<HashedNode> for legion::Entity {
}

impl<'a> Symbol<HashedNodeRef<'a>> for legion::Entity {
}

// impl Deref for HashedNodeDeref {
//     type Target = HashedNodeRef<'a>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl<'a> PartialEq for HashedNodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.location().archetype() == other.0.location().archetype()
            && self.0.location().component() == other.0.location().component()
    }
}

impl<'a> Eq for HashedNodeRef<'a> {}

impl<'a> Hash for HashedNodeRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        rusted_gumtree_core::tree::tree::WithHashs::hash(self, &Default::default()).hash(state)
    }
}

impl<'a> Debug for HashedNodeRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HashedNodeRef")
            .field(&self.0.location())
            .finish()
    }
}

impl<'a> Deref for HashedNodeRef<'a> {
    type Target = EntryRef<'a>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<'a> HashedNodeRef<'a> {
    pub(crate) fn new(entry: EntryRef<'a>) -> Self {
        Self(entry)
    }

    /// Returns the entity's archetype.
    pub fn archetype(&self) -> &Archetype {
        self.0.archetype()
    }

    /// Returns the entity's location.
    pub fn location(&self) -> EntityLocation {
        self.0.location()
    }

    /// Returns a reference to one of the entity's components.
    pub fn into_component<T: Component>(self) -> Result<&'a T, ComponentError> {
        self.0.into_component()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn into_component_unchecked<T: Component>(
        self,
    ) -> Result<&'a mut T, ComponentError> {
        self.0.into_component_unchecked()
    }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<T: Component>(&self) -> Result<&T, ComponentError> {
        self.0.get_component()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn get_component_unchecked<T: Component>(&self) -> Result<&mut T, ComponentError> {
        self.0.get_component_unchecked()
    }

    fn into_compressed_node(
        &self,
    ) -> Result<&CompressedNode<legion::Entity, LabelIdentifier>, ComponentError> {
        Ok(&self.0
        .get_component::<HashedNode>()
        .unwrap()
        .node)
    }
}

impl<'a> AsRef<HashedNodeRef<'a>> for HashedNodeRef<'a> {
    fn as_ref(&self) -> &HashedNodeRef<'a> {
        self
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Typed for HashedNodeRef<'a> {
    type Type = Type;

    fn get_type(&self) -> Type {
        *self.0.get_component::<Type>().unwrap()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Labeled for HashedNodeRef<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.0.get_component::<LabelIdentifier>().unwrap()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Node for HashedNodeRef<'a> {}

impl<'a> rusted_gumtree_core::tree::tree::Stored for HashedNodeRef<'a> {
    type TreeId = NodeIdentifier;
}
impl<'a> rusted_gumtree_core::tree::tree::WithChildren for HashedNodeRef<'a> {
    type ChildIdx = u8;

    fn child_count(&self) -> u8 {
        self.0.get_component::<Vec<legion::Entity>>().unwrap().len() as u8
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0.get_component::<Vec<legion::Entity>>().unwrap()[num::cast::<_, usize>(*idx).unwrap()]
    }
    
    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        let v = self.0.get_component::<Vec<legion::Entity>>().unwrap();
        v[v.len()-1-num::cast::<_, usize>(*idx).unwrap()]
    }

    fn get_children<'b>(&'b self) -> &'b [Self::TreeId] {
        self.0
            .get_component::<Vec<legion::Entity>>()
            .unwrap()
            .as_slice()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::WithHashs for HashedNodeRef<'a> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.0
            .get_component::<HashedNode>()
            .unwrap()
            .hashs
            .hash(kind)
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Tree for HashedNodeRef<'a> {
    fn has_children(&self) -> bool {
        self.0
            .get_component::<Vec<legion::Entity>>()
            .map(|x| !x.is_empty())
            .unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get_component::<LabelIdentifier>().is_ok()
    }
}

impl<'a> HashedNodeRef<'a> {
    // pub(crate) fn new(
    //     hashs: SyntaxNodeHashs<HashSize>,
    //     node: CompressedNode<NodeIdentifier, LabelIdentifier>,
    // ) -> Self {
    //     Self(HashedCompressedNode::new(hashs, node))
    // }
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

impl AsNodeEntityRef for HashedNode {
    type Ref<'a> = HashedNodeRef<'a>;

    fn eq(&self, other: &Self::Ref<'_>) -> bool {
        PartialEq::eq(self, other.get_component::<HashedNode>().unwrap())
    }
}

impl Backend<HashedNode> for legion::World {
    type Symbol = NodeIdentifier;

    fn len(&self) -> usize {
        legion::World::len(self)
    }

    fn with_capacity(cap: usize) -> Self {
        legion::World::new(legion::WorldOptions::default())
    }

    fn intern(&mut self, thing: HashedNode) -> Self::Symbol {
        legion::World::push(self, (thing,))
    }

    fn shrink_to_fit(&mut self) {}

    fn resolve(&self, symbol: Self::Symbol) -> Option<<HashedNode as AsNodeEntityRef>::Ref<'_>> {
        let entity = symbol;
        let er = legion::EntityStore::entry_ref(self, entity).ok();
        er.map(|x| HashedNodeRef(x))
    }

    unsafe fn resolve_unchecked(
        &self,
        symbol: Self::Symbol,
    ) -> <HashedNode as AsNodeEntityRef>::Ref<'_> {
        let entity = symbol;
        let er = legion::EntityStore::entry_ref(self, entity).unwrap();
        HashedNodeRef(er)
    }
}

pub struct NodeStore {
    count: usize,
    errors: usize,
    roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    // internal: legion::World,
    internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
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

impl<'a> NodeStoreTrait<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {

    fn resolve(&'a self, id: &NodeIdentifier) -> HashedNodeRef<'a> {
        self.internal.resolve(id)
    }
}

impl<'a> NodeStoreMutTrait<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {

}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
    fn get_or_insert(&mut self, node: HashedNode) -> NodeIdentifier {
        self.count += 1;
        if node.node.get_type() == Type::Error {
            self.errors += 1;
            println!("{:?}", &node.node);
        }
        self.internal.get_or_intern(node)
    }
}

impl<'a> VersionedNodeStore<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
    fn resolve_root(&self, version: (u8, u8, u8), node: <HashedNode as Stored>::TreeId) {
        todo!()
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

#[derive(Default, Debug, Clone, Copy)]
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

pub struct AccumulatorWithIndentation {
    simple: BasicAccumulator,
    padding_start: usize,
    indentation: Spaces,
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

impl Default for SimpleStores {
    fn default() -> Self {
        Self {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        }
    }
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

            (HashedNode::new(node, hashs, metrics), metrics)
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

            let (compressible_node, metrics) = {
                let metrics = SubTreeMetrics {
                    size: 1,
                    height: 1,
                    hashs,
                };

                (HashedNode::new(node, hashs, metrics), metrics)
            };

            let compressed_node = node_store.get_or_insert(compressible_node);
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
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            // let r = node_store.resolve(id).into_compressed_node().unwrap().clone();
            
            // let r = r.to_owned();
            todo!()
        },
        id,
    )
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            node_store.resolve(id).into_compressed_node().unwrap().clone();
            todo!()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn print_tree_syntax<'a>(node_store: &'a NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            node_store.resolve(id).into_compressed_node().unwrap().clone();
            todo!()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn serialize<'a, W: std::fmt::Write>(
    node_store: &'a NodeStore,
    label_store: &'a LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    nodes::serialize(
        |id| -> CompressedNode<NodeIdentifier, LabelIdentifier> {
            node_store.resolve(id).into_compressed_node().unwrap().clone();
            todo!()
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
            internal: Default::default(),
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
