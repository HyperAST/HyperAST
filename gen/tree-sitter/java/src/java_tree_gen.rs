use std::{
    borrow::BorrowMut,
    cell::Ref,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter, Write},
    fs::File,
    hash::{Hash, Hasher},
    io::{stdout, Stdout},
    marker::PhantomData,
    rc::Rc,
    vec,
};

use num::PrimInt;
use rusted_gumtree_core::tree::tree::{
    HashKind, LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, OwnedLabel, Type,
    WithChildren, WithHashs,
};
use tree_sitter::{Language, Parser, Tree, TreeCursor};

use crate::{spaces::SpacesStore, vec_map_store::VecMapStore};

// use std::any::TypeId;
// use std::process::Command;
// use std::{io::BufReader, rc::Rc, str::FromStr};

// use atomic_counter::RelaxedCounter;
// use num::PrimInt;
// use rusted_gumtree_core::tree::static_analysis::{Declaration, QualifiedName};
// use rusted_gumtree_core::tree::tree::{Label, Type};
// use tree_sitter::{Language, Node, Parser, Tree, TreeCursor};

pub struct JavaTreeGen {
    pub line_break: Vec<u8>,

    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
    // pub(crate) space_store: SpacesStoreD,
}

type SpacesStoreD = SpacesStore<u16, 4>;

pub struct LabelStore {
    internal: VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl LabelStoreTrait for LabelStore {
    type I = LabelIdentifier;
    fn get_id_or_insert_node(&mut self, node: OwnedLabel) -> LabelIdentifier {
        self.internal.get_id_or_insert_node(node)
    }

    fn get_node_at_id<'b>(&'b self, id: &LabelIdentifier) -> Ref<OwnedLabel> {
        self.internal.get_node_at_id(id)
    }
}

pub struct NodeStore {
    internal: VecMapStore<HashedNode, NodeIdentifier>,
}
type HashedNode = HashedCompressedNode<NodeIdentifier, SyntaxNodeHashs<HashSize>>;
impl NodeStoreTrait<HashedNode> for NodeStore {
    fn get_id_or_insert_node(&mut self, node: HashedNode) -> NodeIdentifier {
        self.internal.get_id_or_insert_node(node)
    }

    fn get_node_at_id<'b>(&'b self, id: &NodeIdentifier) -> Ref<HashedNode> {
        self.internal.get_node_at_id(id)
    }
}

extern "C" {
    fn tree_sitter_java() -> Language;
}

#[derive(Debug)]
pub struct FullNode {
    compressible_node: NodeIdentifier,
    depth: usize,
    position: usize,
    height: u32,
    size: u32,
    hashs: SyntaxNodeHashs<u32>,
}

impl FullNode {
    pub fn id(&self) -> &NodeIdentifier {
        &self.compressible_node
    }
}

pub(crate) enum CompressedNode {
    Type(Type),
    Label {
        label: LabelIdentifier,
        kind: Type,
    },
    Children2 {
        children: [NodeIdentifier; 2],
        kind: Type,
    },
    Children {
        children: Box<[NodeIdentifier]>,
        kind: Type,
    },
    Spaces(Box<[Spaces]>),
}

type TypeIdentifier = Type;
type LabelIdentifier = u32;
type NodeIdentifier = u32;
type HashSize = u32;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_size() {
        println!("{}", size_of::<Type>());
        println!("{}", size_of::<CompressedNode>());
    }
}

impl CompressedNode {
    fn new(
        kind: TypeIdentifier,
        label: Option<LabelIdentifier>,
        children: Vec<NodeIdentifier>,
    ) -> Self {
        if children.len() > 2 {
            Self::Children {
                kind,
                children: children.into_boxed_slice(),
            }
        } else if children.len() == 2 {
            Self::Children2 {
                kind,
                children: [children[0], children[1]],
            }
        } else if children.len() > 0 {
            // TODO Children2 Optional child2 might be better
            Self::Children {
                kind,
                children: children.into_boxed_slice(),
            }
        } else if let Some(label) = label {
            Self::Label { kind, label }
        } else {
            Self::Type(kind)
        }
    }
}

impl rusted_gumtree_core::tree::tree::Typed for CompressedNode {
    type Type = Type;

    fn get_type(&self) -> Type {
        match self {
            CompressedNode::Type(kind) => *kind,
            CompressedNode::Label { label: _, kind } => *kind,
            CompressedNode::Children2 { children: _, kind } => *kind,
            CompressedNode::Children { children: _, kind } => *kind,
            CompressedNode::Spaces(_) => Type::Spaces,
        }
    }
}
impl rusted_gumtree_core::tree::tree::Labeled for CompressedNode {
    type Label = LabelIdentifier;

    fn get_label(&self) -> LabelIdentifier {
        match self {
            CompressedNode::Label { label, kind: _ } => *label,
            _ => panic!(),
        }
    }
}

impl rusted_gumtree_core::tree::tree::WithChildren for CompressedNode {
    type ChildIdx = u8;
    fn child_count(&self) -> u8 {
        match self {
            CompressedNode::Children2 {
                children: _,
                kind: _,
            } => 2,
            CompressedNode::Children { children, kind: _ } => children.len() as u8,
            _ => 0,
        }
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> NodeIdentifier {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => children[0],
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => children[1],
            CompressedNode::Children { children, kind: _ } => children[*idx as usize],
            _ => 0,
        }
    }

    // fn descendants_count(&self) -> Self::TreeId {
    //     match self {
    //         CompressedNode::Children2 {
    //             children: _,
    //             kind: _,
    //         } => todo!(),
    //         CompressedNode::Children {
    //             children: _,
    //             kind: _,
    //         } => todo!(),
    //         _ => 0,
    //     }
    // }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        match self {
            CompressedNode::Children2 { children, kind: _ } => &*children,
            CompressedNode::Children { children, kind: _ } => &*children,
            _ => &[],
        }
    }
}
impl rusted_gumtree_core::tree::tree::Node for CompressedNode {}
impl rusted_gumtree_core::tree::tree::Stored for CompressedNode {
    type TreeId = NodeIdentifier;
}

impl rusted_gumtree_core::tree::tree::Tree for CompressedNode {
    fn has_children(&self) -> bool {
        match self {
            CompressedNode::Children2 {
                children: _,
                kind: _,
            } => true,
            CompressedNode::Children {
                children: _,
                kind: _,
            } => true,
            _ => false,
        }
    }

    fn has_label(&self) -> bool {
        match self {
            CompressedNode::Label { label: _, kind: _ } => true,
            _ => false,
        }
    }
}

pub trait NodeHashs<T: PrimInt> {
    type Kind: Default + HashKind;
    fn hash(&self, kind: &Self::Kind) -> T;
}

#[derive(Default, Debug)]
pub struct SyntaxNodeHashs<T: PrimInt> {
    structt: T,
    label: T,
    syntax: T,
}
pub enum SyntaxNodeHashsKinds {
    Struct,
    Label,
    Syntax,
}

impl Default for SyntaxNodeHashsKinds {
    fn default() -> Self {
        Self::Label
    }
}

impl HashKind for SyntaxNodeHashsKinds {
    fn structural() -> Self {
        SyntaxNodeHashsKinds::Struct
    }

    fn label() -> Self {
        SyntaxNodeHashsKinds::Label
    }
}

impl<T: PrimInt> NodeHashs<T> for SyntaxNodeHashs<T> {
    type Kind = SyntaxNodeHashsKinds;
    fn hash(&self, kind: &Self::Kind) -> T {
        match kind {
            SyntaxNodeHashsKinds::Struct => self.structt,
            SyntaxNodeHashsKinds::Label => self.label,
            SyntaxNodeHashsKinds::Syntax => self.syntax,
        }
    }
}

pub struct HashedCompressedNode<T: PrimInt, U: NodeHashs<T>> {
    hashs: U,
    node: CompressedNode,
    phantom: PhantomData<*const T>,
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> Hash for HashedCompressedNode<T, U> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state);
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::Typed
    for HashedCompressedNode<T, U>
{
    type Type = Type;

    fn get_type(&self) -> Type {
        self.node.get_type()
    }
}
impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::Labeled
    for HashedCompressedNode<T, U>
{
    type Label = LabelIdentifier;

    fn get_label(&self) -> LabelIdentifier {
        self.node.get_label()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::Node
    for HashedCompressedNode<T, U>
{
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::Stored
    for HashedCompressedNode<T, U>
{
    type TreeId = NodeIdentifier;
}
impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::WithChildren
    for HashedCompressedNode<T, U>
{
    type ChildIdx = u8;

    fn child_count(&self) -> u8 {
        self.node.child_count()
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> NodeIdentifier {
        self.node.get_child(idx)
    }

    // fn descendants_count(&self) -> Self::TreeId {
    //     self.node.descendants_count()
    // }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        self.node.get_children()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::WithHashs
    for HashedCompressedNode<T, U>
{
    type HK = U::Kind;
    type HP = T;

    fn hash(&self, kind: &Self::HK) -> T {
        self.hashs.hash(kind)
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> rusted_gumtree_core::tree::tree::Tree
    for HashedCompressedNode<T, U>
{
    fn has_children(&self) -> bool {
        self.node.has_children()
    }

    fn has_label(&self) -> bool {
        self.node.has_label()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<T>> HashedCompressedNode<T, U> {
    pub(crate) fn new(hashs: U, node: CompressedNode) -> Self {
        Self {
            hashs,
            node,
            phantom: PhantomData,
        }
    }
}

static ENTER: u32 = {
    let mut result = 1u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'n' as u32;
    result = 31 * result + 't' as u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'r' as u32;
    result
};
static LEAVE: u32 = {
    let mut result = 1u32;
    result = 31 * result + 'l' as u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'a' as u32;
    result = 31 * result + 'v' as u32;
    result = 31 * result + 'e' as u32;
    result
};
static BASE: &u32 = &33u32;

fn inner_node_hash(kind: &u32, label: &u32, size: &u32, middle_hash: &u32) -> u32 {
    let mut left = 1u32;
    left = 31 * left + kind;
    left = 31 * left + label;
    left = 31 * left + ENTER;

    let mut right = 1u32;
    right = 31 * right + kind;
    right = 31 * right + label;
    right = 31 * right + LEAVE;

    left.wrapping_add(*middle_hash)
        .wrapping_add(right.wrapping_mul(hash_factor(size)))
}

fn hash_factor(exponent: &u32) -> u32 {
    fast_exponentiation(BASE, exponent)
}

fn fast_exponentiation(base: &u32, exponent: &u32) -> u32 {
    if exponent == &0 {
        1
    } else if exponent == &1 {
        *base
    } else {
        let mut result: u32 = 1;
        let mut exponent = *exponent;
        let mut base = *base;
        while exponent > 0 {
            if (exponent & 1) != 0 {
                result = result.wrapping_mul(base);
            }
            exponent >>= 1;
            base = base.wrapping_mul(base);
        }
        result
    }
}

// impl Hash for CompressedNode {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match self {
//             CompressedNode::Type(k) => k.hash(state),
//             CompressedNode::Label { kind, label } => {}
//             CompressedNode::Children2 {
//                 kind,
//                 child1,
//                 child2,
//             } => {
//                 let size = 0;
//                 let middle_hash = 0;

//                 let mut k = DefaultHasher::new();
//                 kind.hash(&mut k);
//                 state.write_u32(innerNodeHash(&(((k.finish() & 0xffff0000) >> 32) as u32), &0, &size, &middle_hash));
//             }
//             CompressedNode::Children { kind, children } => {
//                 kind.hash(state);
//                 // children.
//             }
//             CompressedNode::Spaces(s) => s.hash(state),
//         }
//     }
// }

pub struct Acc {
    kind: Type,
    children: Vec<NodeIdentifier>,
    metrics: SubTreeMetrics<u32, SyntaxNodeHashs<u32>>,
    padding_start: usize,
}

#[derive(Default)]
struct SubTreeMetrics<T: PrimInt, U: NodeHashs<T>> {
    hashs: U,
    size: u32,
    height: u32,
    pantom: PhantomData<*const T>,
}

impl Acc {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            kind,
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }
    }

    fn push(&mut self, full_node: FullNode) {
        self.children.push(full_node.compressible_node);
        self.metrics.height = self.metrics.height.max(full_node.height);
        self.metrics.size += full_node.size;
        self.metrics.hashs.structt = self
            .metrics
            .hashs
            .structt
            .wrapping_add(full_node.hashs.structt);
        self.metrics.hashs.label = self.metrics.hashs.label.wrapping_add(full_node.hashs.label);
        self.metrics.hashs.syntax = self
            .metrics
            .hashs
            .syntax
            .wrapping_add(full_node.hashs.syntax);
    }
}

pub struct TypeStore {}

impl TypeStore {
    pub fn get(&mut self, kind: &str) -> TypeIdentifier {
        Type::new(kind)
    }
}

// #[derive(Default)]
// struct LabelStore {
//     hash_table: HashSet<String>,
// }

// impl LabelStore {
//     fn get(&mut self, label: &str) -> &str {
//         if self.hash_table.contains(label) {
//             self.hash_table.get(label).unwrap()
//         } else {
//             self.hash_table.insert(label.to_owned());
//             self.hash_table.get(label).unwrap()
//         }
//     }
// }

// pub struct VecHasher<T: Hash> {
//     state: u64,
//     node_table: Rc<Vec<T>>,
//     default: DefaultHasher,
// }

// impl<T: Hash> Hasher for VecHasher<T> {
//     fn write_u16(&mut self, i: u16) {
//         let a = &self.node_table;
//         let b = &a[i as usize];
//         b.hash(&mut self.default);
//         self.state = self.default.finish();
//     }
//     fn write(&mut self, bytes: &[u8]) {
//         // for &byte in bytes {
//         //     self.state = self.state.rotate_left(8) ^ u64::from(byte);
//         // }
//         panic!()
//     }

//     fn finish(&self) -> u64 {
//         self.state
//     }
// }

// impl<T: Hash> VecHasher<T> {
//     fn hash_identifier(&mut self, id: &NodeIdentifier) {}
// }

// pub(crate) struct BuildVecHasher<T> {
//     node_table: Rc<Vec<T>>,
// }

// impl<T: Hash> std::hash::BuildHasher for BuildVecHasher<T> {
//     type Hasher = VecHasher<T>;
//     fn build_hasher(&self) -> VecHasher<T> {
//         VecHasher {
//             state: 0,
//             node_table: self.node_table.clone(),
//             default: DefaultHasher::new(),
//         }
//     }
// }

// struct NodeStore {
//     hash_table: HashSet<NodeStoreEntry, BuildVecHasher<CompressedNode>>,
//     node_table: Rc<Vec<CompressedNode>>,
//     counter: ConsistentCounter,
// }

// impl Default for NodeStore {
//     fn default() -> Self {
//         let node_table: Rc<Vec<CompressedNode>> = Default::default();
//         Self {
//             hash_table: std::collections::HashSet::with_hasher(BuildVecHasher {
//                 node_table: node_table.clone(),
//             }),
//             node_table,
//             counter: Default::default(),
//         }
//     }
// }

// struct NodeStoreEntry {
//     node: NodeIdentifier,
// }

// impl Hash for NodeStoreEntry {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         state.write_u16(self.node);
//         // CustomHasher::hash_identifier(state, &self.node);
//         // self.hash(state);
//     }
//     // fn hash(&self, state: &mut VecHasher<CompressibleNode>) {
//     //     // if TypeId::of::<H>() == TypeId::of::<VecHasher<CompressibleNode>>() {

//     //     // }
//     //     // CustomHasher::hash_identifier(state, &self.node);
//     //     // self.hash(state);
//     // }
// }

// impl PartialEq for NodeStoreEntry {
//     fn eq(&self, other: &Self) -> bool {
//         self.node == other.node
//     }
// }

// impl Eq for NodeStoreEntry {}

// impl NodeStore {
//     fn get_id_or_insert_node(&mut self, node: CompressedNode) -> NodeIdentifier {
//         let entry = NodeStoreEntry { node: 0 };
//         if self.hash_table.contains(&entry) {
//             self.hash_table.get(&entry).unwrap().node
//         } else {
//             let entry_to_insert = NodeStoreEntry {
//                 node: self.counter.get() as NodeIdentifier,
//             };
//             self.counter.inc();
//             self.hash_table.insert(entry_to_insert);
//             self.hash_table.get(&entry).unwrap().node
//         }
//     }

//     fn get_node_at_id(&self, id: &NodeIdentifier) -> &CompressedNode {
//         &self.node_table[*id as usize]
//     }
// }

fn hash<T: Hash>(x: &T) -> u64 {
    let mut state = DefaultHasher::default();
    x.hash(&mut state);
    state.finish()
}

fn clamp_u64_to_u32(x: &u64) -> u32 {
    ((x & 0xffff0000) >> 32) as u32 ^ (x & 0xffff) as u32
}

impl JavaTreeGen {
    pub fn new() -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(HashedCompressedNode::new(
                SyntaxNodeHashs::default(),
                CompressedNode::Spaces(vec![].into_boxed_slice()),
            )),
        }
    }

    pub fn generate_default(&mut self, text: &[u8], cursor: TreeCursor) -> FullNode {
        let mut acc_stack = vec![Acc::new(self.type_store.get("file"))];
        self.generate(text, cursor, &mut acc_stack)
    }

    pub fn generate(
        &mut self,
        text: &[u8],
        mut cursor: TreeCursor,
        mut acc_stack: &mut Vec<Acc>,
    ) -> FullNode {
        acc_stack.push(Acc {
            kind: self.type_store.get(cursor.node().kind()),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        });

        let mut indentation_stack: Vec<Vec<Spaces>> = vec![];
        indentation_stack.push(JavaTreeGen::compute_indentation(
            &self.line_break,
            text,
            &cursor,
            0,
            &Spaces::format_indentation(&self.line_break),
        ));

        let mut has = Has::Down;
        let mut position = 0;
        let mut depth = 1;
        let mut sum_byte_length; // = cursor.node().start_byte();

        loop {
            sum_byte_length = cursor.node().start_byte();
            if has != Has::Up && cursor.goto_first_child() {
                let parent_indentation = indentation_stack.last().unwrap();
                println!("down: {:?}", cursor.node().kind());
                has = Has::Down;
                // // self.inc(k);
                position += 1;
                depth += 1;

                let indent = JavaTreeGen::compute_indentation(
                    &self.line_break,
                    text,
                    &cursor,
                    sum_byte_length,
                    &parent_indentation,
                );
                indentation_stack.push(indent);

                acc_stack.push(Acc {
                    kind: self.type_store.get(cursor.node().kind()),
                    children: vec![],
                    metrics: Default::default(),
                    padding_start: sum_byte_length,
                });
            } else {
                let parent_indentation = indentation_stack.pop().unwrap();
                let full_node = JavaTreeGen::create_full_node(
                    text,
                    &indentation_stack.last().unwrap_or(&vec![Spaces::LineBreak]),
                    &mut self.node_store,
                    &mut self.label_store,
                    &mut acc_stack,
                    &mut depth,
                    &cursor,
                    sum_byte_length,
                    position,
                );
                sum_byte_length = cursor.node().end_byte();
                if cursor.goto_next_sibling() {
                    println!("right: {:?}", cursor.node().kind());
                    has = Has::Right;
                    // // self.acc(full_node);
                    {
                        let parent = acc_stack.last_mut().unwrap();
                        parent.push(full_node);
                    };
                    // // self.inc(self.kind(cursor.node().kind()));
                    {
                        position += 1;
                        depth += 1;
                        acc_stack.push(Acc {
                            kind: self.type_store.get(cursor.node().kind()),
                            children: vec![],
                            metrics: Default::default(),
                            padding_start: sum_byte_length,
                        });
                    };

                    indentation_stack.push(JavaTreeGen::compute_indentation(
                        &self.line_break,
                        text,
                        &cursor,
                        sum_byte_length,
                        &parent_indentation,
                    ));
                } else {
                    has = Has::Up;
                    if cursor.goto_parent() {
                        println!("up: {:?}", cursor.node().kind());
                        let parent = acc_stack.last_mut().unwrap();
                        parent.push(full_node);
                    } else {
                        return full_node;
                    }
                }
            }
        }
    }

    fn compute_indentation<'a>(
        line_break: &Vec<u8>,
        text: &'a [u8],
        cursor: &TreeCursor,
        padding_start: usize,
        parent_indentation: &'a [Spaces],
    ) -> Vec<Spaces> {
        let spaces = {
            let node = cursor.node();
            let pos = node.start_byte();
            &text[padding_start..pos]
        };
        let spaces_after_lb = spaces_after_lb(&*line_break, spaces);
        match spaces_after_lb {
            Some(s) => Spaces::format_indentation(s),
            None => parent_indentation.to_vec(),
        }
    }

    fn create_full_node(
        text: &[u8],
        old_indentation: &Vec<Spaces>,
        node_store: &mut NodeStore,
        label_store: &mut LabelStore,
        acc_stack: &mut Vec<Acc>,
        depth: &mut usize,
        cursor: &TreeCursor,
        sum_byte_length: usize,
        position: usize,
    ) -> FullNode {
        let node = cursor.node();
        if *depth == 0 {
            if sum_byte_length < text.len() {
                // end of tree but not end of file,
                // thus to be bijective, we need to get the last spaces
                let spaces = Spaces::format_indentation(&text[sum_byte_length..]);
                println!("'{:?}'", &spaces);

                let relativised = Spaces::replace_indentation(&[], &spaces);

                let spaces_leaf = HashedCompressedNode::new(
                    SyntaxNodeHashs {
                        structt: 0,
                        label: 0,
                        syntax: clamp_u64_to_u32(&hash(&relativised)),
                    },
                    CompressedNode::Spaces(relativised.into_boxed_slice()),
                );
                let full_spaces_node = FullNode {
                    hashs: SyntaxNodeHashs {
                        ..spaces_leaf.hashs
                    },
                    compressible_node: node_store.get_id_or_insert_node(spaces_leaf),
                    depth: *depth,
                    position,
                    size: 1,
                    height: 1,
                };
                acc_stack.last_mut().unwrap().push(full_spaces_node);
            }
        }
        let pos = node.start_byte();
        let end = node.end_byte();
        let Acc {
            children,
            kind,
            metrics:
                SubTreeMetrics {
                    hashs:
                        SyntaxNodeHashs {
                            structt: struct_middle_hash,
                            label: label_middle_hash,
                            syntax: syntax_middle_hash,
                        },
                    size,
                    height,
                    pantom: _,
                },
            padding_start,
        } = acc_stack.pop().unwrap();
        println!(
            "node kind {:?} {} {} {}",
            node.kind(),
            struct_middle_hash,
            label_middle_hash,
            syntax_middle_hash
        );
        let label = {
            if node.child(0).is_some() {
                None
            } else if node.is_named() {
                let t = &text[pos..end];
                Some(t.to_vec())
            } else {
                None
            }
        };
        if padding_start != pos {
            let spaces = Spaces::format_indentation(&text[padding_start..pos]);
            println!(
                "ps..pos: '{:?}'",
                std::str::from_utf8(&text[padding_start..pos]).unwrap()
            );
            println!("sbl: '{:?}'", sum_byte_length);
            println!(
                "pos..end: '{:?}'",
                std::str::from_utf8(&text[pos..end]).unwrap()
            );
            let relativised = Spaces::replace_indentation(old_indentation, &spaces);

            let spaces_leaf = HashedCompressedNode::new(
                SyntaxNodeHashs {
                    structt: 0,
                    label: 0,
                    syntax: clamp_u64_to_u32(&hash(&relativised)),
                },
                CompressedNode::Spaces(relativised.into_boxed_slice()),
            );
            let full_spaces_node = FullNode {
                hashs: SyntaxNodeHashs {
                    ..spaces_leaf.hashs
                },
                compressible_node: node_store.get_id_or_insert_node(spaces_leaf),
                depth: *depth,
                position,
                size: 1,
                height: 1,
            };
            if acc_stack.is_empty() {
                println!("kind {:?}", kind);
            }
            println!("kind {:?}", kind);
            println!("oi {:?}", old_indentation);
            println!("s {:?}", spaces);
            println!(
                "r {:?}",
                Spaces::replace_indentation(old_indentation, &spaces)
            );
            println!("kind1 {:?}", acc_stack.last().unwrap().kind);
            acc_stack.last_mut().unwrap().push(full_spaces_node);
        };
        let hashed_label = &clamp_u64_to_u32(&hash(&label));
        let hashed_kind = &clamp_u64_to_u32(&hash(&kind));

        if let Some(t) = &label {
            println!("{:?} label '{:?}'", kind, std::str::from_utf8(&t));
        }
        *depth -= 1;
        let label_id = match label {
            Some(l) => Some(label_store.get_id_or_insert_node(l)),
            None => None,
        };
        let k = *(&kind);
        println!("children {:?} {:?}", k, children.len());
        let compressible_node = HashedCompressedNode::new(
            SyntaxNodeHashs {
                structt: inner_node_hash(hashed_kind, &0, &size, &struct_middle_hash),
                label: inner_node_hash(hashed_kind, hashed_label, &size, &label_middle_hash),
                syntax: inner_node_hash(hashed_kind, hashed_label, &size, &syntax_middle_hash),
            },
            CompressedNode::new(kind, label_id, children),
        );
        println!("hash {:?} {:?}", k, compressible_node.hashs);
        let hashs = SyntaxNodeHashs {
            ..compressible_node.hashs
        };
        let compressible_node_id = node_store.get_id_or_insert_node(compressible_node);
        print_tree_syntax(node_store, label_store, &compressible_node_id);
        println!();
        let full_node = FullNode {
            compressible_node: compressible_node_id,
            depth: *depth,
            position,
            size,
            height,
            hashs,
        };
        full_node
    }

    pub fn main() {
        let mut parser = Parser::new();

        {
            let language = unsafe { tree_sitter_java() };
            parser.set_language(language).unwrap();
        }

        let text = {
            let source_code1 = "class A {void test() {}}";
            source_code1.as_bytes()
        };
        // let mut parser: Parser, old_tree: Option<&Tree>
        let tree = parser.parse(text, None).unwrap();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
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
        };
        let mut acc_stack = vec![Acc {
            kind: java_tree_gen.type_store.get("File"),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }];
        let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);

        print_tree_structure(&java_tree_gen.node_store, &_full_node.compressible_node);

        let mut acc_stack = vec![Acc {
            kind: java_tree_gen.type_store.get("File"),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }];
        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);
    }
    // fn generate<'a>(&mut self, text: &'a [u8], tc: TreeContext, init_acc:ChildrenAcc<'a>) -> FullNode {
    //     let mut tree = self.parser.parse(text, self.old_tree.as_ref()).unwrap();
    //     println!("{}", tree.root_node().to_sexp());
    //     let full_node = self.build_compressed(text, &mut tree, tc, init_acc);
    //     self.old_tree = Option::Some(tree);
    //     full_node
    // }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    let node = node_store.get_node_at_id(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
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
    let node = node_store.get_node_at_id(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.get_node_at_id(label);
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
    let node = node_store.get_node_at_id(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.get_node_at_id(label);
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
    let node = node_store.get_node_at_id(id);
    match &node.node {
        CompressedNode::Type(kind) => {
            out.write_str(&kind.to_string()).unwrap();
            // out.write_fmt(format_args!("{}",kind.to_string())).unwrap();
            None
        }
        CompressedNode::Label { kind: _, label } => {
            let s = &label_store.get_node_at_id(label);
            out.write_str(&std::str::from_utf8(s).unwrap()).unwrap();
            // write!(&mut out, "{}", std::str::from_utf8(s).unwrap()).unwrap();
            None
        }
        CompressedNode::Children2 { kind: _, children } => {
            let ind = serialize(node_store, label_store, &children[0], out, parent_indent)
                .unwrap_or(parent_indent.to_owned());
            serialize(node_store, label_store, &children[1], out, &ind);
            None
        }
        CompressedNode::Children { kind, children } => {
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
            .unwrap_or(parent_indent.to_owned());
            for id in it {
                ind = serialize(node_store, label_store, &id, out, &ind)
                    .unwrap_or(parent_indent.to_owned());
            }
            None
        }
        CompressedNode::Spaces(s) => {
            let a = &**s;
            let mut b = String::new();
            // let mut b = format!("{:#?}", a);
            // fmt::format(args)
            a.iter()
                .for_each(|a| Spaces::fmt(a, &mut b, parent_indent).unwrap());
            // std::io::Write::write_all(out, "<|".as_bytes()).unwrap();
            // std::io::Write::write_all(out, parent_indent.replace("\n", "n").as_bytes()).unwrap();
            // std::io::Write::write_all(out, "|>".as_bytes()).unwrap();
            out.write_str(&b).unwrap();
            Some(if b.contains("\n") {
                b
            } else {
                parent_indent.to_owned()
            })
        }
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub(crate) enum Spaces {
    Space,
    LineBreak,
    // NewLine,
    // CariageReturn,
    Tabulation,
    ParentIndentation,
}

impl Display for Spaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Spaces::Space => write!(f, "s"),
            Spaces::LineBreak => write!(f, "n"),
            // Spaces::NewLine => write!(f, "n"),
            // Spaces::CariageReturn => write!(f, "r"),
            Spaces::Tabulation => write!(f, "t"),
            Spaces::ParentIndentation => write!(f, "0"),
        }
        // match self {
        //     Spaces::Space => write!(f, " "),
        //     Spaces::LineBreak => write!(f, "\n"),
        //     // Spaces::NewLine => write!(f, "n"),
        //     // Spaces::CariageReturn => write!(f, "r"),
        //     Spaces::Tabulation => write!(f, "\t"),
        //     Spaces::ParentIndentation => panic!(),
        // }
    }
}

impl Spaces {
    fn fmt<W: Write>(&self, w: &mut W, p: &str) -> std::fmt::Result {
        match self {
            Spaces::Space => write!(w, " "),
            Spaces::LineBreak => write!(w, "\n"),
            Spaces::Tabulation => write!(w, "\t"),
            Spaces::ParentIndentation => write!(w, "{}", p),
        }
    }
}

impl Debug for Spaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Spaces::Space => write!(f, "s"),
            Spaces::LineBreak => write!(f, "n"),
            // Spaces::NewLine => write!(f, "n"),
            // Spaces::CariageReturn => write!(f, "r"),
            Spaces::Tabulation => write!(f, "t"),
            Spaces::ParentIndentation => write!(f, "0"),
        }
    }
}
const LB: char = '\n';
impl Spaces {
    fn format_indentation(spaces: &[u8]) -> Vec<Spaces> {
        spaces
            .iter()
            .map(|x| match *x as char {
                ' ' => Spaces::Space,
                LB => Spaces::LineBreak,
                '\t' => Spaces::Tabulation,
                _ => panic!(),
            })
            .collect()
    }

    pub(crate) fn replace_indentation(indentation: &[Spaces], spaces: &[Spaces]) -> Vec<Spaces> {
        let mut r = vec![];
        let mut tmp = vec![];
        let mut i = 0;
        for x in spaces {
            tmp.push(*x);
            if i < indentation.len() && indentation[i] == *x {
                i += 1;
                if i == indentation.len() {
                    r.push(Spaces::ParentIndentation);
                    tmp.clear();
                }
            } else {
                i = 0;
                r.extend_from_slice(&*tmp);
                tmp.clear();
            }
        }
        r
    }

    // pub(crate) fn replace_indentation(indentation: &[Spaces], spaces: &[Spaces]) -> Vec<Spaces> {
    //     if spaces.len() < indentation.len() {
    //         return spaces.to_vec();
    //     }
    //     if indentation.len() == 0 {
    //         assert!(false);
    //     }
    //     let mut it = spaces.windows(indentation.len());
    //     let mut r: Vec<Spaces> = vec![];
    //     let mut tmp: &[Spaces] = &[];
    //     loop {
    //         match it.next() {
    //             Some(x) => {
    //                 if x == indentation {
    //                     r.push(Spaces::ParentIndentation);
    //                     for _ in 0..indentation.len()-1 {
    //                         it.next();
    //                     }
    //                     tmp = &[];
    //                 } else {
    //                     if tmp.len()>0 {
    //                         r.push(tmp[0]);
    //                     }
    //                     tmp = x;
    //                 }
    //             }
    //             None => {
    //                 r.extend(tmp);
    //                 return r
    //             },
    //         }
    //     }
    // }
}

#[derive(PartialEq, Eq)]
enum Has {
    Down,
    Up,
    Right,
}

pub(crate) fn spaces_after_lb<'b>(lb: &[u8], spaces: &'b [u8]) -> Option<&'b [u8]> {
    spaces
        .windows(lb.len())
        .rev()
        .position(|window| window == lb)
        .and_then(|i| Some(&spaces[spaces.len() - i - 1..]))
}
impl NodeStore {
    pub(crate) fn new(filling_element: HashedCompressedNode<u32, SyntaxNodeHashs<u32>>) -> Self {
        Self {
            internal: VecMapStore::new(filling_element),
        }
    }
}
impl LabelStore {
    pub(crate) fn new() -> Self {
        Self {
            internal: VecMapStore::new(vec![]),
        }
    }
}

// pub(crate) fn format_indentation_windows(spaces: &[u8]) -> Vec<Spaces> {
//     const line_break:&[u8] = "\r\n".as_bytes();
//     let mut it = spaces.windows(line_break.len());
//     let mut r: Vec<Spaces> = vec![];
//     loop {
//         match it.next() {
//             Some(x) => {
//                 if x == line_break {
//                     r.push(Spaces::LineBreak);
//                     for _ in 0..line_break.len() {
//                         it.next();
//                     }
//                 } else if ' ' as u8 == x[0] {
//                     r.push(Spaces::Space);
//                 } else if '\t' as u8 == x[0] {
//                     r.push(Spaces::Tabulation);
//                 } else {
//                     println!("not a space: {:?}", String::from_utf8(x.to_vec()));
//                     panic!()
//                 }
//             }
//             None => return r,
//         }
//     }
// }

// pub(crate) fn replace_indentation_old<'b>(indentation: &[u8], spaces: &'b [u8]) -> Vec<Spaces> {
//     let mut it = spaces.windows(indentation.len());
//     // .windows(|i| Some(&spaces[spaces.len() - i..]));
//     let mut r: Vec<Spaces> = vec![];
//     // let mut old = 0;
//     loop {
//         match it.next() {
//             Some(x) => {
//                 if x == indentation {
//                     r.push(Spaces::ParentIndentation);
//                     for _ in 0..indentation.len() {
//                         it.next();
//                     }
//                 } else if ' ' as u8 == x[0] {
//                     r.push(Spaces::Space);
//                 // } else if '\n' as u8 == x[0] {
//                 //     r.push(Spaces::NewLine);
//                 // } else if '\r' as u8 == x[0] {
//                 //     r.push(Spaces::CariageReturn);
//                 } else if '\t' as u8 == x[0] {
//                     r.push(Spaces::Tabulation);
//                 } else {
//                     println!("not a space: {:?}", String::from_utf8(x.to_vec()));
//                     panic!()
//                 }
//             }
//             None => return r,
//         }
//     }
// }
